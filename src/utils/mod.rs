use reqwest::{redirect::Policy, Client};
use tracing::error;

use crate::errors::AppError;

pub mod defaults;

#[macro_export]
macro_rules! enclose {
    ( ($( $x:ident ),*), $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub fn get_http_client() -> Result<Client, AppError> {
    Client::builder()
        .redirect(Policy::none())
        .build()
        .map_err(|err| {
            error!("Error building HTTP client: {:?}", err);
            AppError::Internal
        })
}
