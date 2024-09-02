// ==============================================================================================
use anyhow::anyhow;
use reqwest::{header::HeaderMap, Method, StatusCode};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
// ==============================================================================================

pub struct Request;

impl Request {
    /// Processes an HTTP request with the given method, URL, body, and headers.
    /// Retries the request if it fails, up to a maximum number of attempts.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method to use for the request (GET, POST, etc.).
    /// * `url` - The URL to send the request to.
    /// * `body` - An optional JSON body to include in the request (for POST requests).
    /// * `headers` - Optional HTTP headers to include in the request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the JSON response body if the request
    /// is successful, or an `anyhow::Error` if it fails.
    ///
    /// # Errors
    ///
    /// This function returns an error if the input URL cannot be parsed or
    /// if the request method is not supported.
    ///
    /// If the request fails after the maximum number
    /// of attempts, an error is also returned.
    pub async fn process_request<S: AsRef<str>>(
        method: Method,
        url: S,
        headers: Option<HeaderMap>,
        body: Option<Value>,
    ) -> Result<Value, anyhow::Error> {
        let attempts_limit = 15;
        let mut attempt = 1;
        let wait_delay = Duration::from_secs_f64(1.5);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        let url = reqwest::Url::parse(url.as_ref())?;
        let headers = headers.unwrap_or_else(HeaderMap::new);

        while attempt <= attempts_limit {
            let request = match method.clone() {
                Method::GET => client
                    .request(method.clone(), url.clone())
                    .headers(headers.clone()),
                Method::POST => client
                    .request(method.clone(), url.clone())
                    .json(&body)
                    .headers(headers.clone()),
                _ => return Err(anyhow!("The method <{}> is not supported.", method)),
            };

            match request.send().await {
                Ok(res) => match res.status() {
                    StatusCode::OK => {
                        let json: Value = res.json().await?;
                        return Ok(json);
                    }

                    StatusCode::NOT_FOUND | StatusCode::TOO_MANY_REQUESTS => {
                        error!("{:?}", res.text().await?);
                        sleep(wait_delay).await;
                        attempt += 1;
                        continue;
                    }

                    StatusCode::GATEWAY_TIMEOUT => {
                        return Err(anyhow!(
                            "🚨 URL: {} Status: {} | Can't process request.",
                            res.url().to_string(),
                            res.status()
                        ))
                    }

                    _ => {
                        warn!(
                            "Critical response error. URL: {} Status: {} | {:#?}",
                            res.url().to_string(),
                            res.status(),
                            res.text().await?
                        );

                        sleep(wait_delay).await;
                        attempt += 1;
                        continue;
                    }
                },
                Err(_) => {
                    attempt += 1;
                    continue;
                }
            }
        }

        return Err(anyhow!("🚨 Attempts reached. Check URL: {}", url.as_str()));
    }
}
