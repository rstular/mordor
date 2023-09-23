use serde::Deserialize;

pub mod saml;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub saml: saml::Settings,
}
