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
) -> Result<axum::Json<Judge>, http::StatusCode> {
    let res =
        sqlx::query_as::<_, Judge>("SELECT * FROM judges WHERE name = ($1) AND password = ($2)")
            .bind(&user.name)
            .bind(&user.password)
            .fetch_one(&pool)
            .await;

    match res {
        Ok(judge) => {
            sqlx::query("UPDATE judges SET is_active = TRUE WHERE id = ($1)")
                .bind(&judge.id)
                .execute(&pool)
                .await
                .map_err(|_| {
                    eprintln!("Failed to set is_active to TRUE");
                    http::StatusCode::INTERNAL_SERVER_ERROR
                })?;

            println!("Welcome, {}!", judge.name);
            println!("Details: {:?}\n", judge);

            Ok(axum::Json(judge))
        }
        Err(err) => {
            eprintln!("Failed to login: {err:?}");

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
) -> Result<http::StatusCode, http::StatusCode> {
    let res = sqlx::query("UPDATE judges SET is_active = FALSE WHERE id = ($1)")
        .bind(&logout.user_id)
        .execute(&pool)
        .await;

    match res {
        Ok(_) => {
            println!("Goodbye!");

            Ok(http::StatusCode::OK)
        }
        Err(err) => {
            eprintln!("Failed to logout: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
