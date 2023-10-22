use axum::extract::Path;
use axum::http;
use axum::response::Result;
use axum::Form;
use axum::{extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};

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
    category_id: String,
}

impl Candidate {
    fn new(
        first_name: String,
        middle_name: String,
        last_name: String,
        birthdate: String,
        gender: i32,
        college: String,
        category_id: String,
    ) -> Self {
        let uuid = uuid::Uuid::new_v4();
        let parsed_birthdate =
            chrono::NaiveDate::parse_from_str(&birthdate, "%Y-%m-%d").expect("Date is invalid.");

        Self {
            id: uuid,
            first_name,
            middle_name,
            last_name,
            birthdate: parsed_birthdate,
            gender,
            college,
            category_id,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCandidate {
    first_name: String,
    middle_name: String,
    last_name: String,
    birthdate: String,
    gender: i32,
    college: String,
    // Relationships
    category_id: String,
}

// POST
pub async fn create_candidate(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateCandidate>,
) -> Result<(http::StatusCode, axum::Json<Candidate>), http::StatusCode> {
    let candidate = Candidate::new(
        payload.first_name,
        payload.middle_name,
        payload.last_name,
        payload.birthdate,
        payload.gender,
        payload.college,
        payload.category_id,
    );

    let res = sqlx::query(
        r#"
        INSERT INTO candidates (id, first_name, middle_name, last_name, birthdate, gender, college, category_id) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(&candidate.id)
    .bind(&candidate.first_name)
    .bind(&candidate.middle_name)
    .bind(&candidate.last_name)
    .bind(&candidate.birthdate)
    .bind(&candidate.gender)
    .bind(&candidate.college)
    .bind(&candidate.category_id)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => Ok((http::StatusCode::CREATED, axum::Json(candidate))),
        Err(err) => {
            eprintln!("Failed to create candidate: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// GET
pub async fn get_candidates(
    State(pool): State<PgPool>,
) -> Result<axum::Json<Vec<Candidate>>, http::StatusCode> {
    let q = "SELECT * FROM candidates";

    let res = sqlx::query_as::<_, Candidate>(q).fetch_all(&pool).await;

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
    Path(candidate_id): Path<i32>,
) -> Result<axum::Json<Candidate>, http::StatusCode> {
    let q = "SELECT * FROM candidates WHERE id = ($1)";

    let res = sqlx::query_as::<_, Candidate>(q)
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
