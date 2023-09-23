use std::sync::Arc;

use actix_session::Session;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use reqwest::header::HeaderValue;
use sea_orm::{DbConn, EntityTrait, Set};
use tracing::{error, warn};
use url::Url;

use crate::{
    controllers::{
        structures::{FormDataSAMLResponse, QueryDataRedirect},
        LoginModule, USERNAME_SESSION_KEY,
    },
    database::entity::{LoginEntryActiveModel, LoginEntryEntity},
    errors::{AppError, SessionError},
    settings::modules::saml,
    utils::get_http_client,
};

pub const SAML_PROXY_COOKIE: &str = "saml-proxy";
pub const SAML_USER_ALIAS: &str = "[SAML-USER]";

#[derive(Debug)]
pub struct SAMLLoginModule {
    config: Arc<saml::Settings>,
}

impl SAMLLoginModule {
    pub fn new(saml_config: &saml::Settings) -> Self {
        Self {
            config: Arc::new(saml_config.clone()),
        }
    }
}

impl LoginModule for SAMLLoginModule {
    fn name(&self) -> &'static str {
        "NetIDAuth"
    }

    fn subpath(&self) -> &'static str {
        "/saml"
    }

    fn register_controller(&self, svc_cfg: &mut actix_web::web::ServiceConfig) {
        svc_cfg
            .service(start)
            .service(consume)
            .app_data(web::Data::<Arc<saml::Settings>>::new(self.config.clone()));
    }

    fn display_name(&self) -> &'static str {
        "NetID Login (TU Delft)"
    }
}

#[get("/")]
async fn start(
    config: web::Data<Arc<saml::Settings>>,
    query_data: web::Query<QueryDataRedirect>,
    session: Session,
) -> Result<impl Responder, AppError> {
    let client = get_http_client()?;

    let saml_url = config.upstream_url.join("/start").map_err(|err| {
        error!("Error parsing upstream URL: {:?}", err);
        AppError::Internal
    })?;

    let resp = client
        .get(saml_url)
        .query(&[("redirect", &query_data.redirect)])
        .send()
        .await
        .map_err(|err| {
            error!("Error sending request to upstream: {:?}", err);
            Into::<AppError>::into(err)
        })?;

    if !resp.status().is_redirection() {
        error!("Unexpected status code from upstream: {}", resp.status());
        return Err(AppError::UnknownUpstreamError);
    }

    let location = resp
        .headers()
        .get("Location")
        .ok_or(AppError::UnknownUpstreamError)?;

    if let Err(e) = location.to_str() {
        error!("Error parsing Location header: {:?}", e);
        return Err(AppError::UnknownUpstreamError);
    }

    let cookie = match resp.headers().get("Set-Cookie").map(HeaderValue::to_str) {
        Some(Ok(val)) => val,
        Some(Err(e)) => {
            error!("Error parsing Set-Cookie header: {:?}", e);
            return Err(AppError::UnknownUpstreamError);
        }
        None => {
            error!("No Set-Cookie header found");
            return Err(AppError::UnknownUpstreamError);
        }
    };

    session.insert(SAML_PROXY_COOKIE, cookie).map_err(|err| {
        error!("Error inserting cookie into session: {:?}", err);
        SessionError::SetError(err)
    })?;

    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", location))
        .finish())
}

#[post("/consume/")]
async fn consume(
    config: web::Data<Arc<saml::Settings>>,
    db_conn: web::Data<DbConn>,
    query_data: web::Form<FormDataSAMLResponse>,
    session: Session,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    // Get the SAML proxy cookie from the session
    let saml_cookie = session
        .get::<String>(SAML_PROXY_COOKIE)
        .map_err(Into::<SessionError>::into)?
        .ok_or(AppError::NotAuthenticated)?;

    // Construct the HTTP client
    let client = get_http_client()?;

    let saml_url = config.upstream_url.join("/consume").map_err(|err| {
        error!("Error parsing upstream URL: {:?}", err);
        AppError::Internal
    })?;

    let resp = client
        .post(saml_url)
        .form(&[
            ("SAMLResponse", &query_data.saml_response),
            ("RelayState", &query_data.relay_state),
        ])
        .header("Cookie", saml_cookie)
        .send()
        .await
        .map_err(|err| {
            error!("Error sending request to upstream: {:?}", err);
            Into::<AppError>::into(err)
        })?;

    // Proxy should redirect to the redirect URL
    if !resp.status().is_redirection() {
        error!("Unexpected status code from upstream: {}", resp.status());
        return Err(AppError::UnknownUpstreamError);
    }

    if let Err(err) = session.insert(USERNAME_SESSION_KEY, SAML_USER_ALIAS) {
        error!("Error inserting username into session: {:?}", err);
        return Err(AppError::Internal);
    }

    // Update the cookie received from the upstream
    match resp.headers().get("Set-Cookie").map(HeaderValue::to_str) {
        Some(Ok(val)) => {
            if let Err(err) = session.insert(SAML_PROXY_COOKIE, val).map_err(|err| {
                error!("Error inserting cookie into session: {:?}", err);
                SessionError::SetError(err)
            }) {
                error!("Error inserting cookie into session: {:?}", err);
            }
        }
        Some(Err(e)) => {
            warn!("Error parsing Set-Cookie header: {:?}", e);
        }
        None => {
            warn!("No Set-Cookie header found");
        }
    };

    // Get redirect location
    let location = match resp.headers().get("Location").map(HeaderValue::to_str) {
        Some(Ok(val)) => val,
        Some(Err(e)) => {
            error!("Error parsing Location header: {:?}", e);
            return Err(AppError::UnknownUpstreamError);
        }
        None => {
            error!("No Location header found");
            return Err(AppError::UnknownUpstreamError);
        }
    };
    let location_url = match Url::parse(location) {
        Ok(url) => url,
        Err(e) => {
            error!("Error parsing Location header: {:?}", e);
            return Err(AppError::UnknownUpstreamError);
        }
    };

    // Get the remote address
    let remote_addr = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("UNAVAILABLE")
        .to_owned();
    let login_entry = LoginEntryActiveModel {
        username: Set(SAML_USER_ALIAS.to_owned()),
        success: Set(true),
        ip_address: Set(remote_addr),
        ..Default::default()
    };
    if let Err(err) = LoginEntryEntity::insert(login_entry)
        .exec(db_conn.as_ref())
        .await
    {
        error!("Error inserting login entry: {err}");
    }

    Ok(HttpResponse::SeeOther()
        .append_header((
            "Location",
            location_url.path().to_owned() + location_url.query().unwrap_or(""),
        ))
        .finish())
}
