use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Deserialize, Serialize)]
pub struct Criteria {
    id: String,
    name: String,
    description: String,
    max_score: i32,
    weight: f64,
    // Relationships
    category_id: String,
}

pub async fn create_criteria(
    State(pool): State<PgPool>,
    Json(new_criteria): Json<Criteria>,
) -> (StatusCode, Json<Criteria>) {
    let query = "INSERT INTO criterias (id, name, description, max_score, weight, category_id) VALUES ($1, $2, $3, $4, $5, $6)";

    sqlx::query(query)
        .bind(&new_criteria.id)
        .bind(&new_criteria.name)
        .bind(&new_criteria.description)
        .bind(&new_criteria.max_score)
        .bind(&new_criteria.weight)
        .bind(&new_criteria.category_id)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::CREATED, Json(new_criteria))
}
