use anyhow::Context;
use axum::extract::{Path, Query, State};
use axum::http;
use axum::response::Result;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, FixedOffset, NaiveDate};
use sqlx::{FromRow, PgPool, Row};

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Score {
    id: uuid::Uuid,
    score: i32,
    max: i32,
    time_of_scoring: chrono::DateTime<chrono::Utc>,
    // Relationships
    candidate_id: uuid::Uuid,
    criteria_id: uuid::Uuid,
    category_id: uuid::Uuid,
    judge_id: uuid::Uuid,
}

impl Score {
    fn new(score: CreateScore) -> Self {
        let now = chrono::Utc::now();
        let uuid = uuid::Uuid::new_v4();

        Self {
            category_id: score.category_id,
            criteria_id: score.criteria_id,
            judge_id: score.judge_id,
            candidate_id: score.candidate_id,
            max: score.max,
            score: score.score,
            time_of_scoring: now,
            id: uuid,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateScore {
    score: i32,
    max: i32,
    candidate_id: uuid::Uuid,
    criteria_id: uuid::Uuid,
    category_id: uuid::Uuid,
    judge_id: uuid::Uuid,
}

// Submit score function for each individual judge
pub async fn submit_score(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateScore>,
) -> Result<(http::StatusCode, axum::Json<Score>), http::StatusCode> {
    let query = "INSERT INTO scores (id, score, max, time_of_scoring, candidate_id, criteria_id, category_id, judge_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

    let score = Score::new(payload);

    let res = sqlx::query(query)
        .bind(&score.id)
        .bind(&score.score)
        .bind(&score.max)
        .bind(&score.time_of_scoring)
        .bind(&score.candidate_id)
        .bind(&score.criteria_id)
        .bind(&score.category_id)
        .bind(&score.judge_id)
        .execute(&pool);

    match res.await {
        Ok(_) => Ok((http::StatusCode::CREATED, axum::Json(score))),
        Err(err) => {
            eprintln!("Failed to submit score: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScoreQuery {
    candidate_id: uuid::Uuid,
    category_id: uuid::Uuid,
    event_id: uuid::Uuid,
}

// TODO:
// Get all candidates for the current event
// Compute the scores of each candidate
// Return the final scores of each candidate

// NOTE: Will only get the final score for ONE category only
pub async fn get_candidate_scores(
    State(pool): State<PgPool>,
    Query(query): Query<ScoreQuery>,
) -> Result<axum::Json<Vec<Score>>, http::StatusCode> {
    let q = "SELECT * FROM scores WHERE candidate_id = ($1)";

    let res = sqlx::query_as::<_, Score>(q)
        .bind(&query.candidate_id)
        .fetch_all(&pool)
        .await;

    get_candidate_score(State(pool), Query(query))
        .await
        .expect("Failed to get compute candidate score.");

    match res {
        Ok(scores) => {
            // let final_category_score =
            //     get_candidate_score(State(pool), Path(candidate_id), Query(query))
            //         .await
            //         .expect("Failed to add criteria scores.");

            // println!("Weighted Category Score: {final_category_score}");

            Ok(axum::Json(scores))
        }
        Err(err) => {
            eprintln!("Failed to get candidate scores: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize, FromRow)]
pub struct CategoryWeight {
    id: uuid::Uuid,
    weight: f32,
}

pub async fn get_candidate_score(
    State(pool): State<PgPool>,
    Query(query): Query<ScoreQuery>,
) -> Result<f32, http::StatusCode> {
    let category_weights: Vec<CategoryWeight> = sqlx::query_as::<_, CategoryWeight>(
        "SELECT id, weight FROM categories WHERE event_id = ($1)",
    )
    .bind(&query.event_id)
    .fetch_all(&pool)
    .await
    .context("Failed to fetch category weights")
    .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut weighted_scores = Vec::with_capacity(category_weights.len());
    let mut weights = Vec::with_capacity(category_weights.len());

    for (i, category) in category_weights.iter().enumerate() {
        let category_scores =
            sqlx::query_scalar::<_, i32>("SELECT score FROM scores WHERE category_id = ($1)")
                .bind(category.id)
                .fetch_all(&pool)
                .await
                .context(format!("Failed to fetch scores for category {}", i + 1))
                .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

        let total_score: i32 = category_scores.iter().sum();
        let weighted_score = (total_score as f32) * category.weight;

        println!("Category {} Scores: {:?}", i + 1, category_scores);
        println!("Category {} Total Score: {}", i + 1, total_score);
        println!("Category {} Weight: {}", i + 1, category.weight);
        println!("Category {} Weighted Score: {}\n", i + 1, weighted_score);

        weighted_scores.push(weighted_score);
        weights.push(category.weight);
    }

    println!("Weighted Scores: {weighted_scores:?}");

    let weighted_scores_sum: f32 = weighted_scores.iter().sum();
    let weights_sum: f32 = weights.iter().sum();
    let final_score = weighted_scores_sum / weights_sum;

    println!("Sum of Weighted Scores: {weighted_scores_sum}");
    println!("Sum of Category Weights: {weights_sum}\n");
    println!("Final Score: {final_score}");

    Ok(final_score)
}

// Old code
// May or may not be needed, not used yet
// let category_count = sqlx::query_scalar::<_, i64>(
//     "SELECT COUNT(*) AS category_count FROM categories WHERE event_id = ($1)",
// )
// .bind(&category_query.event_id)
// .fetch_one(&pool)
// .await
// .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

// println!("Category count: {category_count}\n");

//
// // Get the scores for each criteria on each category
// // NOTE: do we need to display each criteria scores? Removing the last part of this query
// // (s.criteria_id = ($3)) will make it so that it will just get all the scores from all judges immediately
//
// let judge_scores_on_criteria = sqlx::query_scalar::<_, i32>(
//     r#"
//      SELECT s.score, s.judge_id FROM scores s JOIN criterias c ON s.criteria_id = c.id
//      WHERE s.candidate_id = ($1) AND c.category_id = ($2) AND s.criteria_id = ($3)
//      "#,
// )
// .bind(&candidate_id)
// .bind(&query.category_id)
// .bind(&query.criteria_id)
// .fetch_all(&pool)
// .await
// .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;
//
// let criteria_score: i32 = judge_scores_on_criteria.iter().sum();
//
// println!("\nCriteria Score: {criteria_score}");
// println!("Category Weight: {category_weight}\n");

// let final_category_score = (criteria_score as f32) * category_weight;
