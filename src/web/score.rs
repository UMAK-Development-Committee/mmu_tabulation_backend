use axum::extract::{Path, Query};
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

#[derive(Debug, Deserialize)]
pub struct CandidateScoreQuery {
    event_id: String,
    category_id: String,
}

// NOTE: Will only get the final score for ONE category only, maybe loop on the client side?
pub async fn get_candidate_scores(
    State(pool): State<PgPool>,
    Path(candidate_id): Path<i32>,
    Query(query): Query<CandidateScoreQuery>,
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

    let final_category_score = get_category_score(State(pool), Path(candidate_id), Query(query))
        .await
        .expect("Failed to add criteria scores.");

    println!("Category score: {final_category_score}");

    Ok(Json(candidate_scores))
}

// NOTE:
// Formula: Summation of total category score * weight
pub async fn get_category_score(
    State(pool): State<PgPool>,
    Path(candidate_id): Path<i32>,
    Query(category_query): Query<CandidateScoreQuery>,
) -> Result<f32> {
    // Get the number of categories for the event
    // NOTE: Is this needed?
    let category_count_q =
        "SELECT COUNT(*) AS category_count FROM categories WHERE event_id = ($1)";

    let category_count_row = sqlx::query(category_count_q)
        .bind(&category_query.event_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch category count.");

    let category_count: i64 = category_count_row.get("category_count");

    println!("Category count: {category_count}\n");

    // NOTE: This is the original beginning of this function, but since a score tracker might be
    // needed, perhaps getting the score for each criteria one-by-one would be useful instead of
    // waiting before all scores are submitted?

    // Get category weight
    let weight_q = "SELECT weight from categories WHERE event_id = ($1) AND id = ($2)";

    let row = sqlx::query(weight_q)
        .bind(&category_query.event_id)
        .bind(&category_query.category_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch category weight.");

    let category_weight: f32 = row.get("weight");

    // Get the scores for each criteria on each category
    // NOTE: do we need to display each criteria scores? Removing the last part of this query
    // (s.criteria_id = ($3)) will make it so that it will just get all the scores from all judges immediately
    let scores_q = "SELECT s.score, s.judge_id FROM scores s JOIN criterias c ON s.criteria_id = c.id WHERE s.candidate_id = ($1) AND c.category_id = ($2) AND s.criteria_id = ($3)";

    // NOTE: hard coded for now
    let criteria_id = "criteria1";

    let rows = sqlx::query(scores_q)
        .bind(&candidate_id)
        .bind(&category_query.category_id)
        .bind(&criteria_id)
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch candidate scores on calculate_total_score()");

    let criteria_scores: Vec<i32> = rows
        .iter()
        .map(|row| {
            let score: i32 = row.get("score");
            let judge_id: String = row.get("judge_id");

            println!("{judge_id}: {score}");

            score
        })
        .collect();

    let category_score: i32 = criteria_scores.iter().sum();

    println!("\nTotal score: {category_score}");
    println!("Category weight: {category_weight}\n");

    let final_category_score = (category_score as f32) * category_weight;

    Ok(final_category_score)
}

fn calculate_final_score(category_scores: Vec<f32>) -> f32 {
    category_scores.iter().sum()
}
