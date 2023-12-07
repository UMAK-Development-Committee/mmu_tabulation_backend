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
    pub is_active: bool,
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
    let res = sqlx::query_as::<_, Category>(
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
    .await;

    match res {
        Ok(category) => Ok((http::StatusCode::CREATED, axum::Json(category))),
        Err(err) => Err(AppError::new(
            http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create category: {}", err),
        )),
    }
}

pub async fn get_categories(
    extract::State(pool): extract::State<PgPool>,
    extract::Path(event_id): extract::Path<uuid::Uuid>
) -> Result<axum::Json<Vec<Category>>, http::StatusCode> {
    let res = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE event_id = ($1)")
        .bind(&event_id)
        .fetch_all(&pool)
        .await;

    match res {
        Ok(categories) => Ok(axum::Json(categories)),
        Err(err) => {
            eprintln!("Failed to get categories: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategory {
    category_id: uuid::Uuid
}

pub async fn update_category(
    extract::State(pool): extract::State<PgPool>,
    extract::Path((event_id)): extract::Path<(uuid::Uuid)>,
    extract::Query((payload)): extract::Query<(UpdateCategory)>,
) -> Result<axum::Json<Category>, http::StatusCode> {
    let res = sqlx::query_as::<_, Category>(
        r#"
        UPDATE categories
        SET is_active = CASE
            WHEN id = $1 and event_id = $2 THEN TRUE
            ELSE FALSE
        END
        RETURNING *;
        "#,
    )
    .bind(&payload.category_id)
    .bind(&event_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(category) => Ok(axum::Json(category)),
        Err(err) => {
            eprintln!("Failed to update categories: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_category(
    extract::State(pool): extract::State<PgPool>,
    extract::Path((event_id, category_id)): extract::Path<(uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::Json<Category>, http::StatusCode> {
    let res = sqlx::query_as::<_, Category>(
        "SELECT * FROM categories WHERE event_id = ($1) AND id = ($2)",
    )
    .bind(event_id)
    .bind(category_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(category) => Ok(axum::Json(category)),
        Err(err) => {
            eprintln!("Failed to get categories: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
