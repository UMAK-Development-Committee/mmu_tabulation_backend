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
    judge_id: uuid::Uuid,
}

impl Score {
    fn new(score: CreateScore) -> Self {
        let now = chrono::Utc::now();
        let uuid = uuid::Uuid::new_v4();

        Self {
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
    judge_id: uuid::Uuid,
}

// Submit score function for each individual judge
pub async fn submit_score(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateScore>,
) -> Result<(http::StatusCode, axum::Json<Score>), http::StatusCode> {
    let query = "INSERT INTO scores (id, score, max, time_of_scoring, candidate_id, criteria_id, judge_id) VALUES ($1, $2, $3, $4, $5, $6, $7)";

    let score = Score::new(payload);

    let res = sqlx::query(query)
        .bind(&score.id)
        .bind(&score.score)
        .bind(&score.max)
        .bind(&score.time_of_scoring)
        .bind(&score.candidate_id)
        .bind(&score.criteria_id)
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
    event_id: uuid::Uuid,
    category_id: uuid::Uuid,
    criteria_id: uuid::Uuid,
}

// NOTE: Will only get the final score for ONE category only
pub async fn get_candidate_scores(
    State(pool): State<PgPool>,
    Path(candidate_id): Path<uuid::Uuid>,
    Query(query): Query<ScoreQuery>,
) -> Result<axum::Json<Vec<Score>>, http::StatusCode> {
    let q = "SELECT * FROM scores WHERE candidate_id = ($1)";

    let res = sqlx::query_as::<_, Score>(q)
        .bind(&candidate_id)
        .fetch_all(&pool)
        .await;

    match res {
        Ok(scores) => {
            let final_category_score =
                get_category_score(State(pool), Path(candidate_id), Query(query))
                    .await
                    .expect("Failed to add criteria scores.");

            println!("Category score: {final_category_score}");

            Ok(axum::Json(scores))
        }
        Err(err) => {
            eprintln!("Failed to get candidate scores: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// NOTE:
// Formula: Summation of total category score * weight
pub async fn get_category_score(
    State(pool): State<PgPool>,
    Path(candidate_id): Path<uuid::Uuid>,
    Query(category_query): Query<ScoreQuery>,
) -> Result<f32, http::StatusCode> {
    // May or may not be needed, not used yet
    let category_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) AS category_count FROM categories WHERE event_id = ($1)",
    )
    .bind(&category_query.event_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("Category count: {category_count}\n");

    // Get category weight
    let category_weight = sqlx::query_scalar::<_, f32>(
        "SELECT weight from categories WHERE event_id = ($1) AND id = ($2)",
    )
    .bind(&category_query.event_id)
    .bind(&category_query.category_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get the scores for each criteria on each category
    // NOTE: do we need to display each criteria scores? Removing the last part of this query
    // (s.criteria_id = ($3)) will make it so that it will just get all the scores from all judges immediately

    let criteria_scores = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT s.score, s.judge_id FROM scores s JOIN criterias c ON s.criteria_id = c.id 
        WHERE s.candidate_id = ($1) AND c.category_id = ($2) AND s.criteria_id = ($3)
        "#,
    )
    .bind(&candidate_id)
    .bind(&category_query.category_id)
    .bind(&category_query.criteria_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let category_score: i32 = criteria_scores.iter().sum();

    println!("\nTotal score: {category_score}");
    println!("Category weight: {category_weight}\n");

    let final_category_score = (category_score as f32) * category_weight;

    Ok(final_category_score)
}