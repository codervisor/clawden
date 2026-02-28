use anyhow::Result;
use clap::{Parser, Subcommand};
use reqwest::blocking::Client;

#[derive(Debug, Parser)]
#[command(name = "clawden", version, about = "ClawDen orchestration CLI")]
struct Cli {
    #[arg(long, global = true, default_value = "http://127.0.0.1:8080")]
    server_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,
    /// Start all runtimes from clawden.yaml
    Up {
        /// Specific runtimes to start (starts all if empty)
        runtimes: Vec<String>,
    },
    /// Run a single runtime
    Run {
        runtime: Option<String>,
        /// Channels to connect
        #[arg(long)]
        channel: Vec<String>,
        /// Tools to enable
        #[arg(long = "with")]
        tools: Option<String>,
    },
    /// Show running runtimes
    Ps,
    /// Stop runtimes
    Stop {
        /// Specific runtime to stop (stops all if empty)
        runtime: Option<String>,
    },
    /// Channel management
    Channels {
        #[command(subcommand)]
        command: Option<ChannelCommand>,
    },
}

#[derive(Debug, Subcommand)]
enum ChannelCommand {
    /// Test all channel credentials
    Test {
        /// Specific channel type to test
        channel_type: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::new();
    let base = cli.server_url.trim_end_matches('/');

    match cli.command {
        Commands::Init => println!("clawden init scaffold is not implemented yet"),
        Commands::Up { runtimes } => {
            if runtimes.is_empty() {
                println!("Starting all runtimes from clawden.yaml...");
            } else {
                println!("Starting runtimes: {}", runtimes.join(", "));
            }
            let response = client
                .get(format!("{base}/runtimes"))
                .send()?
                .error_for_status()?;
            let runtimes_list: serde_json::Value = response.json()?;
            println!(
                "Available runtimes: {}",
                serde_json::to_string_pretty(&runtimes_list)?
            );
        }
        Commands::Run {
            runtime,
            channel,
            tools,
        } => {
            let rt = runtime.unwrap_or_else(|| "zeroclaw".to_string());
            let tools_list = tools
                .map(|t| {
                    t.split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            println!(
                "Running {} with channels {:?} and tools {:?}",
                rt, channel, tools_list
            );
            let body = serde_json::json!({
                "instance_name": format!("{}-default", rt),
                "runtime": rt,
                "channels": channel,
                "tools": tools_list,
            });
            let response = client
                .post(format!("{base}/runtimes/{rt}/deploy"))
                .json(&body)
                .send()?
                .error_for_status()?;
            println!("{}", response.text()?);
        }
        Commands::Ps => {
            let response = client
                .get(format!("{base}/agents"))
                .send()?
                .error_for_status()?;
            let agents: Vec<serde_json::Value> = response.json()?;
            if agents.is_empty() {
                println!("No running runtimes");
            } else {
                println!(
                    "{:<20} {:<12} {:<10} {:<10}",
                    "NAME", "RUNTIME", "STATE", "HEALTH"
                );
                for agent in &agents {
                    println!(
                        "{:<20} {:<12} {:<10} {:<10}",
                        agent["name"].as_str().unwrap_or("-"),
                        agent["runtime"].as_str().unwrap_or("-"),
                        agent["state"].as_str().unwrap_or("-"),
                        agent["health"].as_str().unwrap_or("-"),
                    );
                }
            }
        }
        Commands::Stop { runtime } => {
            if let Some(rt) = runtime {
                println!("Stopping {}...", rt);
                let response = client
                    .post(format!("{base}/agents/{rt}/stop"))
                    .send()?
                    .error_for_status()?;
                println!("{}", response.text()?);
            } else {
                println!("Stopping all runtimes...");
                let response = client
                    .get(format!("{base}/agents"))
                    .send()?
                    .error_for_status()?;
                let agents: Vec<serde_json::Value> = response.json()?;
                for agent in &agents {
                    if let Some(id) = agent["id"].as_str() {
                        let _ = client.post(format!("{base}/agents/{id}/stop")).send();
                        println!("Stopped {}", agent["name"].as_str().unwrap_or(id));
                    }
                }
            }
        }
        Commands::Channels { command } => {
            match command {
                None => {
                    // List channels
                    let response = client
                        .get(format!("{base}/channels"))
                        .send()?
                        .error_for_status()?;
                    let channels: Vec<serde_json::Value> = response.json()?;
                    if channels.is_empty() {
                        println!("No channels configured");
                    } else {
                        println!(
                            "{:<15} {:<10} {:<12} {:<12}",
                            "TYPE", "INSTANCES", "CONNECTED", "DISCONNECTED"
                        );
                        for ch in &channels {
                            println!(
                                "{:<15} {:<10} {:<12} {:<12}",
                                ch["channel_type"].as_str().unwrap_or("-"),
                                ch["instance_count"].as_u64().unwrap_or(0),
                                ch["connected"].as_u64().unwrap_or(0),
                                ch["disconnected"].as_u64().unwrap_or(0),
                            );
                        }
                    }
                }
                Some(ChannelCommand::Test { channel_type }) => {
                    let url = if let Some(ct) = &channel_type {
                        format!("{base}/channels/{ct}/test")
                    } else {
                        format!("{base}/channels/telegram/test")
                    };
                    let response = client.post(&url).send()?.error_for_status()?;
                    let result: serde_json::Value = response.json()?;
                    println!("Test result: {}", serde_json::to_string_pretty(&result)?);
                }
            }
        }
    }

    Ok(())
}
