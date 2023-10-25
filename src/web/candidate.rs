use axum::extract::Path;
use axum::extract::State;
use axum::http;
use axum::response::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Serialize, FromRow)]
pub struct Candidate {
    id: uuid::Uuid,
    first_name: String,
    middle_name: String,
    last_name: String,
    birthdate: chrono::NaiveDate,
    gender: i32,
    college: String,
    // Relationships
    category_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateCandidate {
    first_name: String,
    middle_name: String,
    last_name: String,
    birthdate: chrono::NaiveDate,
    gender: i32,
    college: String,
    category_id: uuid::Uuid,
}

pub async fn create_candidate(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateCandidate>,
) -> Result<(http::StatusCode, axum::Json<Candidate>), http::StatusCode> {
    let res = sqlx::query_as::<_, Candidate>(
        r#"
        INSERT INTO candidates (first_name, middle_name, last_name, birthdate, gender, college, category_id) 
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#
    )
    .bind(&payload.first_name)
    .bind(&payload.middle_name)
    .bind(&payload.last_name)
    .bind(&payload.birthdate)
    .bind(&payload.gender)
    .bind(&payload.college)
    .bind(&payload.category_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(candidate) => Ok((http::StatusCode::CREATED, axum::Json(candidate))),
        Err(err) => {
            eprintln!("Failed to create candidate: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_candidates(
    State(pool): State<PgPool>,
) -> Result<axum::Json<Vec<Candidate>>, http::StatusCode> {
    let res = sqlx::query_as::<_, Candidate>("SELECT * FROM candidates")
        .fetch_all(&pool)
        .await;

    match res {
        Ok(candidates) => Ok(axum::Json(candidates)),
        Err(err) => {
            eprintln!("Failed to get all candidates: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_candidate(
    State(pool): State<PgPool>,
    Path(candidate_id): Path<uuid::Uuid>,
) -> Result<axum::Json<Candidate>, http::StatusCode> {
    let res = sqlx::query_as::<_, Candidate>("SELECT * FROM candidates WHERE id = ($1)")
        .bind(&candidate_id)
        .fetch_one(&pool)
        .await;

    match res {
        Ok(candidate) => Ok(axum::Json(candidate)),
        Err(err) => {
            eprintln!("Failed to get candidate: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
