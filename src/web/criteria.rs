use axum::response::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Debug, Deserialize, Serialize)]
pub struct Criteria {
    id: String,
    name: String,
    description: String,
    max_score: i32,
    weight: f64,
    // Relationships
    category_id: String,
}

// POST
pub async fn create_criteria(
    State(pool): State<PgPool>,
    Json(new_criteria): Json<Criteria>,
) -> Result<Json<Criteria>> {
    let res = sqlx::query(
        r#"
        INSERT INTO criterias (id, name, description, max_score, weight, category_id) 
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&new_criteria.id)
    .bind(&new_criteria.name)
    .bind(&new_criteria.description)
    .bind(&new_criteria.max_score)
    .bind(&new_criteria.weight)
    .bind(&new_criteria.category_id)
    .execute(&pool)
    .await
    .unwrap();

    Ok(Json(new_criteria))
}

// GET
pub async fn get_criterias(
    State(pool): State<PgPool>,
    Path((_event_id, category_id)): Path<(String, String)>,
) -> Result<Json<Vec<Criteria>>> {
    let q = "SELECT * FROM criterias WHERE category_id = ($1)";

    let rows = sqlx::query(q)
        .bind(&category_id)
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch criteria, check if it exists.");

    let criterias: Vec<Criteria> = rows
        .iter()
        .map(|row| {
            let criteria = Criteria {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                max_score: row.get("max_score"),
                weight: row.get("weight"),
                category_id: row.get("category_id"),
            };

            criteria
        })
        .collect();

    Ok(Json(criterias))
}

pub async fn get_criteria(
    State(pool): State<PgPool>,
    Path((_event_id, category_id, criteria_id)): Path<(String, String, String)>,
) -> Result<Json<Criteria>> {
    let q = "SELECT * FROM criterias WHERE category_id = ($1) AND id = ($2)";

    let row = sqlx::query(q)
        .bind(&category_id)
        .bind(&criteria_id)
        .fetch_one(&(pool))
        .await
        .expect("Failed to fetch criteria, check if it exists.");

    let criteria = Criteria {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        max_score: row.get("max_score"),
        weight: row.get("weight"),
        category_id: row.get("category_id"),
    };

    Ok(Json(criteria))
}
