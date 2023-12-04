use axum::extract::Path;
use axum::response::Result;
use axum::{extract::State, http};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Serialize, FromRow)]
pub struct Event {
    id: uuid::Uuid,
    name: String,
    active_event: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateEvent {
    name: String,
}

pub async fn create_event(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateEvent>,
) -> Result<(http::StatusCode, axum::Json<Event>), http::StatusCode> {
    let res = sqlx::query_as::<_, Event>("INSERT INTO events (name) VALUES ($1) RETURNING *")
        .bind(&payload.name)
        .fetch_one(&pool)
        .await;

    match res {
        Ok(event) => Ok((http::StatusCode::CREATED, axum::Json(event))),
        Err(err) => {
            eprintln!("Failed to create event: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_events(
    State(pool): State<PgPool>,
) -> Result<axum::Json<Vec<Event>>, http::StatusCode> {
    let res = sqlx::query_as::<_, Event>("SELECT * FROM events")
        .fetch_all(&pool)
        .await;

    match res {
        Ok(events) => Ok(axum::Json(events)),
        Err(err) => {
            eprintln!("Failed to get events: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_event(
    State(pool): State<PgPool>,
    Path(id): Path<uuid::Uuid>,
) -> Result<axum::Json<Event>, http::StatusCode> {
    let res = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = ($1)")
        .bind(&id)
        .fetch_one(&pool)
        .await;

    match res {
        Ok(event) => Ok(axum::Json(event)),
        Err(err) => {
            eprintln!("Failed to get event: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
