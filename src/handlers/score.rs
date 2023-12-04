use anyhow::Context;
use axum::extract::{Query, State};
use axum::http;
use axum::response::Result;
use chrono::Local;
use rust_xlsxwriter::*;
use serde::{Deserialize, Serialize};
use sqlx::query::QueryAs;
use sqlx::{FromRow, PgPool};

use crate::error::AppError;

use super::candidate::Candidate;
use super::category::Category;
use super::criteria::Criteria;
use super::event::Event;
use super::judge::Judge;
use super::Round;

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
) -> Result<(http::StatusCode, axum::Json<Score>), AppError> {
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

            Err(AppError::new(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to submit score: {}", err),
            ))
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateScore {
    score_id: uuid::Uuid,
    score: i32,
}

pub async fn update_score(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<UpdateScore>,
) -> Result<(http::StatusCode, axum::Json<Score>), AppError> {
    let res = sqlx::query_as::<_, Score>(
        r#"
        UPDATE scores SET score = ($1), time_of_scoring = ($2) 
        WHERE id = ($3) 
        RETURNING *
        "#,
    )
    .bind(&payload.score)
    .bind(Local::now())
    .bind(&payload.score_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(score) => Ok((http::StatusCode::CREATED, axum::Json(score))),
        Err(err) => {
            eprintln!("Failed to submit score: {err:?}");

            Err(AppError::new(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to submit score: {}", err),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScoreParam {
    criteria_id: uuid::Uuid,
    category_id: uuid::Uuid,
}

pub async fn get_candidate_scores(
    State(pool): State<PgPool>,
    query: Option<Query<ScoreParam>>,
) -> Result<axum::Json<Vec<Score>>, AppError> {
    let res = match query {
        Some(param) => {
            sqlx::query_as::<_, Score>(
                "SELECT * FROM scores WHERE criteria_id = ($1) or category_id = ($2)",
            )
            .bind(&param.criteria_id)
            .bind(&param.category_id)
            .fetch_all(&pool)
            .await
        }
        None => {
            sqlx::query_as::<_, Score>("SELECT * FROM scores")
                .fetch_all(&pool)
                .await
        }
    };

    match res {
        Ok(scores) => Ok(axum::Json(scores)),
        Err(err) => Err(AppError::new(
            http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get candidate scores: {}", err),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct IndivScoreParam {
    category_id: uuid::Uuid,
    candidate_id: uuid::Uuid,
}

pub async fn get_candidate_score(
    State(pool): State<PgPool>,
    query: Option<Query<IndivScoreParam>>,
) -> Result<axum::Json<Vec<Score>>, AppError> {
    let res = match query {
        Some(param) => {
            sqlx::query_as::<_, Score>(
                "SELECT * FROM scores WHERE category_id = ($1) AND candidate_id = ($2)",
            )
            .bind(&param.category_id)
            .bind(&param.candidate_id)
            .fetch_all(&pool)
            .await
        }
        None => {
            sqlx::query_as::<_, Score>("SELECT * FROM scores")
                .fetch_all(&pool)
                .await
        }
    };

    match res {
        Ok(scores) => Ok(axum::Json(scores)),
        Err(err) => Err(AppError::new(
            http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get candidate scores: {}", err),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct FinalScoreParam {
    // candidate_id: uuid::Uuid,
    // category_id: uuid::Uuid,
    event_id: uuid::Uuid,
}

// #[derive(Debug, Deserialize, FromRow)]
// pub struct Candidate {
//     id: uuid::Uuid,
//     first_name: String,
//     middle_name: String,
//     last_name: String,
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct CandidateFinalScore {
    candidate_id: uuid::Uuid,
    first_name: String,
    middle_name: String,
    last_name: String,
    final_score: f32,
}

pub async fn get_candidate_final_scores(
    State(pool): State<PgPool>,
    Query(query): Query<FinalScoreParam>,
) -> Result<axum::Json<Vec<CandidateFinalScore>>, AppError> {
    // "SELECT id, first_name, middle_name, last_name FROM candidates",
    let res = sqlx::query_as::<_, Candidate>("SELECT * FROM candidates")
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

                let final_score =
                    get_candidate_final_score(&pool, &query.event_id, &candidate.id).await?;

                candidate_final_scores.push(CandidateFinalScore {
                    candidate_id: candidate.id,
                    first_name: candidate.first_name.clone(),
                    middle_name: candidate.middle_name.clone(),
                    last_name: candidate.last_name.clone(),
                    final_score,
                });
            }

            Ok(axum::Json(candidate_final_scores))
        }
        Err(err) => {
            eprintln!("Failed to get candidates when computing scores: {err:?}");

            Err(AppError::new(
                http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get candidate scores: {}", err),
            ))
        }
    }
}

#[derive(Debug, Deserialize, FromRow)]
pub struct CategoryWeight {
    id: uuid::Uuid,
    weight: f32,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct ScoreMax {
    total_score: i64,
    total_max: i64,
    weighted_score: f64,
    weighted_max: f64,
}

// Calculate the final score to send to the client
pub async fn get_candidate_final_score(
    pool: &PgPool,
    event_id: &uuid::Uuid,
    candidate_id: &uuid::Uuid,
) -> Result<f32, AppError> {
    let scores = sqlx::query_as::<_, ScoreMax>(
        r#"
        SELECT 
            COALESCE(SUM(s.score), 0) AS total_score, 
            COALESCE(SUM(s.max), 0) AS total_max,
            COALESCE(SUM(s.score), 0) * cat.weight AS weighted_score,
            COALESCE(SUM(s.max), 0) * cat.weight AS weighted_max
        FROM 
            categories cat
        LEFT JOIN 
            scores s ON s.category_id = cat.id AND s.candidate_id = ($1)
        WHERE 
            cat.event_id = ($2)
        GROUP BY
            cat.id, cat.weight
        "#,
    )
    .bind(candidate_id)
    .bind(event_id)
    .fetch_all(pool)
    .await?;

    let mut weighted_scores_sum: f64 = 0.0;
    let mut weighted_max_sum: f64 = 0.0;

    for score in scores.iter() {
        weighted_scores_sum += score.weighted_score.round_to_two_decimals();
        weighted_max_sum += score.weighted_max.round_to_two_decimals();
    }

    let final_score = ((weighted_scores_sum / weighted_max_sum) * 100.0) as f32;

    Ok(final_score)
}

// #[derive(Debug, Deserialize, FromRow)]
// pub struct Criteria {
//     id: uuid::Uuid,
//     name: String,
// }
//
// #[derive(Debug, Deserialize, FromRow)]
// pub struct CriteriaScore {
//     score: i32,
//     judge_name: String,
//     candidate_first_name: String,
//     candidate_middle_name: String,
//     candidate_last_name: String,
//     weight: f32,
//     max: i32,
//     event_name: String,
// }

// #[derive(Debug, Deserialize, FromRow)]
// pub struct Category {
//     id: uuid::Uuid,
//     name: String,
// }

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Test {
    candidate_id: uuid::Uuid,
    category: String,
    total_score: i64,
}

// EXPERIMENTAL
pub async fn foo(
    State(pool): State<PgPool>,
) -> Result<(http::StatusCode, axum::Json<Vec<Test>>), AppError> {
    // Get all on events table
    // Each category per event
    // Each criteria per category
    // Get all on candidates table
    // Get all on judges table
    // Get all on scores table
    //
    // THINGS TO WRITE:
    // Event Names
    // Category Names
    // Criteria Names
    // Candidate Names
    // Judge Names
    // Canddate Scores
    let res = sqlx::query_as::<_, Test>(
        r#"
        "#,
    )
    .fetch_all(&pool)
    .await?;

    println!("{:?}", res);

    Ok((http::StatusCode::OK, axum::Json(res)))
}

pub async fn generate_score_spreadsheet(
    State(pool): State<PgPool>,
) -> Result<(http::StatusCode, Vec<u8>), AppError> {
    let res = sqlx::query_as::<_, Category>("SELECT * FROM categories")
        .fetch_all(&pool)
        .await;

    match res {
        Ok(categories) => {
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();

            let heading_format = Format::new().set_font_size(13.5).set_bold();
            let bold_format = Format::new().set_bold().set_align(FormatAlign::Center);

            worksheet.set_column_width(0, 15)?;
            worksheet.set_column_width(1, 30)?;

            let mut row_offset: u32 = 0;

            let candidates = sqlx::query_as::<_, Candidate>(
                r#"
                    SELECT * FROM candidates 
                    ORDER BY 
                        CASE
                            WHEN gender = 1 THEN 1
                            ELSE 2
                        END,
                        candidate_number
                    "#,
            )
            .fetch_all(&pool)
            .await?;

            // Could use the Rayon crate for parallelization, but no need
            let (male_candidates, female_candidates): (Vec<&Candidate>, Vec<&Candidate>) =
                candidates
                    .iter()
                    .partition(|candidate| candidate.gender == 1);

            for (category_idx, category) in categories.iter().enumerate() {
                // if category_idx == 1 {
                //     break;
                // }

                worksheet.merge_range(
                    row_offset,
                    0,
                    row_offset,
                    6,
                    category.name.as_str(),
                    &heading_format,
                )?;

                worksheet.write_with_format(1 + row_offset, 0, "Candidate #", &bold_format)?;

                worksheet.write_with_format(1 + row_offset, 1, "Name", &bold_format)?;

                // Could be improved, it's not necessary to fetch the same judges on the same
                // event_id
                // Could use a Hashmap wherein the event_id is they key and the vector of judges
                // are the values
                let judges =
                    sqlx::query_as::<_, Judge>("SELECT * FROM judges WHERE event_id = ($1)")
                        .bind(&category.event_id)
                        .fetch_all(&pool)
                        .await?;

                // Write judge names
                for (i, judge) in judges.iter().enumerate() {
                    worksheet.set_column_width(i as u16 + 2, 30)?;
                    worksheet.write_with_format(
                        1 + row_offset,
                        i as u16 + 2,
                        &judge.name,
                        &bold_format,
                    )?;
                }

                worksheet.set_column_width(judges.len() as u16 + 2, 20)?;
                worksheet.set_column_width(judges.len() as u16 + 3, 15)?;

                worksheet.write_with_format(
                    1 + row_offset,
                    judges.len() as u16 + 2,
                    "Average Score",
                    &bold_format,
                )?;

                worksheet.write_with_format(
                    1 + row_offset,
                    judges.len() as u16 + 3,
                    format!("{}%", category.weight * 100.0),
                    &bold_format,
                )?;

                worksheet.write(row_offset + 2, 0, "MALE")?;

                // Write scores for male candidates
                write_scores(
                    &pool,
                    worksheet,
                    &male_candidates,
                    category,
                    &judges,
                    3 + row_offset,
                    0,
                )
                .await?;

                worksheet.write(row_offset + 3 + male_candidates.len() as u32, 0, "FEMALE")?;

                // Write scores for female candidates
                write_scores(
                    &pool,
                    worksheet,
                    &female_candidates,
                    category,
                    &judges,
                    row_offset + 4 + male_candidates.len() as u32,
                    0,
                )
                .await?;

                row_offset += candidates.len() as u32 + 5;
            }

            let workbook_buffer = workbook.save_to_buffer()?;

            Ok((http::StatusCode::OK, workbook_buffer))
        }
        Err(err) => Err(AppError::new(
            http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get categories: {}", err),
        )),
    }
}

async fn write_scores(
    pool: &PgPool,
    worksheet: &mut Worksheet,
    candidates: &Vec<&Candidate>,
    category: &Category,
    judges: &Vec<Judge>,
    row: RowNum,
    col: ColNum,
) -> Result<(), AppError> {
    for (candidate_idx, candidate) in candidates.iter().enumerate() {
        // Write candidate numbers
        worksheet.write(
            candidate_idx as u32 + row,
            0 + col,
            candidate.candidate_number,
        )?;

        // Write candidate names
        worksheet.write(
            candidate_idx as u32 + row,
            1 + col,
            format!(
                "{}, {} {}",
                candidate.last_name, candidate.first_name, candidate.middle_name
            ),
        )?;

        let mut total_score: f32 = 0.0;

        // Write candidate scores
        for (judge_idx, judge) in judges.iter().enumerate() {
            // Could be improved
            // Use SQL to get the sum instead
            let scores = sqlx::query_as::<_, Score>(
                "SELECT * FROM scores WHERE candidate_id = ($1) AND category_id = ($2) AND judge_id = ($3)",
            )
            .bind(candidate.id)
            .bind(category.id)
            .bind(judge.id)
            .fetch_all(pool)
            .await?;

            let total_score_for_judge: i32 =
                scores.into_iter().fold(0, |acc, score| acc + score.score);

            total_score += total_score_for_judge as f32;

            worksheet.write(
                row + candidate_idx as u32,
                col + 2 + judge_idx as u16,
                total_score_for_judge,
            )?;
        }

        let average_score: f32 = total_score / judges.len() as f32;
        let score_in_percentage: f32 = average_score * category.weight;

        // Write candidate average scores
        worksheet.write(
            row + candidate_idx as u32,
            col + 2 + judges.len() as u16,
            format!("{:.2}", average_score),
        )?;

        // Write candidate average scores in percentage
        worksheet.write(
            row + candidate_idx as u32,
            col + 3 + judges.len() as u16,
            format!("{:.2}", score_in_percentage),
        )?;
    }

    Ok(())
}

// OLD CODE
// FOR GENERATING CSV SPREADSHEET

// Generates a spreadsheet for the scoring system for the sake of transparency
// pub async fn generate_score_spreadsheet(
//     State(pool): State<PgPool>,
// ) -> Result<(http::StatusCode, Vec<u8>), AppError> {
//     let res = sqlx::query_as::<_, Category>("SELECT id, name, weight FROM categories")
//         .fetch_all(&pool)
//         .await;
//
//     match res {
//         Ok(categories) => {
//             let mut csv_writer = csv::Writer::from_writer(Vec::new());
//
//             let headers = [
//                 "Event",
//                 "Category",
//                 "Criteria",
//                 "Candidate First Name",
//                 "Candidate Middle Name",
//                 "Candidate Last Name",
//                 "Judge",
//                 "Score",
//                 "Max",
//                 "Weight",
//             ];
//
//             csv_writer.write_record(&headers).map_err(|err| {
//                 AppError::new(
//                     http::StatusCode::INTERNAL_SERVER_ERROR,
//                     format!("Failed to write record for headers: {}", err),
//                 )
//             })?;
//
//             for category in categories.iter() {
//                 let criterias = sqlx::query_as::<_, Criteria>(
//                     "SELECT id, name FROM criterias WHERE category_id = $1",
//                 )
//                 .bind(category.id)
//                 .fetch_all(&pool)
//                 .await
//                 .map_err(|err| {
//                     AppError::new(
//                         http::StatusCode::INTERNAL_SERVER_ERROR,
//                         format!("Failed to get criterias: {}", err),
//                     )
//                 })?;
//
//                 for criteria in criterias.iter() {
//                     let scores = sqlx::query_as::<_, CriteriaScore>(
//                         r#"
//                         SELECT s.score, s.max, j.name as judge_name,
//                             can.first_name as candidate_first_name,
//                             can.middle_name as candidate_middle_name,
//                             can.last_name as candidate_last_name,
//                             cat.weight as weight,
//                             e.name as event_name
//                         FROM scores s
//                         JOIN judges j ON j.id = s.judge_id
//                         JOIN candidates can ON can.id = s.candidate_id
//                         JOIN categories cat ON cat.id = s.category_id
//                         JOIN events e ON e.id = cat.event_id
//                         WHERE s.category_id = ($1) AND s.criteria_id = ($2)
//                         "#,
//                     )
//                     .bind(category.id)
//                     .bind(criteria.id)
//                     .fetch_all(&pool)
//                     .await
//                     .map_err(|err| {
//                         AppError::new(
//                             http::StatusCode::INTERNAL_SERVER_ERROR,
//                             format!("Failed to get scores: {}", err),
//                         )
//                     })?;
//
//                     for score in scores.iter() {
//                         csv_writer
//                             .write_record(vec![
//                                 &score.event_name,
//                                 &category.name,
//                                 &criteria.name,
//                                 &score.candidate_first_name,
//                                 &score.candidate_middle_name,
//                                 &score.candidate_last_name,
//                                 &score.judge_name,
//                                 &score.score.to_string(),
//                                 &score.max.to_string(),
//                                 &score.weight.to_string(),
//                             ])
//                             .map_err(|err| {
//                                 AppError::new(
//                                     http::StatusCode::INTERNAL_SERVER_ERROR,
//                                     format!("Failed to serialize record: {}", err),
//                                 )
//                             })?;
//                     }
//                 }
//             }
//
//             let csv_bytes = csv_writer.into_inner().map_err(|err| {
//                 AppError::new(
//                     http::StatusCode::INTERNAL_SERVER_ERROR,
//                     format! {"Failed to generate CSV file: {}", err},
//                 )
//             })?;
//
//             Ok((http::StatusCode::OK, csv_bytes))
//         }
//         Err(err) => Err(AppError::new(
//             http::StatusCode::INTERNAL_SERVER_ERROR,
//             format!("Failed to get categories: {}", err),
//         )),
//     }
// }
//

// let judges =
//     sqlx::query_as::<_, Judge>("SELECT * FROM judges WHERE event_id = ($1)")
//         .bind(&category.event_id)
//         .fetch_all(&pool)
//         .await
//         .map_err(|err| {
//             AppError::new(
//                 http::StatusCode::INTERNAL_SERVER_ERROR,
//                 format!("Failed to get judges: {}", err),
//             )
//         })?;
//
// let judge_names: Vec<String> = judges.into_iter().map(|judge| judge.name).collect();
//
// let headers = vec![
//     vec!["Candidate #".to_string(), "Name".to_string()],
//     judge_names,
//     vec![
//         "Average Score".to_string(),
//         format!("{}%", category.weight * 100.0),
//     ],
// ];
//
// let flattened_headers: Vec<String> = headers.into_iter().flatten().collect();
//
// // Write headers from Candidate # to Average Score %
// flattened_headers
//     .iter()
//     .enumerate()
//     .for_each(|(i, header)| {
//         worksheet
//             .write((category_idx + 1) as u32, i as u16, header)
//             .context("Failed to write headers.")
//             .unwrap();
//     });
