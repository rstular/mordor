use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QueryDataOptionalRedirect {
    pub redirect: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryDataRedirect {
    pub redirect: String,
}

#[derive(Debug, Deserialize)]
pub struct FormDataSAMLResponse {
    #[serde(rename = "SAMLResponse")]
    pub saml_response: String,
    #[serde(rename = "RelayState")]
    pub relay_state: String,
}
