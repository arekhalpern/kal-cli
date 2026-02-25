use base64::{engine::general_purpose::STANDARD, Engine};
use openssl::{hash::MessageDigest, pkey::PKey, sign::Signer};

pub fn parse_private_key(key: &str) -> String {
    if key.contains("-----BEGIN") {
        return key.to_string();
    }

    let clean = key.split_whitespace().collect::<String>();
    let mut out = String::from("-----BEGIN PRIVATE KEY-----\n");

    for chunk in clean.as_bytes().chunks(64) {
        out.push_str(std::str::from_utf8(chunk).unwrap_or_default());
        out.push('\n');
    }

    out.push_str("-----END PRIVATE KEY-----");
    out
}

pub fn sign_request(private_key_pem: &str, timestamp: i64, method: &str, path: &str) -> anyhow::Result<String> {
    let path_no_query = path.split('?').next().unwrap_or(path);
    let message = format!("{}{}{}", timestamp, method.to_uppercase(), path_no_query);

    let key = PKey::private_key_from_pem(private_key_pem.as_bytes())?;
    let mut signer = Signer::new(MessageDigest::sha256(), &key)?;
    signer.set_rsa_padding(openssl::rsa::Padding::PKCS1_PSS)?;
    signer.set_rsa_pss_saltlen(openssl::sign::RsaPssSaltlen::DIGEST_LENGTH)?;
    signer.update(message.as_bytes())?;

    let sig = signer.sign_to_vec()?;
    Ok(STANDARD.encode(sig))
}

pub fn get_auth_headers(
    api_key: &str,
    private_key_pem: &str,
    method: &str,
    path: &str,
) -> anyhow::Result<Vec<(&'static str, String)>> {
    let ts = chrono_now_ms();
    let signature = sign_request(private_key_pem, ts, method, path)?;

    Ok(vec![
        ("KALSHI-ACCESS-KEY", api_key.to_string()),
        ("KALSHI-ACCESS-TIMESTAMP", ts.to_string()),
        ("KALSHI-ACCESS-SIGNATURE", signature),
    ])
}

fn chrono_now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_millis() as i64
}
