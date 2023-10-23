use axum::extract::State;
use axum::http;
use axum::response::Result;
use axum::Form;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::web::judge::Judge;

#[derive(Debug, Deserialize)]
pub struct User {
    name: String,
    password: String,
}

pub async fn login(
    State(pool): State<PgPool>,
    axum::Json(user): axum::Json<User>,
) -> Result<http::StatusCode, http::StatusCode> {
    let q = "SELECT * FROM judges WHERE name = ($1) AND password = ($2)";

    let res = sqlx::query(q)
        .bind(&user.name)
        .bind(&user.password)
        .fetch_one(&pool);

    match res.await {
        Ok(row) => {
            let judge_id: String = row.get("id");

            let auth_query = "UPDATE judges SET is_active = TRUE WHERE id = ($1)";

            sqlx::query(auth_query)
                .bind(&judge_id)
                .execute(&pool)
                .await
                .expect("Failed to update is_active value to TRUE.");

            println!("Welcome, {}!", user.name);

            Ok(http::StatusCode::OK)
        }
        Err(err) => {
            eprintln!("User doesn't exist: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LogOut {
    user_id: uuid::Uuid,
}

pub async fn logout(
    State(pool): State<PgPool>,
    axum::Json(logout): axum::Json<LogOut>,
) -> Result<http::StatusCode> {
    let q = "UPDATE judges SET is_active = FALSE WHERE id = ($1)";

    sqlx::query(q)
        .bind(&logout.user_id)
        .execute(&pool)
        .await
        .expect("Failed to update is_active to FALSE.");

    println!("Goodbye!");

    Ok(http::StatusCode::OK)
}
