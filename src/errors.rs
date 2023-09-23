use actix_session::{SessionGetError, SessionInsertError};
use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct APIError {
    message: String,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Could not get data: {0}")]
    GetError(#[from] SessionGetError),
    #[error("Could not set data: {0}")]
    SetError(#[from] SessionInsertError),
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Could not access session data: {0}")]
    SessionError(#[from] SessionError),
    #[error("User not authenticated")]
    NotAuthenticated,
    #[error("Internal error")]
    Internal,
    #[error("A templating error has occured: {0}")]
    TemplateError(#[from] tera::Error),
    #[error("An upstream error has occured: {0}")]
    UpstreamError(#[from] reqwest::Error),
    #[error("An unknown error has occured")]
    UnknownUpstreamError,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(APIError {
                message: self.to_string(),
            })
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::SessionError(inner) => match inner {
                SessionError::GetError(_) => StatusCode::UNAUTHORIZED,
                SessionError::SetError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            AppError::NotAuthenticated => StatusCode::UNAUTHORIZED,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::TemplateError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UpstreamError(_) => StatusCode::BAD_GATEWAY,
            AppError::UnknownUpstreamError => StatusCode::BAD_GATEWAY,
        }
    }
}
