use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Deserialize, Serialize)]
pub struct Judge {
    id: String,
    name: String,
    password: String,
    is_active: bool,
    // Relationships
    category_id: String,
}

pub async fn create_judge(
    State(pool): State<PgPool>,
    Json(new_judge): Json<Judge>,
) -> (StatusCode, Json<Judge>) {
    let query = "INSERT INTO judges (id, name, password, is_active, category_id) VALUES ($1, $2, $3, $4, $5)";

    sqlx::query(query)
        .bind(&new_judge.id)
        .bind(&new_judge.name)
        .bind(&new_judge.password)
        .bind(&new_judge.is_active)
        .bind(&new_judge.category_id)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::CREATED, Json(new_judge))
}
