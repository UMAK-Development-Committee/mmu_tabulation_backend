use anyhow::Context;
use axum::extract::{Query, State};
use axum::http;
use axum::response::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

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
    let res = sqlx::query_as::<_, Score>(
        r#"
        INSERT INTO scores (score, max, candidate_id, criteria_id, category_id, judge_id) 
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&payload.score)
    .bind(&payload.max)
    .bind(&payload.candidate_id)
    .bind(&payload.criteria_id)
    .bind(&payload.category_id)
    .bind(&payload.judge_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(score) => Ok((http::StatusCode::CREATED, axum::Json(score))),
        Err(err) => {
            eprintln!("Failed to submit score: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScoreQuery {
    // candidate_id: uuid::Uuid,
    // category_id: uuid::Uuid,
    event_id: uuid::Uuid,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct Candidate {
    id: uuid::Uuid,
    first_name: String,
    middle_name: String,
    last_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CandidateFinalScore {
    candidate_id: uuid::Uuid,
    final_score: f32,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct ScoreMax {
    score: i32,
    max: i32,
}

pub async fn get_candidate_scores(
    State(pool): State<PgPool>,
    Query(query): Query<ScoreQuery>,
) -> Result<axum::Json<Vec<CandidateFinalScore>>, http::StatusCode> {
    let res = sqlx::query_as::<_, Candidate>(
        "SELECT id, first_name, middle_name, last_name FROM candidates",
    )
    .fetch_all(&pool)
    .await;

    match res {
        Ok(candidates) => {
            let mut candidate_final_scores: Vec<CandidateFinalScore> =
                Vec::with_capacity(candidates.len());

            for candidate in candidates.iter() {
                println!(
                    "Candidate: {} {} {}",
                    candidate.first_name, candidate.middle_name, candidate.last_name
                );

                let final_score = get_candidate_score(&pool, &query.event_id, &candidate.id)
                    .await
                    .expect("Failed to get compute candidate score.");

                candidate_final_scores.push(CandidateFinalScore {
                    candidate_id: candidate.id,
                    final_score,
                });
            }

            Ok(axum::Json(candidate_final_scores))
        }
        Err(err) => {
            eprintln!("Failed to get candidates when computing scores: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize, FromRow)]
pub struct CategoryWeight {
    id: uuid::Uuid,
    weight: f32,
}

// Calculate the final score to send to the client
pub async fn get_candidate_score(
    pool: &PgPool,
    event_id: &uuid::Uuid,
    candidate_id: &uuid::Uuid,
) -> Result<f32, http::StatusCode> {
    let category_weights: Vec<CategoryWeight> = sqlx::query_as::<_, CategoryWeight>(
        "SELECT id, weight FROM categories WHERE event_id = ($1)",
    )
    .bind(&event_id)
    .fetch_all(pool)
    .await
    .context("Failed to fetch category weights")
    .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut weighted_scores = Vec::with_capacity(category_weights.len());
    let mut weighted_max_scores = Vec::with_capacity(category_weights.len());
    let mut weights = Vec::with_capacity(category_weights.len());

    for (i, category) in category_weights.iter().enumerate() {
        let category_scores = sqlx::query_as::<_, ScoreMax>(
            "SELECT score, max FROM scores WHERE category_id = ($1) AND candidate_id = ($2)",
        )
        .bind(category.id)
        .bind(candidate_id)
        .fetch_all(pool)
        .await
        .context(format!("Failed to fetch scores for category {}", i + 1))
        .map_err(|_| http::StatusCode::INTERNAL_SERVER_ERROR)?;

        let total_score: i32 = category_scores
            .iter()
            .fold(0, |acc, score| acc + score.score);

        let total_max_score: i32 = category_scores.iter().fold(0, |acc, score| acc + score.max);

        let weighted_score = (total_score as f32) * category.weight;
        let weighted_max_score = (total_max_score as f32) * category.weight;

        println!("Category {} Scores: {:?}", i + 1, category_scores);
        println!("Category {} Total Score: {}", i + 1, total_score);
        println!("Category {} Max Score: {}", i + 1, total_max_score);
        println!("Category {} Weight: {}", i + 1, category.weight);
        println!("Category {} Weighted Score: {}\n", i + 1, weighted_score);
        println!(
            "Category {} Weighted Max Score: {}\n",
            i + 1,
            weighted_max_score
        );

        weighted_scores.push(weighted_score);
        weighted_max_scores.push(weighted_max_score);
        weights.push(category.weight);
    }

    println!("Weighted Scores: {weighted_scores:?}");
    println!("Weighted Max Scores: {weighted_max_scores:?}");

    let weighted_scores_sum: f32 = weighted_scores.iter().sum();
    let weighted_max_scores_sum: f32 = weighted_max_scores.iter().sum();
    // let weights_sum: f32 = weights.iter().sum();
    let final_score = (weighted_scores_sum / weighted_max_scores_sum) * 100.0;

    println!("Sum of Weighted Scores: {weighted_scores_sum}");
    println!("Sum of Weighted Max Scores: {weighted_max_scores_sum}");
    println!("Final Score: {final_score}\n");

    Ok(final_score)
}

#[derive(Debug, Deserialize, FromRow)]
pub struct Criteria {
    id: uuid::Uuid,
    name: String,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct CriteriaScore {
    score: i32,
    judge_name: String,
    candidate_first_name: String,
    candidate_middle_name: String,
    candidate_last_name: String,
    weight: f32,
    max: i32,
    event_name: String,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct Category {
    id: uuid::Uuid,
    name: String,
}

// Generates a spreadsheet for the scoring system for the sake of transparency
pub async fn generate_score_spreadsheet(
    State(pool): State<PgPool>,
) -> Result<(http::StatusCode, Vec<u8>), http::StatusCode> {
    let categories = sqlx::query_as::<_, Category>("SELECT id, name, weight FROM categories")
        .fetch_all(&pool)
        .await
        .map_err(|err| {
            eprintln!("Failed to get categories: {:?}", err);
            http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut csv_writer = csv::Writer::from_writer(Vec::new());

    let headers = [
        "Event",
        "Category",
        "Criteria",
        "Candidate First Name",
        "Candidate Middle Name",
        "Candidate Last Name",
        "Judge",
        "Score",
        "Max",
        "Weight",
    ];

    csv_writer.write_record(&headers).map_err(|err| {
        eprintln!("Failed to write record for headers: {:?}", err);
        http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    for category in categories.iter() {
        let criterias =
            sqlx::query_as::<_, Criteria>("SELECT id, name FROM criterias WHERE category_id = $1")
                .bind(category.id)
                .fetch_all(&pool)
                .await
                .map_err(|err| {
                    eprintln!("Failed to get criterias: {:?}", err);
                    http::StatusCode::INTERNAL_SERVER_ERROR
                })?;

        for criteria in criterias.iter() {
            let scores = sqlx::query_as::<_, CriteriaScore>(
                r#"
                SELECT s.score, s.max, j.name as judge_name, 
                    can.first_name as candidate_first_name, 
                    can.middle_name as candidate_middle_name, 
                    can.last_name as candidate_last_name,
                    cat.weight as weight,
                    e.name as event_name
                FROM scores s
                JOIN judges j ON j.id = s.judge_id
                JOIN candidates can ON can.id = s.candidate_id
                JOIN categories cat ON cat.id = s.category_id
                JOIN events e ON e.id = cat.event_id
                WHERE s.category_id = ($1) AND s.criteria_id = ($2)
                "#,
            )
            .bind(category.id)
            .bind(criteria.id)
            .fetch_all(&pool)
            .await
            .map_err(|err| {
                eprintln!("Failed to get scores: {:?}", err);
                http::StatusCode::INTERNAL_SERVER_ERROR
            })?;

            for score in scores.iter() {
                csv_writer
                    .write_record(vec![
                        &score.event_name,
                        &category.name,
                        &criteria.name,
                        &score.candidate_first_name,
                        &score.candidate_middle_name,
                        &score.candidate_last_name,
                        &score.judge_name,
                        &score.score.to_string(),
                        &score.max.to_string(),
                        &score.weight.to_string(),
                    ])
                    .map_err(|err| {
                        eprintln!("Failed to serialize record: {:?}", err);
                        http::StatusCode::INTERNAL_SERVER_ERROR
                    })?;
            }
        }
    }

    let csv_bytes = csv_writer.into_inner();

    match csv_bytes {
        Ok(csv) => Ok((http::StatusCode::OK, csv)),
        Err(err) => {
            eprintln!("Failed to generate CSV file: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
