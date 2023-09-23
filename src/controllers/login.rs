use actix_session::Session;
use actix_web::{
    get,
    http::{header::ContentType, StatusCode},
    web, HttpResponse, Responder,
};
use tera::Context;
use tracing::error;

use crate::{
    controllers::{structures::QueryDataOptionalRedirect, TEMPLATES, USERNAME_SESSION_KEY},
    errors::AppError,
};

use super::AllModuleData;

#[get("/")]
pub async fn index(
    module_data: web::Data<AllModuleData>,
    query_data: web::Query<QueryDataOptionalRedirect>,
    session: Session,
) -> Result<impl Responder, AppError> {
    let mut ctx = Context::new();
    ctx.insert("modules", &module_data);

    if let Some(target) = &query_data.redirect {
        // If the user is already logged in & we have a redirect target, redirect to the target
        if let Ok(Some(_)) = session.get::<String>(USERNAME_SESSION_KEY) {
            return Ok(HttpResponse::TemporaryRedirect()
                .append_header(("Location", target.clone()))
                .finish());
        }

        // If the user is not logged in & we have a redirect target, populate the template with the target
        ctx.insert("redirect", &target);
    }

    match TEMPLATES.render("login.html", &ctx) {
        Ok(s) => Ok(HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::html())
            .body(s)),
        Err(e) => {
            error!("Error rendering template: {:?}", e);
            Err(AppError::from(e))
        }
    }
}
