use actix_web::{http::{StatusCode, header::ContentType}, HttpResponse, body};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database Error: {0}")]
    DbError(#[from] sea_orm::DbErr),
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Error while serializing or deserializing JSON: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Error in hashing password")]
    HashError(#[from] pwhash::error::Error),
    #[error("Error in parsing regex: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Error in parsing Utf8 string: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Error while rendering template: {0}")]
    TeraError(#[from] tera::Error),
    #[error("Multipart Error: {0}")]
    MultipartError(#[from] actix_multipart::MultipartError),
    #[error("{0}")]
    ActixError(#[from] actix_web::error::Error),
    #[error("{0}")]
    ApiError(#[from] ApiError),
    #[error("{0}")]
    WebSocketError(#[from] WebSocketError),
    #[error("\"{0}\" is Null")]
    NoneValue(&'static str),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::DbError(_)
            | Self::IoError(_)
            | Self::HashError(_)
            | Self::RegexError(_)
            | Self::Utf8Error(_)
            | Self::TeraError(_)
            | Self::MultipartError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ActixError(e) => e.as_response_error().status_code(),
            Self::WebSocketError(e) => e.status_code(),
            Self::ApiError(e) => e.status_code(),
            Self::SerdeError(_) => StatusCode::BAD_REQUEST,
            Self::NoneValue(_) => StatusCode::NOT_FOUND,
        }
    }
}

impl actix_web::error::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse<body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(json!({
                "code": self.status_code().as_u16(),
                "message": self.to_string()
            }))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::DbError(_)
            | Self::IoError(_)
            | Self::HashError(_)
            | Self::RegexError(_)
            | Self::Utf8Error(_)
            | Self::TeraError(_)
            | Self::MultipartError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ActixError(e) => e.as_response_error().status_code(),
            Self::ApiError(e) => e.status_code(),
            Self::WebSocketError(e) => e.status_code(),
            Self::SerdeError(_) => StatusCode::BAD_REQUEST,
            Self::NoneValue(_) => StatusCode::NOT_FOUND,
        }
    }
}

#[derive(Error, Debug, Clone, Copy)]
pub enum WebSocketError {
    #[error("tmp error: {0}")]
    LoginError(&'static str),
}

impl WebSocketError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::LoginError(_) => StatusCode::UNAUTHORIZED,
        }
    }
}

#[derive(Error, Debug, Clone, Copy)]
pub enum ApiError {
    #[error("You don't have enough storage space")]
    LowStorage,
    #[error("File with that name already exists")]
    AlreadyExists,
}

impl ApiError {
    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::LowStorage => StatusCode::BAD_REQUEST,
            Self::AlreadyExists => StatusCode::CONFLICT,
        }
    }
}
