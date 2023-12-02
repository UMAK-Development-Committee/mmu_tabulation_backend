// NOTE: Not sure if I need this module, will leave it here for now just in case I need custom Errors
// Might use anyhow instead

// Remove some noise
#![allow(unused)]

use axum::http;
use axum::response::{IntoResponse, Response};
use rust_xlsxwriter::XlsxError;

#[derive(Debug)]
pub struct AppError {
    message: String,
    code: http::StatusCode,
}

impl AppError {
    pub fn new(code: http::StatusCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError {
            code: http::StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("SQLx Error: {}", error),
        }
    }
}

impl From<XlsxError> for AppError {
    fn from(error: XlsxError) -> Self {
        AppError {
            code: http::StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Xlsx Error: {}", error),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError {
            code: http::StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Anyhow Error: {}", error),
        }
    }
}

// impl IntoResponse for XlsxError {
//     fn into_response(self) -> Response {
//         println!("->> {:?}\n", self);
//
//         // You might want to customize this part based on how you want to handle XlsxError in responses
//         (
//             http::StatusCode::INTERNAL_SERVER_ERROR,
//             format!("XlsxError: {}", self),
//         )
//             .into_response()
//     }
// }

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        println!("->> {self:?}\n");

        (self.code, self.message).into_response()
    }
}
