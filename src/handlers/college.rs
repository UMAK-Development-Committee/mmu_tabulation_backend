use axum::{extract, http, response::Result};
use serde::{Serialize};
use sqlx::{FromRow, PgPool};


#[derive(Debug, Serialize, FromRow)]
pub struct College {
    college_id: String,
    college_logo_path: String,
    college_name: String
}

pub async fn get_colleges(
    extract::State(pool): extract::State<PgPool>,
) -> Result<axum::Json<Vec<College>>, http::StatusCode> {
    let res = sqlx::query_as::<_, College>(
        "SELECT * FROM college",
    )
    .fetch_all(&pool)
    .await;

    match res {
        Ok(category) => Ok(axum::Json(category)),
        Err(err) => {
            eprintln!("Failed to get college information: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
