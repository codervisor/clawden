use anyhow::Result;
use clawden_config::{ChannelInstanceYaml, ClawDenYaml};
use dialoguer::{Input, Password, Select};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::fmt;
use std::io::{self, IsTerminal, Write};
use std::time::Duration;

use crate::cli::FeishuCommand;

const DEFAULT_FEISHU_API_BASE_URL: &str = "https://open.feishu.cn";
const FEISHU_EVENT_REMINDERS: &[&str] = &["im.message.receive_v1"];
const FEISHU_PERMISSION_REMINDERS: &[&str] = &["im:message", "im:message:send_as_bot"];

pub async fn exec_feishu(command: FeishuCommand) -> Result<()> {
    match command {
        FeishuCommand::Verify {
            app_id,
            app_secret,
            channel,
        } => {
            exec_verify(VerifyOptions {
                app_id,
                app_secret,
                channel,
            })
            .await
        }
        FeishuCommand::Setup => exec_setup().await,
    }
}

#[derive(Debug, Clone)]
struct VerifyOptions {
    app_id: Option<String>,
    app_secret: Option<String>,
    channel: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FeishuCredentials {
    channel_name: Option<String>,
    app_id: String,
    app_secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FeishuVerification {
    bot_info: BotInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BotInfo {
    name: Option<String>,
    open_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VerifyFailure {
    InvalidCredentials { message: String },
    BotCapabilityDisabled { message: String },
    Transport { message: String },
}

impl fmt::Display for VerifyFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCredentials { message }
            | Self::BotCapabilityDisabled { message }
            | Self::Transport { message } => f.write_str(message),
        }
    }
}

struct FeishuVerifier {
    client: Client,
    base_url: String,
}

impl FeishuVerifier {
    fn new() -> Result<Self> {
        let base_url = std::env::var("CLAWDEN_FEISHU_API_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_FEISHU_API_BASE_URL.to_string());
        Self::with_base_url(&base_url)
    }

    fn with_base_url(base_url: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| anyhow::anyhow!("failed to build Feishu HTTP client: {e}"))?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    async fn verify(
        &self,
        app_id: &str,
        app_secret: &str,
    ) -> std::result::Result<FeishuVerification, VerifyFailure> {
        let token = self.obtain_tenant_access_token(app_id, app_secret).await?;
        let bot_info = self.fetch_bot_info(&token).await?;
        Ok(FeishuVerification { bot_info })
    }

    async fn obtain_tenant_access_token(
        &self,
        app_id: &str,
        app_secret: &str,
    ) -> std::result::Result<String, VerifyFailure> {
        let url = format!(
            "{}/open-apis/auth/v3/tenant_access_token/internal",
            self.base_url
        );
        let (status, payload) = self
            .send_json(
                self.client
                    .post(url)
                    .json(&json!({ "app_id": app_id, "app_secret": app_secret })),
            )
            .await?;

        parse_tenant_token_response(status, &payload)
    }

    async fn fetch_bot_info(
        &self,
        tenant_access_token: &str,
    ) -> std::result::Result<BotInfo, VerifyFailure> {
        let url = format!("{}/open-apis/bot/v3/info", self.base_url);
        let (status, payload) = self
            .send_json(self.client.get(url).bearer_auth(tenant_access_token))
            .await?;

        parse_bot_info_response(status, &payload)
    }

    async fn send_json(
        &self,
        request: reqwest::RequestBuilder,
    ) -> std::result::Result<(StatusCode, Value), VerifyFailure> {
        let response = request.send().await.map_err(|e| VerifyFailure::Transport {
            message: format!("Feishu API request failed: {e}"),
        })?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| VerifyFailure::Transport {
                message: format!("failed to read Feishu API response: {e}"),
            })?;
        let payload = serde_json::from_str(&body).unwrap_or_else(|_| json!({ "raw": body }));
        Ok((status, payload))
    }
}

async fn exec_verify(options: VerifyOptions) -> Result<()> {
    let credentials = resolve_feishu_credentials(
        options.app_id.as_deref(),
        options.app_secret.as_deref(),
        options.channel.as_deref(),
    )?;
    let verifier = FeishuVerifier::new()?;
    let verification = verifier
        .verify(&credentials.app_id, &credentials.app_secret)
        .await
        .map_err(anyhow::Error::msg)?;

    print_verification(&credentials, &verification);
    Ok(())
}

async fn exec_setup() -> Result<()> {
    print_setup_guide();

    let app_id = prompt_text("App ID")?;
    let app_secret = prompt_secret("App Secret")?;
    let credentials = FeishuCredentials {
        channel_name: Some("feishu".to_string()),
        app_id,
        app_secret,
    };

    println!("\nVerifying credentials...");
    let verifier = FeishuVerifier::new()?;
    let verification = verifier
        .verify(&credentials.app_id, &credentials.app_secret)
        .await
        .map_err(anyhow::Error::msg)?;

    print_verification(&credentials, &verification);
    print_config_snippet(&credentials);
    Ok(())
}

fn resolve_feishu_credentials(
    app_id_override: Option<&str>,
    app_secret_override: Option<&str>,
    channel_name: Option<&str>,
) -> Result<FeishuCredentials> {
    let env_app_id = env_credential("FEISHU_APP_ID");
    let env_app_secret = env_credential("FEISHU_APP_SECRET");
    let need_config_lookup = channel_name.is_some()
        || app_id_override.is_none() && env_app_id.is_none()
        || app_secret_override.is_none() && env_app_secret.is_none();
    let selected_channel = if need_config_lookup {
        let config = super::up::load_config_with_env_file(None)?;
        match config {
            Some(config) => select_feishu_channel(&config, channel_name)?,
            None if channel_name.is_some() => {
                anyhow::bail!("--channel requires clawden.yaml in the current directory");
            }
            None => None,
        }
    } else {
        None
    };

    let channel_ref = selected_channel.as_ref().map(|(_, channel)| channel);
    let app_id = app_id_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or(env_app_id)
        .or_else(|| channel_field(channel_ref, "app_id"))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing Feishu app_id; set FEISHU_APP_ID, pass --app-id, or configure channels.<name>.app_id"
            )
        })?;
    let app_secret = app_secret_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or(env_app_secret)
        .or_else(|| channel_field(channel_ref, "app_secret"))
        .ok_or_else(|| anyhow::anyhow!("missing Feishu app_secret; set FEISHU_APP_SECRET, pass --app-secret, or configure channels.<name>.app_secret"))?;

    Ok(FeishuCredentials {
        channel_name: selected_channel.map(|(name, _)| name),
        app_id,
        app_secret,
    })
}

fn env_credential(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn select_feishu_channel(
    config: &ClawDenYaml,
    channel_name: Option<&str>,
) -> Result<Option<(String, ChannelInstanceYaml)>> {
    let mut channels = config
        .channels
        .iter()
        .filter_map(|(name, channel)| {
            let channel_type = ClawDenYaml::resolve_channel_type(name, channel)
                .unwrap_or_else(|| name.clone())
                .to_ascii_lowercase();
            if channel_type == "feishu" || channel_type == "lark" {
                Some((name.clone(), channel.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    channels.sort_by(|left, right| left.0.cmp(&right.0));

    if let Some(requested_name) = channel_name {
        return channels
            .into_iter()
            .find(|(name, _)| name == requested_name)
            .map(Some)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "channel '{requested_name}' is not configured as a Feishu/Lark channel"
                )
            });
    }

    match channels.len() {
        0 => Ok(None),
        1 => Ok(channels.into_iter().next()),
        _ => {
            let labels = channels
                .iter()
                .map(|(name, channel)| {
                    let app_id = channel
                        .extra
                        .get("app_id")
                        .and_then(Value::as_str)
                        .unwrap_or("<missing app_id>");
                    format!("{name} ({app_id})")
                })
                .collect::<Vec<_>>();
            let selection = prompt_select("Select Feishu channel to verify", &labels, 0)?;
            Ok(channels.into_iter().nth(selection))
        }
    }
}

fn channel_field(channel: Option<&ChannelInstanceYaml>, field: &str) -> Option<String> {
    channel
        .and_then(|value| value.extra.get(field))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn print_verification(credentials: &FeishuCredentials, verification: &FeishuVerification) {
    println!("Feishu app verification:");
    if let Some(channel_name) = &credentials.channel_name {
        println!("  Channel:          {channel_name}");
    }
    println!("  App ID:           {}", credentials.app_id);
    println!("  Credentials:      valid (tenant token obtained)");
    println!("  Bot capability:   enabled");
    if let Some(name) = &verification.bot_info.name {
        println!("  Bot name:         {name}");
    }
    if let Some(open_id) = &verification.bot_info.open_id {
        println!("  Bot open_id:      {open_id}");
    }
    println!();
    println!("Reminder: ensure these event subscriptions are enabled:");
    for event in FEISHU_EVENT_REMINDERS {
        println!("  - {event}");
    }
    println!();
    println!("Reminder: ensure these permissions are granted:");
    for permission in FEISHU_PERMISSION_REMINDERS {
        println!("  - {permission}");
    }
    println!();
    println!("Note: ClawDen runtimes use long connection mode (WebSocket).");
    println!("No webhook URL or verification token is required.");
}

fn print_setup_guide() {
    println!("----------------------------------------");
    println!("  Feishu Bot Setup Guide");
    println!("----------------------------------------");
    println!();
    println!("Step 1: Create a Feishu App");
    println!("  Open: https://open.feishu.cn/app");
    println!("  Click 'Create Custom App'");
    println!("  Choose 'Enterprise Custom App'");
    println!("  Give it a name and description");
    println!();
    println!("Step 2: Get Credentials");
    println!("  Go to 'Credentials & Basic Info' in your app settings");
    println!("  Copy the App ID and App Secret");
    println!();
    println!("Step 3: Enable Bot Capability");
    println!("  Go to 'Add Features' -> 'Bot'");
    println!("  Enable the bot feature and set a bot name");
    println!();
    println!("Step 4: Configure Event Subscriptions");
    println!("  Go to 'Event Subscriptions'");
    println!("  Select 'Long Connection' mode");
    println!("  Add event: im.message.receive_v1");
    println!();
    println!("Step 5: Set Permissions");
    println!("  Go to 'Permissions & Scopes'");
    println!("  Add: im:message, im:message:send_as_bot");
    println!();
    println!("Step 6: Publish");
    println!("  Go to 'Version Management & Release'");
    println!("  Create a new version and submit for review");
    println!("  For testing, use 'Create Test Version' for immediate access");
}

fn print_config_snippet(credentials: &FeishuCredentials) {
    println!();
    println!("Your clawden.yaml channel config:");
    println!();
    println!("channels:");
    println!("  feishu:");
    println!("    type: feishu");
    println!("    app_id: $FEISHU_APP_ID");
    println!("    app_secret: $FEISHU_APP_SECRET");
    println!();
    println!("Add these to your .env file:");
    println!("FEISHU_APP_ID={}", credentials.app_id);
    println!("FEISHU_APP_SECRET={}", credentials.app_secret);
}

fn prompt_text(prompt: &str) -> Result<String> {
    if io::stdin().is_terminal() {
        let value: String = Input::new().with_prompt(prompt).interact_text()?;
        return Ok(value.trim().to_string());
    }

    print!("{prompt}: ");
    io::stdout().flush()?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

fn prompt_select(prompt: &str, labels: &[String], default: usize) -> Result<usize> {
    if io::stdin().is_terminal() {
        return Ok(Select::new()
            .with_prompt(prompt)
            .items(labels)
            .default(default)
            .interact()?);
    }

    println!("{prompt}:");
    for (index, label) in labels.iter().enumerate() {
        println!("  {}. {label}", index + 1);
    }
    print!("Selection [{}]: ", default + 1);
    io::stdout().flush()?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return Ok(default);
    }

    if let Ok(index) = trimmed.parse::<usize>() {
        if (1..=labels.len()).contains(&index) {
            return Ok(index - 1);
        }
    }

    labels
        .iter()
        .position(|label| label == trimmed)
        .ok_or_else(|| anyhow::anyhow!("invalid selection '{trimmed}'"))
}

fn prompt_secret(prompt: &str) -> Result<String> {
    if io::stdin().is_terminal() {
        let value = Password::new().with_prompt(prompt).interact()?;
        return Ok(value.trim().to_string());
    }

    prompt_text(prompt)
}

fn parse_tenant_token_response(
    status: StatusCode,
    payload: &Value,
) -> std::result::Result<String, VerifyFailure> {
    if !status.is_success() {
        return Err(VerifyFailure::InvalidCredentials {
            message: invalid_credentials_message(extract_api_message(payload)),
        });
    }

    let code = payload.get("code").and_then(Value::as_i64).unwrap_or(0);
    if code != 0 {
        return Err(VerifyFailure::InvalidCredentials {
            message: invalid_credentials_message(extract_api_message(payload)),
        });
    }

    payload
        .get("tenant_access_token")
        .and_then(Value::as_str)
        .filter(|token| !token.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| VerifyFailure::Transport {
            message: "Feishu auth API returned success without tenant_access_token".to_string(),
        })
}

fn parse_bot_info_response(
    status: StatusCode,
    payload: &Value,
) -> std::result::Result<BotInfo, VerifyFailure> {
    if !status.is_success() {
        return Err(VerifyFailure::BotCapabilityDisabled {
            message: bot_capability_message(extract_api_message(payload)),
        });
    }

    let code = payload.get("code").and_then(Value::as_i64).unwrap_or(0);
    if code != 0 {
        return Err(VerifyFailure::BotCapabilityDisabled {
            message: bot_capability_message(extract_api_message(payload)),
        });
    }

    let data = payload.get("data").unwrap_or(payload);
    let name = data
        .get("bot_name")
        .or_else(|| data.get("name"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let open_id = data
        .get("open_id")
        .or_else(|| data.get("bot_open_id"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    Ok(BotInfo { name, open_id })
}

fn extract_api_message(payload: &Value) -> Option<String> {
    payload
        .get("msg")
        .or_else(|| payload.get("message"))
        .or_else(|| payload.get("error"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            payload
                .get("raw")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|message| !message.is_empty())
                .map(ToOwned::to_owned)
        })
}

fn invalid_credentials_message(details: Option<String>) -> String {
    match details {
        Some(details) => format!(
            "Invalid credentials. Check App ID and App Secret in your Feishu Developer Console. Feishu said: {details}"
        ),
        None => "Invalid credentials. Check App ID and App Secret in your Feishu Developer Console.".to_string(),
    }
}

fn bot_capability_message(details: Option<String>) -> String {
    match details {
        Some(details) => format!(
            "Bot capability is not enabled. Go to your app settings -> Add Features -> Bot. Feishu said: {details}"
        ),
        None => {
            "Bot capability is not enabled. Go to your app settings -> Add Features -> Bot."
                .to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_bot_info_response, parse_tenant_token_response, resolve_feishu_credentials, BotInfo,
        VerifyFailure,
    };
    use reqwest::StatusCode;
    use serde_json::json;

    #[test]
    fn parses_tenant_token_success_payload() {
        let token = parse_tenant_token_response(
            StatusCode::OK,
            &json!({"code": 0, "tenant_access_token": "tenant-token"}),
        )
        .expect("token should parse");

        assert_eq!(token, "tenant-token");
    }

    #[test]
    fn surfaces_invalid_credentials_message() {
        let err = parse_tenant_token_response(
            StatusCode::OK,
            &json!({"code": 99991663, "msg": "invalid app secret"}),
        )
        .expect_err("credentials should fail");

        assert_eq!(
            err,
            VerifyFailure::InvalidCredentials {
                message: "Invalid credentials. Check App ID and App Secret in your Feishu Developer Console. Feishu said: invalid app secret".to_string(),
            }
        );
    }

    #[test]
    fn parses_bot_info_payload() {
        let bot = parse_bot_info_response(
            StatusCode::OK,
            &json!({
                "code": 0,
                "data": {
                    "bot_name": "ClawDen Helper",
                    "open_id": "ou_xxx"
                }
            }),
        )
        .expect("bot info should parse");

        assert_eq!(
            bot,
            BotInfo {
                name: Some("ClawDen Helper".to_string()),
                open_id: Some("ou_xxx".to_string()),
            }
        );
    }

    #[test]
    fn resolve_credentials_uses_flag_values_without_yaml() {
        let creds = resolve_feishu_credentials(Some("cli_test"), Some("secret"), None)
            .expect("flags should be enough");

        assert_eq!(creds.channel_name, None);
        assert_eq!(creds.app_id, "cli_test");
        assert_eq!(creds.app_secret, "secret");
    }
}
