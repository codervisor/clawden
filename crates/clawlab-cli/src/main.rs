use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "clawlab", version, about = "ClawLab orchestration CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,
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
enum ServerCommand {
    Start,
}

#[derive(Debug, Subcommand)]
enum AgentCommand {
    List,
    Start { name: String },
    Stop { name: String },
    Health,
}

#[derive(Debug, Subcommand)]
enum FleetCommand {
    Status,
}

#[derive(Debug, Subcommand)]
enum TaskCommand {
    Send { agent: String, message: String },
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => println!("clawlab init scaffold is not implemented yet"),
        Commands::Server { command } => match command {
            ServerCommand::Start => println!("server start delegated to clawlab-server binary"),
        },
        Commands::Agent { command } => println!("agent command: {command:?}"),
        Commands::Fleet { command } => println!("fleet command: {command:?}"),
        Commands::Task { command } => println!("task command: {command:?}"),
        Commands::Skill { command } => println!("skill command: {command:?}"),
        Commands::Config { command } => println!("config command: {command:?}"),
    }

    Ok(())
}
