use axum::response::Result;
use axum::{extract, http};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::error::AppError;

#[derive(Debug, Serialize, FromRow)]
pub struct Criteria {
    id: uuid::Uuid,
    name: String,
    max_score: i32,
    // Relationships
    category_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateCriteria {
    name: String,
    max_score: i32,
}

// POST
pub async fn create_criteria(
    extract::State(pool): extract::State<PgPool>,
    extract::Path((_event_id, category_id)): extract::Path<(uuid::Uuid, uuid::Uuid)>,
    axum::Json(payload): axum::Json<CreateCriteria>,
) -> Result<(http::StatusCode, axum::Json<Criteria>), AppError> {
    let res = sqlx::query_as::<_, Criteria>(
        r#"
        INSERT INTO criterias (name, description, max_score, category_id) 
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.max_score)
    .bind(&category_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(criteria) => Ok((http::StatusCode::CREATED, axum::Json(criteria))),
        Err(err) => Err(AppError::new(
            http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create criteria: {}", err),
        )),
    }
}

pub async fn get_criterias(
    extract::State(pool): extract::State<PgPool>,
    extract::Path((_event_id, category_id)): extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::Json<Vec<Criteria>>, http::StatusCode> {
    let res = sqlx::query_as::<_, Criteria>("SELECT * FROM criterias WHERE category_id = ($1)")
        .bind(&category_id)
        .fetch_all(&pool)
        .await;

    match res {
        Ok(criterias) => Ok(axum::Json(criterias)),
        Err(err) => {
            eprintln!("Failed to get criterias: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_criteria(
    extract::State(pool): extract::State<PgPool>,
    extract::Path((_event_id, category_id, criteria_id)): extract::Path<(
        uuid::Uuid,
        uuid::Uuid,
        uuid::Uuid,
    )>,
) -> Result<axum::Json<Criteria>, http::StatusCode> {
    let res = sqlx::query_as::<_, Criteria>(
        "SELECT * FROM criterias WHERE category_id = ($1) AND id = ($2)",
    )
    .bind(&category_id)
    .bind(&criteria_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(criteria) => Ok(axum::Json(criteria)),
        Err(err) => {
            eprintln!("Failed to get criteria: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
