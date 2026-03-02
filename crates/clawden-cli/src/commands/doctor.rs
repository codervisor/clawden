use anyhow::Result;
use clawden_config::ClawDenYaml;
use clawden_core::{ProcessManager, RuntimeInstaller};
use reqwest::blocking::Client;
use std::time::Duration;

use crate::util::{command_exists, get_provider_key_from_vault};

pub fn exec_doctor(installer: &RuntimeInstaller) -> Result<()> {
    println!("Prerequisites");
    println!("  docker ............... {}", yes_no(ProcessManager::docker_available()));
    println!("  node ................. {}", yes_no(command_exists("node")));
    println!("  npm .................. {}", yes_no(command_exists("npm")));
    println!("  git .................. {}", yes_no(command_exists("git")));
    println!(
        "  curl/wget ............ {}",
        yes_no(command_exists("curl") || command_exists("wget"))
    );
    println!("  clawden_home ......... {}", installer.root_dir().display());

    let yaml_path = std::env::current_dir()?.join("clawden.yaml");
    if yaml_path.exists() {
        println!("\nConfiguration ({})", yaml_path.display());
        let mut config = ClawDenYaml::from_file(&yaml_path).map_err(anyhow::Error::msg)?;
        match config.validate() {
            Ok(()) => println!("  schema ............... ok"),
            Err(errs) => {
                println!("  schema ............... fail");
                for err in errs {
                    println!("    - {err}");
                }
            }
        }

        match config.resolve_env_vars() {
            Ok(()) => println!("  env resolution ....... ok"),
            Err(errs) => {
                println!("  env resolution ....... fail");
                for err in errs {
                    println!("    - {err}");
                }
            }
        }

        if config.providers.is_empty() {
            println!("  providers ............ none configured");
        } else {
            for (name, provider) in &config.providers {
                let provider_key = provider
                    .api_key
                    .clone()
                    .or(get_provider_key_from_vault(name)?);
                let key_state = if provider_key.is_some() {
                    "ok"
                } else {
                    "missing api_key"
                };
                println!("  provider.{name} ....... {key_state}");

                if let Some(api_key) = provider_key {
                    let base_url = provider
                        .base_url
                        .clone()
                        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
                    match probe_provider(name, &base_url, &api_key) {
                        Ok(()) => println!("    credential_probe ..... ok"),
                        Err(err) => println!("    credential_probe ..... fail ({err})"),
                    }
                }
            }
        }
    } else {
        println!("\nConfiguration\n  clawden.yaml .......... missing");
    }

    println!("\nRuntimes");
    let installed = installer.list_installed()?;
    if installed.is_empty() {
        println!("  installed ............ none");
    }
    for row in installed {
        println!("  {} ............. {}", row.runtime, row.version);
    }
    Ok(())
}

fn probe_provider(provider: &str, base_url: &str, api_key: &str) -> Result<()> {
    let endpoint = if provider == "anthropic" {
        format!("{}/v1/models", base_url.trim_end_matches('/'))
    } else {
        format!("{}/models", base_url.trim_end_matches('/'))
    };

    let client = Client::builder().timeout(Duration::from_secs(6)).build()?;
    let mut request = client.get(endpoint);
    match provider {
        "anthropic" => {
            request = request
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01");
        }
        _ => {
            request = request.bearer_auth(api_key);
        }
    }

    let response = request.send()?;
    if response.status().is_success() {
        Ok(())
    } else {
        anyhow::bail!("http_status={}", response.status());
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "ok"
    } else {
        "missing"
    }
}
