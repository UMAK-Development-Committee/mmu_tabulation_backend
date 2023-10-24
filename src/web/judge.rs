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

impl Judge {
    fn new(create: CreateJudge) -> Self {
        let uuid = uuid::Uuid::new_v4();

        Self {
            id: uuid,
            name: create.name,
            event_id: create.event_id,
            password: create.password,
            is_active: create.is_active,
        }
    }
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
    let judge = Judge::new(payload);

    let res = sqlx::query(
        "INSERT INTO judges (id, name, password, is_active, event_id) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&judge.id)
    .bind(&judge.name)
    .bind(&judge.password)
    .bind(&judge.is_active)
    .bind(&judge.event_id)
    .execute(&pool)
    .await;

    match res {
        Ok(_) => Ok((http::StatusCode::CREATED, axum::Json(judge))),
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
