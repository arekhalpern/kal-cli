use clap::{Args, Subcommand};

use crate::{client::KalshiClient, output::print_value, AppContext};

#[derive(Debug, Clone, Args)]
pub struct ExchangeCmd {
    #[command(subcommand)]
    command: ExchangeSubcmd,
}

#[derive(Debug, Clone, Subcommand)]
enum ExchangeSubcmd {
    Status,
    Schedule,
    Announcements,
}

pub async fn run(ctx: &AppContext, cmd: ExchangeCmd) -> anyhow::Result<()> {
    let client = KalshiClient::new(ctx.runtime.clone())?;

    let result = match cmd.command {
        ExchangeSubcmd::Status => client.get_public("/exchange/status", None).await?,
        ExchangeSubcmd::Schedule => client.get_public("/exchange/schedule", None).await?,
        ExchangeSubcmd::Announcements => client.get_public("/exchange/announcements", None).await?,
    };

    print_value(ctx.output_mode, &result)
}
