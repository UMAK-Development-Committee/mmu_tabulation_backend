use axum::response::Result;
use axum::{extract::State, http};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Serialize, FromRow)]
pub struct Judge {
    pub id: uuid::Uuid,
    pub name: String,
    pub password: String,
    pub is_active: bool,
    // Relationships
    pub event_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateJudge {
    name: String,
    password: String,
    is_active: bool,
    event_id: uuid::Uuid,
}

pub async fn create_judge(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateJudge>,
) -> Result<(http::StatusCode, axum::Json<Judge>), http::StatusCode> {
    let res = sqlx::query_as::<_, Judge>(
        r#"
        INSERT INTO judges (name, password, is_active, event_id) 
        VALUES ($1, $2, $3, $4) 
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.password)
    .bind(&payload.is_active)
    .bind(&payload.event_id)
    .fetch_one(&pool)
    .await;

    match res {
        Ok(judge) => Ok((http::StatusCode::CREATED, axum::Json(judge))),
        Err(err) => {
            eprintln!("Failed to create judge: {err:?}");

            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_judges(
    State(pool): State<PgPool>,
) -> Result<axum::Json<Vec<Judge>>, http::StatusCode> {
    let res = sqlx::query_as::<_, Judge>("SELECT * FROM judges")
        .fetch_all(&pool)
        .await;

    match res {
        Ok(judges) => Ok(axum::Json(judges)),
        Err(err) => {
            eprintln!("Failed to get judges: {err:?}");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
