use axum::response::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Debug, Deserialize, Serialize)]
pub struct Category {
    id: String,
    name: String,
    weight: f32,
    // Relationships
    event_id: String,
}

// POST
pub async fn create_category(
    State(pool): State<PgPool>,
    Path(event_id): Path<String>,
    Json(new_category): Json<Category>,
) -> Result<Json<Category>> {
    let query = "INSERT INTO categories (id, name, weight, event_id) VALUES ($1, $2, $3, $4)";

    sqlx::query(query)
        .bind(&new_category.id)
        .bind(&new_category.name)
        .bind(&new_category.weight)
        .bind(event_id)
        .execute(&(pool))
        .await
        .expect("Failed to insert category.");

    Ok(Json(new_category))
}

// GET
pub async fn get_categories(
    State(pool): State<PgPool>,
    Path(event_id): Path<String>,
) -> Result<Json<Vec<Category>>> {
    let q = "SELECT * FROM categories WHERE event_id = ($1)";
    let query = sqlx::query(q);

    let rows = query
        .bind(event_id)
        .fetch_all(&(pool))
        .await
        .expect("Failed to fetch list of categories.");

    let categories: Vec<Category> = rows
        .iter()
        .map(|row| {
            let category = Category {
                id: row.get("id"),
                name: row.get("name"),
                weight: row.get("weight"),
                event_id: row.get("event_id"),
            };

            category
        })
        .collect();

    Ok(Json(categories))
}

pub async fn get_category(
    State(pool): State<PgPool>,
    Path((event_id, category_id)): Path<(String, String)>,
) -> Result<Json<Category>> {
    let q = "SELECT * FROM categories WHERE event_id = ($1) AND id = ($2)";
    let query = sqlx::query(q);

    let row = query
        .bind(event_id)
        .bind(category_id)
        .fetch_one(&(pool))
        .await
        .expect("Failed to fetch category row, check if the row exists.");

    let category = Category {
        id: row.get("id"),
        name: row.get("name"),
        weight: row.get("weight"),
        event_id: row.get("event_id"),
    };

    Ok(Json(category))
}
