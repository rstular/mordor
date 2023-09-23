use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub upstream_url: Url,
}
