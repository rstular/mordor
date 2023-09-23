use actix_session::Session;
use actix_web::{
    dev::ServiceRequest,
    get,
    web::{self, ServiceConfig},
    Error, HttpResponse, Responder,
};
use actix_web_httpauth::{extractors::basic::BasicAuth, middleware::HttpAuthentication};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};
use tracing::{error, warn};

use crate::{
    controllers::{structures::QueryDataOptionalRedirect, LoginModule, USERNAME_SESSION_KEY},
    database::entity::{
        BasicLoginUserColumn, BasicLoginUserEntity, LoginEntryActiveModel, LoginEntryEntity,
    },
    errors::{AppError, SessionError},
};

#[derive(Debug, Default)]
pub struct BasicAuthLoginModule;

impl LoginModule for BasicAuthLoginModule {
    fn name(&self) -> &'static str {
        "BasicAuth"
    }

    fn subpath(&self) -> &'static str {
        "/basic"
    }

    fn register_controller(&self, svc_cfg: &mut ServiceConfig) {
        svc_cfg.service(
            web::scope("")
                .wrap(HttpAuthentication::basic(validator))
                .service(index),
        );
    }

    fn display_name(&self) -> &'static str {
        "External users"
    }
}

async fn store_login_attempt(
    user_id: &str,
    success: bool,
    remote_addr: String,
    db_conn: &DbConn,
) -> bool {
    let login_entry = LoginEntryActiveModel {
        username: Set(user_id.to_owned()),
        success: Set(success),
        ip_address: Set(remote_addr),
        ..Default::default()
    };

    match LoginEntryEntity::insert(login_entry).exec(db_conn).await {
        Ok(_) => true,
        Err(e) => {
            error!(
                "Error inserting login entry (user_id='{user_id}', success={success}): {e}",
                user_id = user_id,
                success = success,
                e = e
            );
            false
        }
    }
}

async fn validator(
    mut req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // First make sure that we got the password & convert the password to bytes
    let password = if let Some(password) = credentials.password() {
        password.as_bytes()
    } else {
        warn!("No password provided");
        return Err((AppError::NotAuthenticated.into(), req));
    };

    // Obtain a database connection handle
    let db_conn = match req.extract::<web::Data<DbConn>>().await {
        Ok(db_conn) => db_conn,
        Err(e) => {
            error!("Error extracting database connection: {}", e);
            return Err((AppError::Internal.into(), req));
        }
    };

    // Extract the username
    let user_id = credentials.user_id();
    // Get the remote address
    let remote_addr = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("UNAVAILABLE")
        .to_owned();

    // Find the username in the database
    let user_entity = match BasicLoginUserEntity::find()
        .filter(BasicLoginUserColumn::Username.eq(user_id))
        .one(db_conn.as_ref())
        .await
    {
        Ok(Some(user_entity)) => user_entity,
        Ok(None) => {
            // If the username is not found, store a failed login attempt and return an error
            warn!("User '{}' not found", user_id);
            store_login_attempt(user_id, false, remote_addr, &db_conn).await;
            return Err((AppError::NotAuthenticated.into(), req));
        }
        Err(e) => {
            // If there is an error querying the database, return an error
            error!("Error querying database: {}", e);
            return Err((AppError::Internal.into(), req));
        }
    };

    // Verify that the password matches
    match argon2::verify_encoded(&user_entity.password, password) {
        Ok(true) => {}
        Ok(false) => {
            // If the password does not match, store a failed login attempt and return an error
            error!("Password does not match");
            store_login_attempt(user_id, false, remote_addr, &db_conn).await;
            return Err((AppError::NotAuthenticated.into(), req));
        }
        Err(e) => {
            // If there is an error verifying the password, return an error
            error!("Error verifying password: {}", e);
            return Err((AppError::Internal.into(), req));
        }
    };

    // Store a successful login attempt
    store_login_attempt(user_id, true, remote_addr, &db_conn).await;
    Ok(req)
}

#[get("/")]
async fn index(
    session: Session,
    auth_data: BasicAuth,
    query_data: web::Query<QueryDataOptionalRedirect>,
) -> Result<impl Responder, AppError> {
    let user_id = auth_data.user_id().to_string();
    session
        .insert(USERNAME_SESSION_KEY, &user_id)
        .map_err(Into::<SessionError>::into)?;

    if let Some(target) = &query_data.redirect {
        Ok(HttpResponse::TemporaryRedirect()
            .append_header(("Location", target.clone()))
            .finish())
    } else {
        Ok(HttpResponse::Ok().body(format!("Logged in as '{user_id}'")))
    }
}
