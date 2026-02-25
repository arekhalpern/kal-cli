use std::collections::BTreeMap;

use clap::{Args, Subcommand};
use serde_json::Value;

use crate::{client::KalshiClient, output::print_rows, AppContext};

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
            let mut q = BTreeMap::new();
            q.insert("limit".to_string(), limit.to_string());
            if let Some(t) = ticker {
                q.insert("ticker".to_string(), t);
            }
            let data = client.get_public("/markets/trades", Some(q)).await?;
            let rows = data
                .get("trades")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            print_rows(
                ctx.output_mode,
                &rows,
                &["trade_id", "ticker", "count", "yes_price", "created_time"],
            )
        }
    }
}
