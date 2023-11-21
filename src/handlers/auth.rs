use axum::extract::State;
use axum::http;
use axum::response::Result;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AppError;
use crate::handlers::judge::Judge;

#[derive(Debug, Deserialize)]
pub struct User {
    username: String,
    password: String,
}

pub async fn login(
    State(pool): State<PgPool>,
    axum::Json(user): axum::Json<User>,
) -> Result<axum::Json<Judge>, AppError> {
    let res = sqlx::query_as::<_, Judge>(
        "SELECT * FROM judges WHERE username = ($1) AND password = ($2)",
    )
    .bind(&user.username)
    .bind(&user.password)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(judge) => {
            sqlx::query("UPDATE judges SET is_active = TRUE WHERE id = ($1)")
                .bind(&judge.id)
                .execute(&pool)
                .await
                .map_err(|err| {
                    eprintln!("Failed to set is_active to TRUE");
                    AppError::new(
                        http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to set is_active to TRUE: {}", err),
                    )
                })?;

            println!("Welcome, {}!", judge.name);
            println!("Details: {:?}\n", judge);

            Ok(axum::Json(judge))
        }
        Err(err) => {
            eprintln!("Failed to login: {err:?}");

            Err(AppError::new(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to login: {}", err),
            ))
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
) -> Result<http::StatusCode, AppError> {
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

            Err(AppError::new(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to logout: {}", err),
            ))
        }
    }
}
