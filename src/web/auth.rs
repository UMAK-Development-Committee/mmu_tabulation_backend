use axum::response::Result;
use axum::Form;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};

use crate::web::judge::Judge;

#[derive(Debug, Deserialize)]
pub struct User {
    name: String,
    password: String,
}

pub async fn login(State(pool): State<PgPool>, Form(user): Form<User>) -> Result<Json<Value>> {
    let q = "SELECT * FROM judges WHERE name = ($1) AND password = ($2)";
    let query = sqlx::query(q);

    let row = query
        .bind(&user.name)
        .bind(&user.password)
        .fetch_one(&(pool))
        .await
        .expect("Invalid user.");

    let judge_id: String = row.get("id");

    let auth_q = "UPDATE judges SET is_active = TRUE WHERE id = ($1)";
    let auth_query = sqlx::query(auth_q);

    auth_query
        .bind(judge_id)
        .execute(&(pool))
        .await
        .expect("Failed to update is_active value to TRUE.");

    println!("Welcome, {}!", user.name);

    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
pub struct LogOut {
    user_id: String,
}

pub async fn logout(State(pool): State<PgPool>, Form(logout): Form<LogOut>) -> Result<Json<Value>> {
    let q = "UPDATE judges SET is_active = FALSE WHERE id = ($1)";
    let query = sqlx::query(q);

    query
        .bind(&logout.user_id)
        .execute(&(pool))
        .await
        .expect("Failed to update is_active to FALSE.");

    println!("Goodbye!");

    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}
