use axum::{extract, http, response::Result};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::error::AppError;

#[derive(Debug, Serialize, FromRow)]
pub struct Category {
    pub id: uuid::Uuid,
    pub name: String,
    pub weight: f32,
    // Relationships
    pub event_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategory {
    name: String,
    weight: f32,
}

pub async fn create_category(
    extract::State(pool): extract::State<PgPool>,
    extract::Path(event_id): extract::Path<uuid::Uuid>,
    axum::Json(payload): axum::Json<CreateCategory>,
) -> Result<(http::StatusCode, axum::Json<Category>), AppError> {
    let category = sqlx::query_as::<_, Category>(
        r#"
        INSERT INTO categories (name, weight, event_id) 
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.weight)
    .bind(&event_id)
    .fetch_one(&pool)
    .await?;

    Ok((http::StatusCode::CREATED, axum::Json(category)))
}

pub async fn get_categories(
    extract::State(pool): extract::State<PgPool>,
    extract::Path(event_id): extract::Path<uuid::Uuid>,
) -> Result<axum::Json<Vec<Category>>, AppError> {
    let categories =
        sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE event_id = ($1)")
            .bind(&event_id)
            .fetch_all(&pool)
            .await?;

    Ok(axum::Json(categories))
}

pub async fn get_category(
    extract::State(pool): extract::State<PgPool>,
    extract::Path((event_id, category_id)): extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::Json<Category>, AppError> {
    let category = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE event_id = ($1) AND id = ($2)",
    )
    .bind(event_id)
    .bind(category_id)
    .fetch_one(&pool)
    .await?;

    Ok(axum::Json(category))
}
