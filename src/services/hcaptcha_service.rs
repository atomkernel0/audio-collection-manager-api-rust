use crate::error::{Error, Result};
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
struct HCaptchaResponse {
    success: bool,
}

pub async fn verify_hcaptcha(response: &str) -> Result<bool> {
    let secret = env::var("HCAPTCHA_SECRET")
        .map_err(|_| Error::EnvVarError("HCAPTCHA_SECRET not found".to_string()))?;

    let client = reqwest::Client::new();
    let res = client
        .post("https://hcaptcha.com/siteverify")
        .form(&[("secret", &secret), ("response", &response.to_string())])
        .send()
        .await
        .map_err(|_| Error::DbError("Failed to send hCaptcha verification request".to_string()))?;

    if res.status().is_success() {
        let captcha_response: HCaptchaResponse = res
            .json()
            .await
            .map_err(|_| Error::DbError("Failed to parse hCaptcha response".to_string()))?;
        Ok(captcha_response.success)
    } else {
        Ok(false)
    }
}
