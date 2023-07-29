use std::fmt;

use actix_web::{http::{StatusCode, header::ContentType}, HttpResponse, body};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
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
    #[error("WebSocketError: {0}")]
    WebSocketError(#[from] WebSocketError),
    #[error("\"{0}\" is None")]
    NoneValue(&'static str),
}

impl ApiError {
    pub fn code(&self) -> u16 {
        match *self {
            Self::DbError(_)
            | Self::IoError(_)
            | Self::HashError(_)
            | Self::RegexError(_)
            | Self::Utf8Error(_) 
            | Self::TeraError(_) => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            Self::WebSocketError(e) => {
                match e {
                    WebSocketError::LoginError(_) => StatusCode::UNAUTHORIZED.as_u16(),
                }
            }
            Self::SerdeError(_) => StatusCode::BAD_REQUEST.as_u16(),
            Self::NoneValue(_) => StatusCode::NOT_FOUND.as_u16(),
        }
    }
}

impl actix_web::error::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse<body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(json!({ "message": self.to_string() }))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Self::DbError(_)
            | Self::IoError(_)
            | Self::HashError(_)
            | Self::RegexError(_)
            | Self::Utf8Error(_) 
            | Self::TeraError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WebSocketError(e) => {
                match e {
                    WebSocketError::LoginError(_) => StatusCode::UNAUTHORIZED,
                }
            }
            Self::SerdeError(_) => StatusCode::BAD_REQUEST,
            Self::NoneValue(_) => StatusCode::NOT_FOUND,
        }
    }
}

#[derive(Error, Debug, Clone, Copy)]
pub enum WebSocketError {
    LoginError(&'static str)
}

impl fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::LoginError(v) => {
                log::warn!(target: "Little CDN", "Failed to login user: {}", v);
                write!(f, "Failed to login user: {}", v)
            }
        }
    }
}
