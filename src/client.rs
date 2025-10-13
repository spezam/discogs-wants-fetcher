use crate::wants::{Want, Wants};
use core::time::Duration;
use reqwest::Url;
use tokio::time::sleep;

pub struct DiscogsClient {
    reqwest_client: reqwest::Client,
}

const API_BASE_URL: &str = "https://api.discogs.com";

impl DiscogsClient {
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(6))
            .timeout(Duration::from_secs(6))
            .user_agent("WantsFetcher/1.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/115.0")
            .build()
            .expect("Cannot initialize HTTP client");

        DiscogsClient {
            reqwest_client: client,
        }
    }

    pub async fn get_wants_raw(&self, username: &String) -> Result<Vec<Want>, reqwest::Error> {
        let mut wants: Vec<Want> = Vec::new();
        let mut url = Url::parse_with_params(
            format!("{}/users/{}/wants", API_BASE_URL, username).as_str(),
            &[
                ("per_page", "100"),
                ("sort", "added"),
                ("sort_order", "desc"),
            ],
        )
        .unwrap()
        .to_string();

        loop {
            let response = self.reqwest_client.get(url).send().await?;

            // check status
            if !response.status().is_success() {
                println!("Current status: {:?}", response);
            }
            // check x-discogs-ratelimit-remaining
            let remaining_ratelimit = response
                .headers()
                .get("x-discogs-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("0")
                .parse::<i32>()
                .unwrap();
            println!("x-discogs-ratelimit-remaining {:?}", remaining_ratelimit);

            // unmmashal result
            let mut response_wants = response.error_for_status()?.json::<Wants>().await?;
            wants.append(&mut response_wants.wants);

            if remaining_ratelimit <= 2 {
                println!("We will hit ratelimit, slowing down and continue in 10000 ms");
                sleep(Duration::from_millis(10000)).await;
            }

            if let Some(next) = &response_wants.pagination.urls.next {
                println!("next: {}", next);
                url = next.to_string();
            } else {
                break;
            }
        }

        Ok(wants)
    }
}
