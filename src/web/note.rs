use axum::extract::{Query, State};
use axum::http;
use axum::response::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Serialize, FromRow)]
pub struct Note {
    id: uuid::Uuid,
    note: String,
    last_change: chrono::DateTime<chrono::Utc>,
    // Relationships
    candidate_id: uuid::Uuid,
    judge_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateNote {
    note: String,
    candidate_id: uuid::Uuid,
    judge_id: uuid::Uuid,
}

pub async fn create_note(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateNote>,
) -> Result<(http::StatusCode, axum::Json<Note>), http::StatusCode> {
    let res = sqlx::query_as::<_, Note>(
        r#"
        INSERT INTO notes (note, candidate_id, judge_id) 
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(&payload.note)
    .bind(&payload.candidate_id)
    .bind(&payload.judge_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(note) => Ok((http::StatusCode::CREATED, axum::Json(note))),
        Err(err) => {
            eprintln!("Failed to create note: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NoteQuery {
    candidate_id: uuid::Uuid,
}

pub async fn get_note(
    State(pool): State<PgPool>,
    Query(query): Query<NoteQuery>,
) -> Result<axum::Json<Vec<Note>>, http::StatusCode> {
    let res = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE candidate_id = ($1)")
        .bind(&query.candidate_id)
        .fetch_all(&pool)
        .await;

    match res {
        Ok(notes) => Ok(axum::Json(notes)),
        Err(err) => {
            eprintln!("Failed to get note: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
