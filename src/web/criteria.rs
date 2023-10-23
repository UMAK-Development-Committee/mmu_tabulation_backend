use axum::response::Result;
use axum::{
    extract::{Path, State},
    http,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};

#[derive(Debug, Serialize, FromRow)]
pub struct Criteria {
    id: uuid::Uuid,
    name: String,
    description: String,
    max_score: i32,
    weight: f64,
    // Relationships
    category_id: uuid::Uuid,
}

impl Criteria {
    fn new(create: CreateCriteria) -> Self {
        let uuid = uuid::Uuid::new_v4();

        Self {
            category_id: create.category_id,
            weight: create.weight,
            max_score: create.max_score,
            name: create.name,
            description: create.description,
            id: uuid,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateCriteria {
    name: String,
    description: String,
    max_score: i32,
    weight: f64,
    category_id: uuid::Uuid,
}

// POST
pub async fn create_criteria(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateCriteria>,
) -> Result<(http::StatusCode, axum::Json<Criteria>), http::StatusCode> {
    let criteria = Criteria::new(payload);

    let res = sqlx::query(
        r#"
        INSERT INTO criterias (id, name, description, max_score, weight, category_id) 
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&criteria.id)
    .bind(&criteria.name)
    .bind(&criteria.description)
    .bind(&criteria.max_score)
    .bind(&criteria.weight)
    .bind(&criteria.category_id)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => Ok((http::StatusCode::CREATED, axum::Json(criteria))),
        Err(err) => {
            eprintln!("Failed to create criteria: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// GET
pub async fn get_criterias(
    State(pool): State<PgPool>,
    Path((_event_id, category_id)): Path<(uuid::Uuid, uuid::Uuid)>,
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
    State(pool): State<PgPool>,
    Path((_event_id, category_id, criteria_id)): Path<(uuid::Uuid, uuid::Uuid, uuid::Uuid)>,
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
