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
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(2))
            .user_agent(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/115.0",
            )
            .build()
            .expect("Cannot initialize HTTP client");

        DiscogsClient {
            reqwest_client: client,
        }
    }

    pub async fn get_wants_raw(&self, username: &String) -> Result<Vec<Want>, reqwest::Error> {
        let url = Url::parse_with_params(
            format!("{}/users/{}/wants", API_BASE_URL, username).as_str(),
            &[("per_page", "100"), ("per_page", &"100".to_string())],
        )
        .unwrap();

        let response: Wants = self
            .reqwest_client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut wants: Vec<Want> = response.wants;

        let pages = response.pagination.pages;
        for page in 1..pages {
            sleep(Duration::from_millis(1000)).await;

            let url = Url::parse_with_params(
                format!("{}/users/{}/wants", API_BASE_URL, username).as_str(),
                &[
                    ("page", &page.to_string()),
                    ("per_page", &"100".to_string()),
                ],
            )
            .unwrap();
            println!("{}", url);

            let mut response: Wants = self
                .reqwest_client
                .get(url)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            wants.append(&mut response.wants);
        }

        Ok(wants)
    }
}
