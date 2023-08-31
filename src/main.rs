// Ignore unused imports for now to remove some noise
#![allow(unused_imports)]

use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{env, io, net::SocketAddr};

// NOTE: will use .unwrap() for now for most error handling situations, might change to a much
// better way of handling errors when polishing

#[tokio::main]
async fn main() {
    // Not sure if this is needed, will comment for now
    // initialize tracing
    // tracing_subscriber::fmt::init();

    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env not found.");

    let pool = sqlx::postgres::PgPool::connect(&db_url)
        .await
        .expect("Can't connect to database.");

    // TODO: Have relationships between candidates and other stuff, put these in separate
    // route for now
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/events", post(create_event))
        .route("/categories", post(create_category))
        .route("/judges", post(create_judge))
        .route("/criterias", post(create_criteria))
        .route("/candidates", post(create_candidate))
        .route("/scores", post(submit_score))
        .route("/notes", post(create_note))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    println!("\nServer has started, listening on: {}\n", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello_world() -> &'static str {
    "Hello, World!"
}

#[derive(Debug, Deserialize, Serialize)]
struct Event {
    id: i32,
    name: String,
}

async fn create_event(
    State(pool): State<PgPool>,
    Json(new_event): Json<Event>,
) -> (StatusCode, Json<Event>) {
    let query = "INSERT INTO events (id, name) VALUES ($1, $2)";

    sqlx::query(query)
        .bind(&new_event.id)
        .bind(&new_event.name)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::CREATED, Json(new_event))
}

#[derive(Debug, Deserialize, Serialize)]
struct Category {
    id: i32,
    name: String,
}

async fn create_category(
    State(pool): State<PgPool>,
    Json(new_category): Json<Category>,
) -> (StatusCode, Json<Category>) {
    let query = "INSERT INTO categories (id, name) VALUES ($1, $2)";

    sqlx::query(query)
        .bind(&new_category.id)
        .bind(&new_category.name)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::CREATED, Json(new_category))
}

#[derive(Debug, Deserialize, Serialize)]
struct Judge {
    id: i32,
    name: String,
    password: String,
    is_active: bool,
}

async fn create_judge(
    State(pool): State<PgPool>,
    Json(new_judge): Json<Judge>,
) -> (StatusCode, Json<Judge>) {
    let query = "INSERT INTO judges (id, name, password, is_active) VALUES ($1, $2, $3, $4)";

    sqlx::query(query)
        .bind(&new_judge.id)
        .bind(&new_judge.name)
        .bind(&new_judge.password)
        .bind(&new_judge.is_active)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::CREATED, Json(new_judge))
}

#[derive(Debug, Deserialize, Serialize)]
struct Criteria {
    id: i32,
    name: String,
    description: String,
    max_score: i32,
    weight: f64,
}

async fn create_criteria(
    State(pool): State<PgPool>,
    Json(new_criteria): Json<Criteria>,
) -> (StatusCode, Json<Criteria>) {
    let query = "INSERT INTO criterias (id, name, description, max_score, weight) VALUES ($1, $2, $3, $4, $5)";

    sqlx::query(query)
        .bind(&new_criteria.id)
        .bind(&new_criteria.name)
        .bind(&new_criteria.description)
        .bind(&new_criteria.max_score)
        .bind(&new_criteria.weight)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::CREATED, Json(new_criteria))
}

#[derive(Debug, Deserialize, Serialize)]
struct Candidate {
    id: i32,
    first_name: String,
    middle_name: String,
    last_name: String,
    birthdate: String,
    gender: i32,
    college: String,
}

async fn create_candidate(
    State(pool): State<PgPool>,
    Json(new_candidate): Json<Candidate>,
) -> (StatusCode, Json<Candidate>) {
    let query = "INSERT INTO candidates (id, first_name, middle_name, last_name, birthdate, gender, college) VALUES ($1, $2, $3, $4, $5, $6, $7)";

    let parsed_birthdate =
        sqlx::types::chrono::NaiveDate::parse_from_str(&new_candidate.birthdate, "%Y-%m-%d")
            .expect("Date is invalid.");

    sqlx::query(query)
        .bind(&new_candidate.id)
        .bind(&new_candidate.first_name)
        .bind(&new_candidate.middle_name)
        .bind(&new_candidate.last_name)
        .bind(parsed_birthdate)
        .bind(&new_candidate.gender)
        .bind(&new_candidate.college)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::CREATED, Json(new_candidate))
}

#[derive(Debug, Deserialize, Serialize)]
struct CandidateScore {
    id: i32,
    score: i32,
    max: i32,
    time_of_scoring: String,
}

async fn submit_score(
    State(pool): State<PgPool>,
    Json(score): Json<CandidateScore>,
) -> (StatusCode, Json<CandidateScore>) {
    let query = "INSERT INTO scores (id, score, max, time_of_scoring) VALUES ($1, $2, $3, $4)";

    let parsed_time_of_scoring = sqlx::types::chrono::DateTime::parse_from_str(
        &score.time_of_scoring,
        "%Y-%m-%d %H:%M:%S %z",
    )
    .expect("Date and time is invalid.");

    sqlx::query(query)
        .bind(&score.id)
        .bind(&score.score)
        .bind(&score.max)
        .bind(parsed_time_of_scoring)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::OK, Json(score))
}

#[derive(Debug, Deserialize, Serialize)]
struct CandidateNote {
    id: i32,
    note: String,
    last_change: String,
}

async fn create_note(
    State(pool): State<PgPool>,
    Json(new_note): Json<CandidateNote>,
) -> (StatusCode, Json<CandidateNote>) {
    let query = "INSERT INTO notes (id, note, last_change) VALUES ($1, $2, $3)";

    let parsed_last_change_date = sqlx::types::chrono::DateTime::parse_from_str(
        &new_note.last_change,
        "%Y-%m-%d %H:%M:%S %z",
    )
    .expect("Date and time is invalid.");

    sqlx::query(query)
        .bind(&new_note.id)
        .bind(&new_note.note)
        .bind(parsed_last_change_date)
        .execute(&(pool))
        .await
        .unwrap();

    (StatusCode::CREATED, Json(new_note))
}
