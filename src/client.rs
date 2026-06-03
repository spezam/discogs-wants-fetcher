use crate::wants::{Want, Wants};
use core::time::Duration;
use tokio::time::sleep;
use url::Url;

const API_BASE_URL: &str = "https://api.discogs.com";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub struct DiscogsClient {
    reqwest_client: reqwest::Client,
    base_url: String,
}

impl DiscogsClient {
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(6))
            .timeout(Duration::from_secs(6))
            .user_agent("discogs-wants-fetcher/0.1.0 +github.com/spezam/discogs-wants-fetcher")
            .build()
            .expect("Cannot initialize HTTP client");

        DiscogsClient {
            reqwest_client: client,
            base_url: API_BASE_URL.to_string(),
        }
    }

    pub async fn get_wants_raw(&self, username: &str) -> Result<Vec<Want>, Error> {
        let mut wants: Vec<Want> = Vec::new();
        let mut url = Url::parse_with_params(
            &format!("{}/users/{}/wants", self.base_url, username),
            &[
                ("per_page", "100"),
                ("sort", "added"),
                ("sort_order", "desc"),
            ],
        )?
        .to_string();

        loop {
            let response = self.reqwest_client.get(&url).send().await?;

            let remaining_ratelimit = response
                .headers()
                .get("x-discogs-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("0")
                .parse::<i32>()
                .unwrap_or(0);

            let mut response_wants = response.error_for_status()?.json::<Wants>().await?;
            wants.append(&mut response_wants.wants);

            if remaining_ratelimit <= 2 {
                eprintln!("Rate limit low ({remaining_ratelimit} remaining), backing off for 10s");
                sleep(Duration::from_secs(10)).await;
            }

            match response_wants.pagination.urls.next {
                Some(next) => url = next,
                None => break,
            }
        }

        Ok(wants)
    }
}

impl Default for DiscogsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl DiscogsClient {
    pub fn new_with_base_url(base_url: &str) -> Self {
        let client = reqwest::ClientBuilder::new()
            .build()
            .expect("Cannot initialize HTTP client");
        DiscogsClient {
            reqwest_client: client,
            base_url: base_url.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_page_body(server_url: &str) -> String {
        format!(
            r#"{{
                "pagination": {{
                    "page": 1, "pages": 1, "per_page": 100, "items": 1,
                    "urls": {{ "last": null, "next": null }}
                }},
                "wants": [{{
                    "id": 42,
                    "resource_url": "{server_url}/users/testuser/wants/42",
                    "date_added": "2024-01-01T00:00:00-08:00",
                    "rating": 0,
                    "basic_information": {{
                        "id": 1, "master_id": 0, "master_url": null,
                        "resource_url": "{server_url}/releases/1",
                        "title": "Kind of Blue", "year": 1959,
                        "formats": [],
                        "artists": [{{
                            "name": "Miles Davis", "anv": "", "join": "",
                            "role": "", "tracks": "", "id": 99,
                            "resource_url": "{server_url}/artists/99"
                        }}],
                        "labels": [], "thumb": "", "cover_image": "",
                        "genres": ["Jazz"], "styles": ["Modal"]
                    }}
                }}]
            }}"#
        )
    }

    #[tokio::test]
    async fn test_get_wants_single_page() {
        let mut server = mockito::Server::new_async().await;
        let body = single_page_body(&server.url());

        let _mock = server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"/users/testuser/wants.*".to_string()),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("x-discogs-ratelimit-remaining", "60")
            .with_body(&body)
            .create_async()
            .await;

        let client = DiscogsClient::new_with_base_url(&server.url());
        let wants = client.get_wants_raw("testuser").await.unwrap();

        assert_eq!(wants.len(), 1);
        assert_eq!(wants[0].id, 42);
        assert_eq!(wants[0].basic_information.title, "Kind of Blue");
        assert_eq!(wants[0].basic_information.year, 1959);
        assert_eq!(wants[0].basic_information.artists[0].name, "Miles Davis");
    }

    #[tokio::test]
    async fn test_get_wants_http_error() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"/users/noone/wants.*".to_string()),
            )
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"message": "User not found"}"#)
            .create_async()
            .await;

        let client = DiscogsClient::new_with_base_url(&server.url());
        let result = client.get_wants_raw("noone").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Http(_)));
    }
}
