use axum::response::Result;
use axum::{extract::State, http};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Debug, Serialize, FromRow)]
pub struct Judge {
    id: uuid::Uuid,
    name: String,
    password: String,
    is_active: bool,
    // Relationships
    category_id: uuid::Uuid,
}

impl Judge {
    fn new(create: CreateJudge) -> Self {
        let uuid = uuid::Uuid::new_v4();

        Self {
            id: uuid,
            name: create.name,
            category_id: create.category_id,
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
    category_id: uuid::Uuid,
}

pub async fn create_judge(
    State(pool): State<PgPool>,
    axum::Json(payload): axum::Json<CreateJudge>,
) -> Result<(http::StatusCode, axum::Json<Judge>), http::StatusCode> {
    let judge = Judge::new(payload);

    let res = sqlx::query("INSERT INTO judges (id, name, password, is_active, category_id) VALUES ($1, $2, $3, $4, $5)")
        .bind(&judge.id)
        .bind(&judge.name)
        .bind(&judge.password)
        .bind(&judge.is_active)
        .bind(&judge.category_id)
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
