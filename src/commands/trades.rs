use clap::{Args, Subcommand};

use crate::{
    client::KalshiClient,
    output::{extract_array, print_rows},
    query::QueryParams,
    AppContext,
};

#[derive(Debug, Clone, Args)]
pub struct TradesCmd {
    #[command(subcommand)]
    command: TradesSubcmd,
}

#[derive(Debug, Clone, Subcommand)]
enum TradesSubcmd {
    List {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

pub async fn run(ctx: &AppContext, cmd: TradesCmd) -> anyhow::Result<()> {
    let client = KalshiClient::new(ctx.runtime.clone())?;

    match cmd.command {
        TradesSubcmd::List { ticker, limit } => {
            let q = QueryParams::new()
                .limit(limit)
                .optional("ticker", ticker)
                .build();
            let data = client.get_public("/markets/trades", q).await?;
            let rows = extract_array(&data, "trades");
            print_rows(
                ctx.output_mode,
                &rows,
                &["trade_id", "ticker", "count", "yes_price", "created_time"],
            )
        }
    }
}
