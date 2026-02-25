use std::{collections::BTreeMap, time::{SystemTime, UNIX_EPOCH}};

use clap::{Args, Subcommand};
use serde_json::Value;

use crate::{
    client::KalshiClient,
    config::ensure_auth,
    output::{print_rows, print_value, render_positions_table},
    AppContext,
};

#[derive(Debug, Clone, Args)]
pub struct PortfolioCmd {
    #[command(subcommand)]
    command: PortfolioSubcmd,
}

#[derive(Debug, Clone, Subcommand)]
enum PortfolioSubcmd {
    Balance,
    Positions {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long = "event")]
        event_ticker: Option<String>,
        #[arg(long, default_value_t = false)]
        settled: bool,
        #[arg(long, default_value_t = false)]
        unsettled: bool,
        #[arg(long, default_value_t = false)]
        compact: bool,
    },
    Fills {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long, default_value_t = 7)]
        days: u64,
    },
    Settlements {
        #[arg(long)]
        ticker: Option<String>,
        #[arg(long, default_value_t = 30)]
        days: u64,
    },
}

pub async fn run(ctx: &AppContext, cmd: PortfolioCmd) -> anyhow::Result<()> {
    ensure_auth(&ctx.runtime)?;
    let client = KalshiClient::new(ctx.runtime.clone())?;

    match cmd.command {
        PortfolioSubcmd::Balance => {
            let data = client.get_auth("/portfolio/balance", None).await?;
            print_value(ctx.output_mode, &data)
        }
        PortfolioSubcmd::Positions {
            ticker,
            event_ticker,
            settled,
            unsettled,
            compact,
        } => {
            let mut q = BTreeMap::new();
            q.insert("limit".to_string(), "200".to_string());
            if let Some(t) = ticker {
                q.insert("ticker".to_string(), t);
            }
            if let Some(e) = event_ticker {
                q.insert("event_ticker".to_string(), e);
            }
            if settled {
                q.insert("settlement_status".to_string(), "settled".to_string());
            } else if unsettled {
                q.insert("settlement_status".to_string(), "unsettled".to_string());
            }

            let data = client.get_auth("/portfolio/positions", Some(q)).await?;
            let rows = data
                .get("market_positions")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            render_positions_table(ctx.output_mode, &rows, compact)
        }
        PortfolioSubcmd::Fills { ticker, days } => {
            let mut q = BTreeMap::new();
            q.insert("limit".to_string(), "200".to_string());
            q.insert("min_ts".to_string(), ts_days_ago(days).to_string());
            if let Some(t) = ticker {
                q.insert("ticker".to_string(), t);
            }

            let data = client.get_auth("/portfolio/fills", Some(q)).await?;
            let rows = data
                .get("fills")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            print_rows(
                ctx.output_mode,
                &rows,
                &["trade_id", "ticker", "side", "action", "count", "yes_price", "created_time"],
            )
        }
        PortfolioSubcmd::Settlements { ticker, days } => {
            let mut q = BTreeMap::new();
            q.insert("limit".to_string(), "200".to_string());
            q.insert("min_ts".to_string(), ts_days_ago(days).to_string());
            if let Some(t) = ticker {
                q.insert("ticker".to_string(), t);
            }

            let data = client.get_auth("/portfolio/settlements", Some(q)).await?;
            let rows = data
                .get("settlements")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            print_rows(
                ctx.output_mode,
                &rows,
                &["ticker", "market_result", "realized_pnl", "yes_count", "no_count", "settlement_ts"],
            )
        }
    }
}

fn ts_days_ago(days: u64) -> i64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    now - (days as i64 * 86_400)
}
