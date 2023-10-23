use axum::{
    extract::{Path, State},
    http,
    response::Result,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Serialize, FromRow)]
pub struct Category {
    id: uuid::Uuid,
    name: String,
    weight: f32,
    // Relationships
    event_id: uuid::Uuid,
}

impl Category {
    fn new(create: CreateCategory) -> Self {
        let uuid = uuid::Uuid::new_v4();

        Self {
            id: uuid,
            event_id: create.event_id,
            weight: create.weight,
            name: create.name,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCategory {
    name: String,
    weight: f32,
    event_id: uuid::Uuid,
}

pub async fn create_category(
    State(pool): State<PgPool>,
    Path(event_id): Path<uuid::Uuid>,
    axum::Json(payload): axum::Json<CreateCategory>,
) -> Result<(http::StatusCode, axum::Json<Category>), http::StatusCode> {
    let category = Category::new(CreateCategory {
        name: payload.name,
        weight: payload.weight,
        event_id,
    });

    let res =
        sqlx::query("INSERT INTO categories (id, name, weight, event_id) VALUES ($1, $2, $3, $4)")
            .bind(&category.id)
            .bind(&category.name)
            .bind(&category.weight)
            .bind(&event_id)
            .execute(&pool)
            .await;

    match res {
        Ok(_) => Ok((http::StatusCode::CREATED, axum::Json(category))),
        Err(err) => {
            eprintln!("Failed to create category: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_categories(
    State(pool): State<PgPool>,
    Path(event_id): Path<uuid::Uuid>,
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

pub async fn get_category(
    State(pool): State<PgPool>,
    Path((event_id, category_id)): Path<(uuid::Uuid, uuid::Uuid)>,
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
