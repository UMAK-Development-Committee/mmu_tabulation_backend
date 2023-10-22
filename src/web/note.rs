use axum::extract::{Path, Query};
use axum::http;
use axum::response::Result;
use axum::Form;
use axum::{extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};

#[derive(Debug, Serialize, FromRow)]
pub struct Note {
    id: uuid::Uuid,
    note: String,
    last_change: chrono::DateTime<chrono::Utc>,
    // Relationships
    candidate_id: uuid::Uuid,
    judge_id: String,
}

impl Note {
    fn new(note: String, candidate_id: uuid::Uuid, judge_id: String) -> Self {
        let now = chrono::Utc::now();
        let uuid = uuid::Uuid::new_v4();

        Self {
            judge_id,
            candidate_id,
            note,
            id: uuid,
            last_change: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateNote {
    note: String,
    candidate_id: uuid::Uuid,
    judge_id: String,
}

pub async fn create_note(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateNote>,
) -> Result<(http::StatusCode, axum::Json<Note>), http::StatusCode> {
    let query = "INSERT INTO notes (id, note, last_change, candidate_id, judge_id) VALUES ($1, $2, $3, $4, $5)";

    let note = Note::new(payload.note, payload.candidate_id, payload.judge_id);

    let res = sqlx::query(query)
        .bind(&note.id)
        .bind(&note.note)
        .bind(&note.last_change)
        .bind(&note.candidate_id)
        .bind(&note.judge_id)
        .execute(&pool)
        .await;

    match res {
        Ok(_) => Ok((http::StatusCode::CREATED, axum::Json(note))),
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
