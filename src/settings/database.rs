use serde::Deserialize;

use crate::utils::defaults;

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default = "defaults::database::file")]
    pub file: String,
}

impl Settings {
    pub fn get_url(&self) -> String {
        format!("sqlite://{}?mode=rwc", self.file)
    }
}
