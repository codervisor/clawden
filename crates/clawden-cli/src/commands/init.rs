use anyhow::Result;
use clawden_config::{ClawDenYaml, ProviderRefYaml};
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::collections::BTreeSet;
use std::io::{self, IsTerminal, Write};

use crate::util::{append_audit_file, parse_runtime, store_provider_key_in_vault};

#[derive(Debug, Clone)]
pub struct InitOptions {
    pub runtime: String,
    pub multi: bool,
    pub template: Option<String>,
    pub reconfigure: bool,
    pub non_interactive: bool,
    pub yes: bool,
    pub force: bool,
}

#[derive(Debug, Clone, Copy)]
enum TemplateKind {
    TelegramBot,
    DiscordBot,
    ApiOnly,
    MultiRuntime,
}

#[derive(Debug, Clone)]
struct WizardSelection {
    runtime: String,
    multi: bool,
    mode: Option<String>,
    provider: Option<String>,
    provider_key: Option<String>,
    model: Option<String>,
    channels: Vec<String>,
    tools: Vec<String>,
}

pub fn exec_init(options: InitOptions) -> Result<()> {
    let yaml_path = std::env::current_dir()?.join("clawden.yaml");
    if yaml_path.exists() && !options.force && !options.reconfigure {
        anyhow::bail!("clawden.yaml already exists. Use --reconfigure or --force.");
    }

    let _ = parse_runtime(&options.runtime)?;

    let template = options
        .template
        .as_deref()
        .map(parse_template)
        .transpose()?;

    let mut selection = load_existing_selection(&yaml_path, &options)?;
    let interactive = !options.non_interactive && !options.yes && io::stdin().is_terminal();

    if interactive && template.is_none() {
        selection = run_wizard(selection)?;
    }

    let yaml_content = if let Some(kind) = template {
        render_template(kind, &selection.runtime)
    } else {
        render_wizard_yaml(&selection)
    };

    std::fs::write(&yaml_path, &yaml_content)?;
    println!("Created {}", yaml_path.display());

    let env_path = yaml_path.parent().unwrap().join(".env");
    ensure_env_file(&env_path, template, &selection)?;

    if let (Some(provider), Some(key)) = (&selection.provider, &selection.provider_key) {
        if !key.is_empty() {
            let path = store_provider_key_in_vault(provider, key)?;
            println!("Stored encrypted provider key in {}", path.display());
        }
    }

    append_audit_file("project.init", &selection.runtime, "ok")?;
    Ok(())
}

fn load_existing_selection(
    yaml_path: &std::path::Path,
    options: &InitOptions,
) -> Result<WizardSelection> {
    if options.reconfigure && yaml_path.exists() {
        let mut parsed = ClawDenYaml::from_file(yaml_path).map_err(anyhow::Error::msg)?;
        let _ = parsed.resolve_env_vars();

        let runtime = parsed
            .runtime
            .clone()
            .or_else(|| parsed.runtimes.first().map(|rt| rt.name.clone()))
            .unwrap_or_else(|| options.runtime.clone());
        let multi = !parsed.runtimes.is_empty();
        let channels = if let Some(first_rt) = parsed.runtimes.first() {
            first_rt.channels.clone()
        } else {
            Vec::new()
        };
        let tools = if let Some(first_rt) = parsed.runtimes.first() {
            first_rt.tools.clone()
        } else if !parsed.tools.is_empty() {
            parsed.tools.clone()
        } else {
            vec!["git".to_string(), "http".to_string()]
        };
        let provider = if let Some(first_rt) = parsed.runtimes.first() {
            first_rt.provider.clone()
        } else {
            match parsed.provider {
                Some(ProviderRefYaml::Name(name)) => Some(name),
                Some(ProviderRefYaml::Inline(entry)) => entry
                    .provider_type
                    .map(|pt| format!("{:?}", pt).to_ascii_lowercase()),
                None => None,
            }
        };
        let model = parsed
            .model
            .clone()
            .or_else(|| parsed.runtimes.first().and_then(|rt| rt.model.clone()));

        return Ok(WizardSelection {
            runtime,
            multi,
            mode: parsed.mode.clone(),
            provider,
            provider_key: None,
            model,
            channels,
            tools,
        });
    }

    Ok(WizardSelection {
        runtime: options.runtime.clone(),
        multi: options.multi,
        mode: None,
        provider: Some("openai".to_string()),
        provider_key: None,
        model: Some("gpt-4o-mini".to_string()),
        channels: Vec::new(),
        tools: vec!["git".to_string(), "http".to_string()],
    })
}

fn run_wizard(mut selection: WizardSelection) -> Result<WizardSelection> {
    println!("Welcome to ClawDen! Let's set up your project.\n");

    // Step 1: Execution Mode
    println!("Step 1/5 - Execution Mode");
    let mode_default = match selection.mode.as_deref() {
        Some("direct") => 1,
        _ => 0,
    };
    let mode_idx = Select::new()
        .with_prompt("How do you want to run claw runtimes?")
        .items(&["Docker (recommended)", "Direct install (no Docker)"])
        .default(mode_default)
        .interact()?;
    selection.mode = Some(if mode_idx == 1 { "direct" } else { "docker" }.to_string());

    // Step 2: Runtime Selection
    println!("\nStep 2/5 - Runtime Selection");
    let runtime: String = Input::new()
        .with_prompt("Runtime")
        .default(selection.runtime.clone())
        .interact_text()?;
    let _ = parse_runtime(&runtime)?;
    selection.runtime = runtime;
    selection.multi = Confirm::new()
        .with_prompt("Use multi-runtime config?")
        .default(selection.multi)
        .interact()?;

    // Step 3: Channel Configuration
    println!("\nStep 3/5 - Channel Configuration");
    let channel_options = ["telegram", "discord", "slack"];
    let detected_channels = detect_channel_envs();
    let channel_defaults: Vec<bool> = channel_options
        .iter()
        .map(|ch| {
            selection.channels.iter().any(|c| c == ch) || detected_channels.iter().any(|c| c == ch)
        })
        .collect();
    if !detected_channels.is_empty() {
        println!("Detected channel tokens in environment:");
        for ch in &detected_channels {
            println!("  [x] {ch}");
        }
    }
    let channel_indices = MultiSelect::new()
        .with_prompt("Select channels (↑↓ navigate, space toggle, enter confirm)")
        .items(&channel_options)
        .defaults(&channel_defaults)
        .interact()?;
    selection.channels = channel_indices
        .into_iter()
        .map(|i| channel_options[i].to_string())
        .collect();

    // Step 4: LLM Provider
    println!("\nStep 4/5 - LLM Provider");
    let provider_options = [
        "openai",
        "anthropic",
        "google",
        "openrouter",
        "local (openai-compatible)",
        "skip",
    ];
    let detected_default = print_detected_provider_envs();
    let provider_idx = Select::new()
        .with_prompt("Choose provider")
        .items(&provider_options)
        .default(detected_default)
        .interact()?;
    selection.provider = match provider_idx {
        0 => Some("openai".to_string()),
        1 => Some("anthropic".to_string()),
        2 => Some("google".to_string()),
        3 => Some("openrouter".to_string()),
        4 => Some("local-llm".to_string()),
        _ => None,
    };
    if selection.provider.is_some()
        && Confirm::new()
            .with_prompt("Store provider API key in local encrypted vault now?")
            .default(false)
            .interact()?
    {
        print!("Enter API key (input hidden): ");
        io::stdout().flush()?;
        let key = rpassword::read_password()?;
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            selection.provider_key = Some(trimmed.to_string());
        }
    }

    // Step 5: Tools
    println!("\nStep 5/5 - Tools");
    let tool_options = [
        "git",
        "http",
        "core-utils",
        "python",
        "code-tools",
        "database",
    ];
    let tool_defaults: Vec<bool> = tool_options
        .iter()
        .map(|t| selection.tools.iter().any(|s| s == t))
        .collect();
    let tool_indices = MultiSelect::new()
        .with_prompt("Select built-in tools (↑↓ navigate, space toggle, enter confirm)")
        .items(&tool_options)
        .defaults(&tool_defaults)
        .interact()?;
    selection.tools = tool_indices
        .into_iter()
        .map(|i| tool_options[i].to_string())
        .collect();
    if selection.tools.is_empty() {
        selection.tools = vec!["git".to_string(), "http".to_string()];
    }

    println!("\nSetup complete. Generating files...");
    Ok(selection)
}

fn render_wizard_yaml(selection: &WizardSelection) -> String {
    let mut yaml =
        String::from("# ClawDen config\n# Docs: https://github.com/codervisor/clawden\n\n");

    if let Some(mode) = &selection.mode {
        yaml.push_str(&format!("mode: {mode}\n\n"));
    }

    if selection.multi {
        yaml.push_str("channels:\n");
        if selection.channels.is_empty() {
            yaml.push_str("  {}\n");
        } else {
            for ch in &selection.channels {
                let env_name = match ch.as_str() {
                    "telegram" => "TELEGRAM_BOT_TOKEN",
                    "discord" => "DISCORD_BOT_TOKEN",
                    "slack" => "SLACK_BOT_TOKEN",
                    _ => "CHANNEL_TOKEN",
                };
                yaml.push_str(&format!(
                    "  {ch}:\n    type: {ch}\n    token: ${env_name}\n"
                ));
            }
        }

        if let Some(name) = &selection.provider {
            yaml.push_str("\nproviders:\n");
            yaml.push_str(&format!(
                "  {name}:\n    type: {}\n    api_key: ${}\n",
                provider_type_for_name(name),
                env_var_for_provider(name).unwrap_or("OPENAI_API_KEY")
            ));
        }

        yaml.push_str("\nruntimes:\n");
        yaml.push_str(&format!("  - name: {}\n", selection.runtime));
        if selection.channels.is_empty() {
            yaml.push_str("    channels: []\n");
        } else {
            yaml.push_str(&format!(
                "    channels: [{}]\n",
                selection.channels.join(", ")
            ));
        }
        yaml.push_str(&format!("    tools: [{}]\n", selection.tools.join(", ")));
        if let Some(provider) = &selection.provider {
            yaml.push_str(&format!("    provider: {provider}\n"));
        }
        if let Some(model) = &selection.model {
            yaml.push_str(&format!("    model: {model}\n"));
        }
    } else {
        yaml.push_str(&format!("runtime: {}\n\n", selection.runtime));
        if selection.channels.is_empty() {
            yaml.push_str("channels: {}\n\n");
        } else {
            yaml.push_str("channels:\n");
            for ch in &selection.channels {
                let env_name = match ch.as_str() {
                    "telegram" => "TELEGRAM_BOT_TOKEN",
                    "discord" => "DISCORD_BOT_TOKEN",
                    "slack" => "SLACK_BOT_TOKEN",
                    _ => "CHANNEL_TOKEN",
                };
                yaml.push_str(&format!(
                    "  {ch}:\n    type: {ch}\n    token: ${env_name}\n"
                ));
            }
            yaml.push('\n');
        }

        yaml.push_str("tools:\n");
        for tool in &selection.tools {
            yaml.push_str(&format!("  - {tool}\n"));
        }

        if let Some(name) = &selection.provider {
            yaml.push_str("\nproviders:\n");
            yaml.push_str(&format!(
                "  {name}:\n    type: {}\n    api_key: ${}\n",
                provider_type_for_name(name),
                env_var_for_provider(name).unwrap_or("OPENAI_API_KEY")
            ));
            yaml.push_str(&format!("\nprovider: {name}\n"));
        }
        if let Some(model) = &selection.model {
            yaml.push_str(&format!("model: {model}\n"));
        }
    }

    yaml
}

fn parse_template(template: &str) -> Result<TemplateKind> {
    match template {
        "telegram-bot" => Ok(TemplateKind::TelegramBot),
        "discord-bot" => Ok(TemplateKind::DiscordBot),
        "api-only" => Ok(TemplateKind::ApiOnly),
        "multi-runtime" => Ok(TemplateKind::MultiRuntime),
        _ => anyhow::bail!("unknown template: {template}"),
    }
}

fn render_template(template: TemplateKind, runtime: &str) -> String {
    match template {
        TemplateKind::TelegramBot => format!(
            "runtime: {runtime}\n\
             channels:\n\
               telegram:\n\
                 type: telegram\n\
                 token: $TELEGRAM_BOT_TOKEN\n\
             provider: openai\n\
             model: gpt-4o-mini\n\
             tools: [git, http]\n"
        ),
        TemplateKind::DiscordBot => format!(
            "runtime: {runtime}\n\
             channels:\n\
               discord:\n\
                 type: discord\n\
                 token: $DISCORD_BOT_TOKEN\n\
             provider: openai\n\
             model: gpt-4o-mini\n\
             tools: [git, http]\n"
        ),
        TemplateKind::ApiOnly => format!(
            "runtime: {runtime}\n\
             channels: {{}}\n\
             provider: openai\n\
             model: gpt-4o-mini\n\
             tools: [git, http]\n"
        ),
        TemplateKind::MultiRuntime => {
            "channels: {}\nproviders:\n  openai:\n    type: openai\n    api_key: $OPENAI_API_KEY\nruntimes:\n  - name: zeroclaw\n    channels: []\n    tools: [git, http]\n    provider: openai\n    model: gpt-4o-mini\n  - name: nanoclaw\n    channels: []\n    tools: [git, http]\n    provider: openai\n    model: gpt-4o-mini\n".to_string()
        }
    }
}

fn ensure_env_file(
    env_path: &std::path::Path,
    template: Option<TemplateKind>,
    selection: &WizardSelection,
) -> Result<()> {
    let mut required = BTreeSet::new();
    for channel in &selection.channels {
        match channel.as_str() {
            "telegram" => {
                required.insert("TELEGRAM_BOT_TOKEN".to_string());
            }
            "discord" => {
                required.insert("DISCORD_BOT_TOKEN".to_string());
            }
            "slack" => {
                required.insert("SLACK_BOT_TOKEN".to_string());
            }
            _ => {}
        }
    }
    if let Some(provider) = &selection.provider {
        if let Some(name) = env_var_for_provider(provider) {
            required.insert(name.to_string());
        }
    }
    if matches!(template, Some(TemplateKind::TelegramBot)) {
        required.insert("TELEGRAM_BOT_TOKEN".to_string());
        required.insert("OPENAI_API_KEY".to_string());
    }
    if matches!(template, Some(TemplateKind::DiscordBot)) {
        required.insert("DISCORD_BOT_TOKEN".to_string());
        required.insert("OPENAI_API_KEY".to_string());
    }
    if matches!(
        template,
        Some(TemplateKind::ApiOnly | TemplateKind::MultiRuntime)
    ) {
        required.insert("OPENAI_API_KEY".to_string());
    }

    let mut content = if env_path.exists() {
        std::fs::read_to_string(env_path)?
    } else {
        String::from("# ClawDen environment variables\n")
    };

    for key in required {
        if !content.contains(&format!("{key}=")) {
            content.push_str(&format!("{key}=\n"));
        }
    }

    std::fs::write(env_path, content)?;
    println!("Created {}", env_path.display());
    Ok(())
}

fn provider_type_for_name(name: &str) -> &str {
    if name == "local-llm" {
        "openai"
    } else {
        name
    }
}

fn env_var_for_provider(provider: &str) -> Option<&'static str> {
    match provider {
        "openai" => Some("OPENAI_API_KEY"),
        "anthropic" => Some("ANTHROPIC_API_KEY"),
        "google" => Some("GEMINI_API_KEY"),
        "openrouter" => Some("OPENROUTER_API_KEY"),
        _ => None,
    }
}

/// Print detected provider API keys and return the index of the first detected
/// provider to use as the default selection (falls back to 0 / openai).
fn print_detected_provider_envs() -> usize {
    // Order matches the provider prompt options:
    // 0=openai, 1=anthropic, 2=google, 3=openrouter
    let checks = [
        (
            "OPENAI_API_KEY",
            std::env::var("OPENAI_API_KEY").is_ok(),
            0usize,
        ),
        (
            "ANTHROPIC_API_KEY",
            std::env::var("ANTHROPIC_API_KEY").is_ok(),
            1,
        ),
        (
            "OPENROUTER_API_KEY",
            std::env::var("OPENROUTER_API_KEY").is_ok(),
            3,
        ),
        (
            "GEMINI_API_KEY / GOOGLE_API_KEY",
            std::env::var("GEMINI_API_KEY").is_ok() || std::env::var("GOOGLE_API_KEY").is_ok(),
            2,
        ),
    ];
    println!("Detected API keys in environment:");
    let mut first_detected: Option<usize> = None;
    for (name, present, idx) in &checks {
        let marker = if *present { "x" } else { " " };
        println!("  [{marker}] {name}");
        if *present && first_detected.is_none() {
            first_detected = Some(*idx);
        }
    }
    if let Some(idx) = first_detected {
        println!("  (auto-selected based on detected key)");
        idx
    } else {
        0
    }
}

/// Detect channel tokens already set in the environment.
fn detect_channel_envs() -> Vec<String> {
    let mut channels = Vec::new();
    if std::env::var("TELEGRAM_BOT_TOKEN").is_ok() {
        channels.push("telegram".to_string());
    }
    if std::env::var("DISCORD_BOT_TOKEN").is_ok() {
        channels.push("discord".to_string());
    }
    if std::env::var("SLACK_BOT_TOKEN").is_ok() {
        channels.push("slack".to_string());
    }
    channels
}
