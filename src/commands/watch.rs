use clap::{Args, Subcommand};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::http::Request};

use crate::{auth, config::ensure_auth, output::print_ndjson, output::OutputMode, AppContext};

#[derive(Debug, Clone, Args)]
pub struct WatchCmd {
    #[command(subcommand)]
    command: WatchSubcmd,
}

#[derive(Debug, Clone, Subcommand)]
enum WatchSubcmd {
    Ticker {
        ticker: String,
        #[arg(long)]
        tickers: Option<String>,
    },
    Orderbook {
        ticker: String,
    },
    Trades {
        ticker: String,
    },
}

pub async fn run(ctx: &AppContext, cmd: WatchCmd) -> anyhow::Result<()> {
    ensure_auth(&ctx.runtime)?;

    let api_key = ctx
        .runtime
        .api_key
        .clone()
        .ok_or_else(|| anyhow::anyhow!("missing api key"))?;
    let api_secret = ctx
        .runtime
        .api_secret
        .clone()
        .ok_or_else(|| anyhow::anyhow!("missing api secret"))?;
    let ws_url = ctx.runtime.ws_url().to_string();

    let headers = auth::get_auth_headers(
        &api_key,
        &auth::parse_private_key(&api_secret),
        "GET",
        "/trade-api/ws/v2",
    )?;

    let mut req_builder = Request::builder().uri(&ws_url);
    for (k, v) in headers {
        req_builder = req_builder.header(k, v);
    }
    let request = req_builder.body(())?;

    let (socket, _) = connect_async(request).await?;
    let (mut writer, mut reader) = socket.split();

    let subscribe_msg = match cmd.command {
        WatchSubcmd::Ticker { ticker, tickers } => {
            let list = tickers
                .map(|v| {
                    v.split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| vec![ticker]);
            json!({"id": 1, "cmd": "subscribe", "params": {"channels": ["ticker"], "market_tickers": list}})
        }
        WatchSubcmd::Orderbook { ticker } => {
            json!({"id": 1, "cmd": "subscribe", "params": {"channels": ["orderbook_delta"], "market_ticker": ticker}})
        }
        WatchSubcmd::Trades { ticker } => {
            json!({"id": 1, "cmd": "subscribe", "params": {"channels": ["trade"], "market_ticker": ticker}})
        }
    };

    writer
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&subscribe_msg)?.into(),
        ))
        .await?;

    while let Some(msg) = reader.next().await {
        let msg = msg?;
        if msg.is_text() {
            let text = msg.into_text()?;
            let parsed: Value = match serde_json::from_str(&text) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("warning: failed to parse websocket message as JSON: {err}");
                    json!({"raw": text})
                }
            };
            match ctx.output_mode {
                OutputMode::Json => print_ndjson(&parsed),
                OutputMode::Table => println!("{}", serde_json::to_string_pretty(&parsed)?),
            }
        }
    }

    Ok(())
}
