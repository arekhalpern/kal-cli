use crate::{auth, config::RuntimeConfig};
use reqwest::{Method, StatusCode};
use serde_json::Value;
use std::{collections::BTreeMap, time::Duration};

#[derive(Clone)]
pub struct KalshiClient {
    http: reqwest::Client,
    runtime: RuntimeConfig,
}

impl KalshiClient {
    pub fn new(runtime: RuntimeConfig) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        Ok(Self { http, runtime })
    }

    pub async fn get_public(
        &self,
        path: &str,
        query: Option<BTreeMap<String, String>>,
    ) -> anyhow::Result<Value> {
        let use_auth = self.runtime.api_key.is_some() && self.runtime.api_secret.is_some();
        self.request(Method::GET, path, query, None, use_auth).await
    }

    pub async fn get_auth(
        &self,
        path: &str,
        query: Option<BTreeMap<String, String>>,
    ) -> anyhow::Result<Value> {
        self.request(Method::GET, path, query, None, true).await
    }

    pub async fn post_auth(&self, path: &str, body: Option<Value>) -> anyhow::Result<Value> {
        self.request(Method::POST, path, None, body, true).await
    }

    pub async fn delete_auth(&self, path: &str, body: Option<Value>) -> anyhow::Result<Value> {
        self.request(Method::DELETE, path, None, body, true).await
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        query: Option<BTreeMap<String, String>>,
        body: Option<Value>,
        auth_required: bool,
    ) -> anyhow::Result<Value> {
        let base_url = self.runtime.rest_base_url();
        let url = format!("{}{}", base_url, path);

        let mut attempt = 0;
        let max_retries = 3;

        loop {
            let mut req = self.http.request(method.clone(), &url);

            if let Some(q) = &query {
                req = req.query(q);
            }

            if let Some(b) = &body {
                req = req.json(b);
            }

            if auth_required {
                let api_key = self
                    .runtime
                    .api_key
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("missing api key"))?;
                let api_secret = self
                    .runtime
                    .api_secret
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("missing api secret"))?;

                let signed_path = signed_path(path, &query);
                for (name, value) in auth::get_auth_headers(
                    api_key,
                    &auth::parse_private_key(api_secret),
                    method.as_str(),
                    &format!("/trade-api/v2{}", signed_path),
                )? {
                    req = req.header(name, value);
                }
            }

            let response = req.send().await?;
            if response.status().is_success() {
                if response.status() == StatusCode::NO_CONTENT {
                    return Ok(serde_json::json!({}));
                }
                return Ok(response.json::<Value>().await?);
            }

            if matches!(response.status(), StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE)
                && attempt < max_retries
            {
                attempt += 1;
                let retry_after_ms = response
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|secs| secs.saturating_mul(1000));
                let exponential_ms = (1000_u64 * (1_u64 << (attempt - 1))).min(30_000);
                let jitter_seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos() as u64)
                    .unwrap_or(500_000_000);
                let jitter_factor = 0.5 + (jitter_seed % 1000) as f64 / 1000.0; // 0.5..1.499
                let jitter_ms = (exponential_ms as f64 * jitter_factor).round() as u64;
                let backoff_ms = retry_after_ms.unwrap_or(jitter_ms.min(30_000));
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                continue;
            }

            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| String::from("<empty>"));
            anyhow::bail!("kalshi api error {}: {}", status, text);
        }
    }
}

fn signed_path(path: &str, query: &Option<BTreeMap<String, String>>) -> String {
    let mut out = path.to_string();
    if let Some(q) = query {
        if !q.is_empty() {
            out.push('?');
            out.push_str(
                &q.iter()
                    .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                    .collect::<Vec<_>>()
                    .join("&"),
            );
        }
    }
    out
}
