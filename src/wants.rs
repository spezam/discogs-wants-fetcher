use serde::Deserialize;
use std::fmt;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct Wants {
    pub pagination: Pagination,
    pub wants: Vec<Want>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub pages: i64,
    pub per_page: i64,
    pub items: i64,
    pub urls: Urls,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct Urls {
    pub last: Option<String>,
    pub next: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct Want {
    pub id: i64,
    pub resource_url: String,
    pub date_added: String,
    pub basic_information: BasicInformation,
    pub rating: i64,
}

impl fmt::Display for Want {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let info = &self.basic_information;
        let artists = info
            .artists
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{} — {} ({})", artists, info.title, info.year)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct BasicInformation {
    pub id: i64,
    pub master_id: i64,
    pub master_url: Option<String>,
    pub resource_url: String,
    pub title: String,
    pub year: i64,
    pub formats: Vec<Format>,
    pub artists: Vec<Artist>,
    pub labels: Vec<Label>,
    pub thumb: String,
    pub cover_image: String,
    pub genres: Vec<String>,
    pub styles: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct Format {
    pub name: String,
    pub qty: String,
    pub descriptions: Vec<String>,
    pub text: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct Artist {
    pub name: String,
    pub anv: String,
    pub join: String,
    pub role: String,
    pub tracks: String,
    pub id: i64,
    pub resource_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct Label {
    pub name: String,
    pub catno: String,
    pub entity_type: String,
    pub entity_type_name: String,
    pub id: i64,
    pub resource_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_pagination() {
        let json = r#"{
            "page": 1,
            "pages": 3,
            "per_page": 100,
            "items": 250,
            "urls": {
                "last": "https://api.discogs.com/users/foo/wants?page=3",
                "next": "https://api.discogs.com/users/foo/wants?page=2"
            }
        }"#;
        let p: Pagination = serde_json::from_str(json).unwrap();
        assert_eq!(p.page, 1);
        assert_eq!(p.pages, 3);
        assert_eq!(p.per_page, 100);
        assert_eq!(p.items, 250);
        assert_eq!(
            p.urls.next.unwrap(),
            "https://api.discogs.com/users/foo/wants?page=2"
        );
        assert!(p.urls.last.is_some());
    }

    #[test]
    fn test_deserialize_want_and_display() {
        let json = r#"{
            "id": 12345,
            "resource_url": "https://api.discogs.com/users/foo/wants/12345",
            "date_added": "2023-01-15T10:00:00-08:00",
            "rating": 0,
            "basic_information": {
                "id": 67890,
                "master_id": 111,
                "master_url": null,
                "resource_url": "https://api.discogs.com/releases/67890",
                "title": "Kind of Blue",
                "year": 1959,
                "formats": [],
                "artists": [{
                    "name": "Miles Davis",
                    "anv": "", "join": "", "role": "", "tracks": "",
                    "id": 99,
                    "resource_url": "https://api.discogs.com/artists/99"
                }],
                "labels": [],
                "thumb": "",
                "cover_image": "",
                "genres": ["Jazz"],
                "styles": ["Modal"]
            }
        }"#;
        let w: Want = serde_json::from_str(json).unwrap();
        assert_eq!(w.id, 12345);
        assert_eq!(w.basic_information.title, "Kind of Blue");
        assert_eq!(w.basic_information.year, 1959);
        assert_eq!(w.basic_information.artists[0].name, "Miles Davis");
        assert_eq!(w.basic_information.genres, vec!["Jazz"]);
        assert_eq!(w.to_string(), "Miles Davis — Kind of Blue (1959)");
    }

    #[test]
    fn test_deserialize_wants_envelope() {
        let json = r#"{
            "pagination": {"page":1,"pages":1,"per_page":50,"items":0,"urls":{}},
            "wants": []
        }"#;
        let w: Wants = serde_json::from_str(json).unwrap();
        assert_eq!(w.pagination.page, 1);
        assert_eq!(w.pagination.items, 0);
        assert!(w.wants.is_empty());
    }

    #[test]
    fn test_display_multiple_artists() {
        let mut want = Want::default();
        want.basic_information.title = "Bitches Brew".to_string();
        want.basic_information.year = 1970;
        want.basic_information.artists = vec![
            Artist { name: "Miles Davis".to_string(), ..Default::default() },
            Artist { name: "Wayne Shorter".to_string(), ..Default::default() },
        ];
        assert_eq!(want.to_string(), "Miles Davis, Wayne Shorter — Bitches Brew (1970)");
    }
}
