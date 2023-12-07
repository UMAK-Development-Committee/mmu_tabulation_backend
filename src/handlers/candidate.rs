use axum::extract::Path;
use axum::extract::State;
use axum::http;
use axum::response::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::error::AppError;

#[derive(Debug, Serialize, FromRow)]
pub struct Candidate {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub gender: i32,
    pub college_id: String,
    pub candidate_number: i32,
    // Relationships
    pub category_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateCandidate {
    first_name: String,
    middle_name: String,
    last_name: String,
    candidate_number: i32,
    gender: i32,
    college_id: String,
    category_id: uuid::Uuid,
}

pub async fn create_candidate(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateCandidate>,
) -> Result<(http::StatusCode, axum::Json<Candidate>), AppError> {
    let candidate = sqlx::query_as::<_, Candidate>(
        r#"
        INSERT INTO candidates (first_name, middle_name, last_name, birthdate, gender, college, category_id) 
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#
    )
    .bind(&payload.first_name)
    .bind(&payload.middle_name)
    .bind(&payload.last_name)
    .bind(&payload.gender)
    .bind(&payload.candidate_number)
    .bind(&payload.college_id)
    .bind(&payload.category_id)
    .fetch_one(&pool)
    .await?;

    Ok((http::StatusCode::CREATED, axum::Json(candidate)))
}

pub async fn get_candidates(
    State(pool): State<PgPool>,
) -> Result<axum::Json<Vec<Candidate>>, AppError> {
    let candidates = sqlx::query_as::<_, Candidate>("SELECT * FROM candidates")
        .fetch_all(&pool)
        .await?;

    Ok(axum::Json(candidates))
}

pub async fn get_candidate(
    State(pool): State<PgPool>,
    Path(candidate_id): Path<uuid::Uuid>,
) -> Result<axum::Json<Candidate>, AppError> {
    let candidate = sqlx::query_as::<_, Candidate>("SELECT * FROM candidates WHERE id = ($1)")
        .bind(&candidate_id)
        .fetch_one(&pool)
        .await?;

    Ok(axum::Json(candidate))
}
