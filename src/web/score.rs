use axum::extract::Path;
use axum::response::Result;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, FixedOffset, NaiveDate};
use sqlx::{PgPool, Row};

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

// NOTE: Must be a Form data (?)

// Submit score function for each individual judge
pub async fn submit_score(
    State(pool): State<PgPool>,
    Json(score): Json<CandidateScore>,
) -> Result<Json<CandidateScore>> {
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
        .expect("Failed to submit score.");

    Ok(Json(score))
}

pub async fn get_candidate_scores(
    State(pool): State<PgPool>,
    Path(candidate_id): Path<i32>,
) -> Result<Json<Vec<CandidateScore>>> {
    let q = "SELECT * FROM scores WHERE candidate_id = ($1)";

    let rows = sqlx::query(q)
        .bind(&candidate_id)
        .fetch_all(&(pool))
        .await
        .expect("Failed to fetch candidate scores.");

    let candidate_scores: Vec<CandidateScore> = rows
        .iter()
        .map(|row| {
            let time_of_scoring: DateTime<FixedOffset> = row.get("time_of_scoring");

            let score = CandidateScore {
                candidate_id: row.get("candidate_id"),
                score: row.get("score"),
                criteria_id: row.get("criteria_id"),
                time_of_scoring: time_of_scoring.to_string(),
                judge_id: row.get("judge_id"),
                id: row.get("id"),
                max: row.get("max"),
            };

            score
        })
        .collect();

    let total_criteria_score = add_criteria_scores(State(pool), Path(candidate_id))
        .await
        .expect("Failed to add criteria scores.");

    println!("Total criteria score of candidate {candidate_id}: {total_criteria_score}");

    Ok(Json(candidate_scores))
}

// NOTE:
// Formula: Summation of total category score * weight
pub async fn add_criteria_scores(
    State(pool): State<PgPool>,
    Path(candidate_id): Path<i32>,
) -> Result<i32> {
    // Get the scores for each criteria on each categorty
    let q = "SELECT score FROM scores WHERE candidate_id = ($1)";

    let rows = sqlx::query(q)
        .bind(&candidate_id)
        .fetch_all(&(pool))
        .await
        .expect("Failed to fetch candidate scores on calculate_total_score()");

    let criteria_scores: Vec<i32> = rows
        .iter()
        .map(|row| {
            let score: i32 = row.get("score");

            score
        })
        .collect();

    let total_criteria_score: i32 = criteria_scores.iter().sum();

    Ok(total_criteria_score)
}
