use axum::extract::Path;
use axum::response::Result;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDate;
use sqlx::{PgPool, Row};

#[derive(Debug, Deserialize, Serialize)]
pub struct Candidate {
    id: i32,
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
    Json(new_candidate): Json<Candidate>,
) -> (StatusCode, Json<Candidate>) {
    let query = "INSERT INTO candidates (id, first_name, middle_name, last_name, birthdate, gender, college, category_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

    let parsed_birthdate =
        sqlx::types::chrono::NaiveDate::parse_from_str(&new_candidate.birthdate, "%Y-%m-%d")
            .expect("Date is invalid.");

    sqlx::query(query)
        .bind(&new_candidate.id)
        .bind(&new_candidate.first_name)
        .bind(&new_candidate.middle_name)
        .bind(&new_candidate.last_name)
        .bind(parsed_birthdate)
        .bind(&new_candidate.gender)
        .bind(&new_candidate.college)
        .bind(&new_candidate.category_id)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::CREATED, Json(new_candidate))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CandidateScore {
    id: String,
    score: i32,
    max: i32,
    time_of_scoring: String,
    // Relationships
    candidate_id: i32,
    criteria_id: String,
    judge_id: String,
}

pub async fn submit_score(
    State(pool): State<PgPool>,
    Json(score): Json<CandidateScore>,
) -> (StatusCode, Json<CandidateScore>) {
    let query = "INSERT INTO scores (id, score, max, time_of_scoring, candidate_id, criteria_id, judge_id) VALUES ($1, $2, $3, $4, $5, $6, $7)";

    let parsed_time_of_scoring = sqlx::types::chrono::DateTime::parse_from_str(
        &score.time_of_scoring,
        "%Y-%m-%d %H:%M:%S %z",
    )
    .expect("Date and time is invalid.");

    sqlx::query(query)
        .bind(&score.id)
        .bind(&score.score)
        .bind(&score.max)
        .bind(parsed_time_of_scoring)
        .bind(&score.candidate_id)
        .bind(&score.criteria_id)
        .bind(&score.judge_id)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::OK, Json(score))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CandidateNote {
    id: String,
    note: String,
    last_change: String,
    // Relationships
    candidate_id: i32,
    judge_id: String,
}

pub async fn create_note(
    State(pool): State<PgPool>,
    Json(new_note): Json<CandidateNote>,
) -> (StatusCode, Json<CandidateNote>) {
    let query = "INSERT INTO notes (id, note, last_change, candidate_id, judge_id) VALUES ($1, $2, $3, $4, $5)";

    let parsed_last_change_date = sqlx::types::chrono::DateTime::parse_from_str(
        &new_note.last_change,
        "%Y-%m-%d %H:%M:%S %z",
    )
    .expect("Date and time is invalid.");

    sqlx::query(query)
        .bind(&new_note.id)
        .bind(&new_note.note)
        .bind(parsed_last_change_date)
        .bind(&new_note.candidate_id)
        .bind(&new_note.judge_id)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::CREATED, Json(new_note))
}

// GET
pub async fn get_candidates(State(pool): State<PgPool>) -> Result<Json<Vec<Candidate>>> {
    let q = "SELECT * FROM candidates";
    let query = sqlx::query(q);

    let rows = query
        .fetch_all(&(pool))
        .await
        .expect("Failed to fetch list of candidates.");

    let candidates: Vec<Candidate> = rows
        .iter()
        .map(|row| {
            let birthdate_sql: NaiveDate = row.get("birthdate");

            let candidate = Candidate {
                id: row.get("id"),
                first_name: row.get("first_name"),
                middle_name: row.get("middle_name"),
                last_name: row.get("last_name"),
                birthdate: birthdate_sql.to_string(),
                gender: row.get("gender"),
                college: row.get("college"),
                category_id: row.get("category_id"),
            };

            candidate
        })
        .collect();

    Ok(Json(candidates))
}

pub async fn get_candidate(
    State(pool): State<PgPool>,
    Path(candidate_id): Path<i32>,
) -> Result<Json<Candidate>> {
    let q = "SELECT * FROM candidates WHERE id = ($1)";
    let query = sqlx::query(q);

    let row = query
        .bind(candidate_id)
        .fetch_one(&(pool))
        .await
        .expect("Failed to fetch candidate, check if the candidate exists.");

    let birthdate_sql: NaiveDate = row.get("birthdate");

    let candidate = Candidate {
        id: row.get("id"),
        first_name: row.get("first_name"),
        middle_name: row.get("middle_name"),
        last_name: row.get("last_name"),
        birthdate: birthdate_sql.to_string(),
        gender: row.get("gender"),
        college: row.get("college"),
        category_id: row.get("category_id"),
    };

    Ok(Json(candidate))
}
