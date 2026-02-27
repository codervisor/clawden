use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
    Server {
        #[command(subcommand)]
        command: ServerCommand,
    },
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    Fleet {
        #[command(subcommand)]
        command: FleetCommand,
    },
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    Skill {
        #[command(subcommand)]
        command: SkillCommand,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
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

#[derive(Debug, Subcommand)]
enum ServerCommand {
    Start,
}

#[derive(Debug, Subcommand)]
enum AgentCommand {
    Register {
        name: String,
        #[arg(value_enum)]
        runtime: RuntimeArg,
        #[arg(long = "capability")]
        capabilities: Vec<String>,
    },
    List,
    Start {
        id: String,
    },
    Stop {
        id: String,
    },
    Health,
}

#[derive(Debug, Subcommand)]
enum FleetCommand {
    Status,
}

#[derive(Debug, Subcommand)]
enum TaskCommand {
    Send {
        message: String,
        #[arg(long)]
        agent_id: Option<String>,
        #[arg(long = "capability")]
        required_capabilities: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum SkillCommand {
    Create { name: String },
    Test { name: String },
    Publish { name: String },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Set { key: String, value: String },
    Diff,
}

#[derive(Debug, Clone, ValueEnum)]
enum RuntimeArg {
    Openclaw,
    Zeroclaw,
    Picoclaw,
    Nanoclaw,
    Ironclaw,
    Nullclaw,
    Microclaw,
    Mimiclaw,
}

impl RuntimeArg {
    fn as_runtime(&self) -> &'static str {
        match self {
            RuntimeArg::Openclaw => "open-claw",
            RuntimeArg::Zeroclaw => "zero-claw",
            RuntimeArg::Picoclaw => "pico-claw",
            RuntimeArg::Nanoclaw => "nano-claw",
            RuntimeArg::Ironclaw => "iron-claw",
            RuntimeArg::Nullclaw => "null-claw",
            RuntimeArg::Microclaw => "micro-claw",
            RuntimeArg::Mimiclaw => "mimi-claw",
        }
    }
}

#[derive(Debug, Serialize)]
struct RegisterAgentRequest {
    name: String,
    runtime: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SendTaskRequest {
    message: String,
    required_capabilities: Vec<String>,
    agent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FleetStatus {
    total_agents: usize,
    running_agents: usize,
    degraded_agents: usize,
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
            println!("Available runtimes: {}", serde_json::to_string_pretty(&runtimes_list)?);
        }
        Commands::Run { runtime, channel, tools } => {
            let rt = runtime.unwrap_or_else(|| "zeroclaw".to_string());
            let tools_list = tools.map(|t| t.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>()).unwrap_or_default();
            println!("Running {} with channels {:?} and tools {:?}", rt, channel, tools_list);
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
                println!("{:<20} {:<12} {:<10} {:<10}", "NAME", "RUNTIME", "STATE", "HEALTH");
                for agent in &agents {
                    println!("{:<20} {:<12} {:<10} {:<10}",
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
                        println!("{:<15} {:<10} {:<12} {:<12}", "TYPE", "INSTANCES", "CONNECTED", "DISCONNECTED");
                        for ch in &channels {
                            println!("{:<15} {:<10} {:<12} {:<12}",
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
                    let response = client
                        .post(&url)
                        .send()?
                        .error_for_status()?;
                    let result: serde_json::Value = response.json()?;
                    println!("Test result: {}", serde_json::to_string_pretty(&result)?);
                }
            }
        }
        Commands::Server { command } => match command {
            ServerCommand::Start => println!("run: cargo run -p clawden-server"),
        },
        Commands::Agent { command } => match command {
            AgentCommand::Register {
                name,
                runtime,
                capabilities,
            } => {
                let body = RegisterAgentRequest {
                    name,
                    runtime: runtime.as_runtime().to_string(),
                    capabilities,
                };
                let response = client
                    .post(format!("{base}/agents/register"))
                    .json(&body)
                    .send()?
                    .error_for_status()?;
                println!("{}", response.text()?);
            }
            AgentCommand::List => {
                let response = client
                    .get(format!("{base}/agents"))
                    .send()?
                    .error_for_status()?;
                println!("{}", response.text()?);
            }
            AgentCommand::Start { id } => {
                let response = client
                    .post(format!("{base}/agents/{id}/start"))
                    .send()?
                    .error_for_status()?;
                println!("{}", response.text()?);
            }
            AgentCommand::Stop { id } => {
                let response = client
                    .post(format!("{base}/agents/{id}/stop"))
                    .send()?
                    .error_for_status()?;
                println!("{}", response.text()?);
            }
            AgentCommand::Health => {
                let response = client
                    .get(format!("{base}/agents/health"))
                    .send()?
                    .error_for_status()?;
                println!("{}", response.text()?);
            }
        },
        Commands::Fleet { command } => match command {
            FleetCommand::Status => {
                let response = client
                    .get(format!("{base}/fleet/status"))
                    .send()?
                    .error_for_status()?;
                let status: FleetStatus = response.json()?;
                println!(
                    "fleet: total={}, running={}, degraded={}",
                    status.total_agents, status.running_agents, status.degraded_agents
                );
            }
        },
        Commands::Task { command } => match command {
            TaskCommand::Send {
                message,
                agent_id,
                required_capabilities,
            } => {
                let body = SendTaskRequest {
                    message,
                    required_capabilities,
                    agent_id,
                };
                let response = client
                    .post(format!("{base}/task/send"))
                    .json(&body)
                    .send()?
                    .error_for_status()?;
                println!("{}", response.text()?);
            }
        },
        Commands::Skill { command } => match command {
            SkillCommand::Create { name } => {
                scaffold_skill_template(&name)?;
                println!("created skill scaffold: {name}");
            }
            SkillCommand::Test { name } => println!("skill test not implemented yet: {name}"),
            SkillCommand::Publish { name } => println!("skill publish not implemented yet: {name}"),
        },
        Commands::Config { command } => println!("config command: {command:?}"),
    }

    Ok(())
}

fn scaffold_skill_template(name: &str) -> Result<()> {
    let skill_dir = Path::new(name);
    if skill_dir.exists() {
        anyhow::bail!("destination already exists: {}", skill_dir.display());
    }

    fs::create_dir_all(skill_dir.join("src"))?;

    let package_json = format!(
        r#"{{
    "name": "@clawden-skill/{name}",
    "version": "0.1.0",
    "private": true,
    "type": "module",
    "scripts": {{
        "build": "tsc -p tsconfig.json"
    }},
    "dependencies": {{
        "@clawden/sdk": "^0.1.0"
    }},
    "devDependencies": {{
        "typescript": "^5.7.3"
    }}
}}
"#
    );

    let tsconfig = r#"{
    "compilerOptions": {
        "target": "ES2022",
        "module": "ESNext",
        "moduleResolution": "Bundler",
        "strict": true,
        "declaration": true,
        "outDir": "dist"
    },
    "include": ["src"]
}
"#;

    let source = format!(
        r#"import {{ defineSkill }} from '@clawden/sdk';

export default defineSkill({{
    name: '{name}',
    version: '0.1.0',
    runtimes: ['openclaw', 'zeroclaw'],
    tools: [],
    async execute(context) {{
        return `echo: ${{context.input}}`;
    }},
}});
"#
    );

    fs::write(skill_dir.join("package.json"), package_json)?;
    fs::write(skill_dir.join("tsconfig.json"), tsconfig)?;
    fs::write(skill_dir.join("src").join("index.ts"), source)?;
    Ok(())
}
