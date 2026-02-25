mod auth;
mod client;
mod commands;
mod config;
mod output;
mod query;

use clap::{Args, Parser, Subcommand, ValueEnum};
use commands::{
    config_cmd, events, exchange, markets, order, portfolio, shell, trades, watch,
};
use config::{resolve_runtime_config, Environment, RuntimeConfig};
use output::OutputMode;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Clone, Args)]
struct GlobalOpts {
    #[arg(short = 'o', long = "output", value_enum, default_value = "table", global = true)]
    output: OutputFormat,

    #[arg(short = 'e', long = "env", value_enum, global = true)]
    environment: Option<Environment>,

    #[arg(long = "api-key", global = true)]
    api_key: Option<String>,

    #[arg(long = "api-secret", global = true)]
    api_secret: Option<String>,
}

#[derive(Debug, Parser)]
#[command(name = "kal", version, about = "Kalshi CLI")]
struct Cli {
    #[command(flatten)]
    global: GlobalOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Markets(markets::MarketsCmd),
    Events(events::EventsCmd),
    Order(order::OrderCmd),
    Portfolio(portfolio::PortfolioCmd),
    Trades(trades::TradesCmd),
    Exchange(exchange::ExchangeCmd),
    Watch(watch::WatchCmd),
    Config(config_cmd::ConfigCmd),
    Shell,
}

#[derive(Clone)]
pub struct AppContext {
    pub runtime: RuntimeConfig,
    pub output_mode: OutputMode,
}

impl From<OutputFormat> for OutputMode {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Table => OutputMode::Table,
            OutputFormat::Json => OutputMode::Json,
        }
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    dispatch(cli).await
}

pub(crate) async fn dispatch(cli: Cli) -> anyhow::Result<()> {
    let output_mode = OutputMode::from(cli.global.output);

    if let Commands::Config(cmd) = &cli.command {
        return config_cmd::run(cmd.clone(), output_mode).await;
    }

    let runtime = resolve_runtime_config(
        cli.global.environment,
        cli.global.api_key,
        cli.global.api_secret,
    )?;

    let ctx = AppContext {
        runtime,
        output_mode,
    };

    match cli.command {
        Commands::Markets(cmd) => markets::run(&ctx, cmd).await,
        Commands::Events(cmd) => events::run(&ctx, cmd).await,
        Commands::Order(cmd) => order::run(&ctx, cmd).await,
        Commands::Portfolio(cmd) => portfolio::run(&ctx, cmd).await,
        Commands::Trades(cmd) => trades::run(&ctx, cmd).await,
        Commands::Exchange(cmd) => exchange::run(&ctx, cmd).await,
        Commands::Watch(cmd) => watch::run(&ctx, cmd).await,
        Commands::Shell => shell::run(output_mode).await,
        Commands::Config(_) => unreachable!(),
    }
}
