use axum::extract::Path;
use axum::response::Result;
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    id: String,
    name: String,
}

// POST
pub async fn create_event(
    State(pool): State<PgPool>,
    Json(new_event): Json<Event>,
) -> Result<Json<Event>> {
    let query = "INSERT INTO events (id, name) VALUES ($1, $2)";

    sqlx::query(query)
        .bind(&new_event.id)
        .bind(&new_event.name)
        .execute(&(pool))
        .await
        .expect("Failed to insert event.");

    Ok(Json(new_event))
}

// GET
pub async fn get_events(State(pool): State<PgPool>) -> Result<Json<Vec<Event>>> {
    let q = "SELECT * FROM events";
    let query = sqlx::query(q);

    let rows = query
        .fetch_all(&(pool))
        .await
        .expect("Failed to fetch list of events.");

    let events: Vec<Event> = rows
        .iter()
        .map(|row| {
            let event = Event {
                id: row.get("id"),
                name: row.get("name"),
            };

            event
        })
        .collect();

    Ok(Json(events))
}

pub async fn get_event(State(pool): State<PgPool>, Path(id): Path<String>) -> Result<Json<Event>> {
    let q = "SELECT * FROM events WHERE id = ($1)";
    let query = sqlx::query(q);

    let row = query
        .bind(id)
        .fetch_one(&(pool))
        .await
        .expect("Failed to fetch event row, check if the row exists.");

    let event = Event {
        id: row.get("id"),
        name: row.get("name"),
    };

    Ok(Json(event))
}
