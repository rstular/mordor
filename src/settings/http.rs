use std::net::SocketAddrV4;

use serde::Deserialize;

use crate::utils::defaults;

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default = "defaults::http::address")]
    pub address: SocketAddrV4,
    #[serde(default)]
    pub path: Option<String>,
}
