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

mod error;
mod web;

use web::{auth, candidate, category, criteria, event, judge, score};

// NOTE: will use .unwrap() or .expect() for now for most error handling situations, might change to a much
// better way of handling errors when polishing (if possible)

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
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        // Events
        .route("/events", post(event::create_event).get(event::get_events))
        .route("/events/:event_id", get(event::get_event))
        // Categories
        .route(
            "/events/:event_id/categories",
            post(category::create_category).get(category::get_categories),
        )
        .route(
            "/events/:event_id/categories/:category_id",
            get(category::get_category),
        )
        // Judges
        .route("/judges", post(judge::create_judge))
        // Criterias
        .route(
            "/events/:event_id/categories/:category_id/criterias",
            post(criteria::create_criteria).get(criteria::get_criterias),
        )
        .route(
            "/events/:event_id/categories/:category_id/criterias/:criteria_id",
            get(criteria::get_criteria),
        )
        // Candidates
        .route(
            "/candidates",
            post(candidate::create_candidate).get(candidate::get_candidates),
        )
        .route("/candidates/:candidate_id", get(candidate::get_candidate))
        .route(
            "/candidates/:candidate_id/scores",
            get(score::get_candidate_scores),
        )
        // .route(
        //     "/candidates/:candidate_id/add_scores",
        //     get(score::add_criteria_scores),
        // )
        .route("/scores", post(score::submit_score))
        .route("/notes", post(candidate::create_note))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    println!("\nServer has started, listening on: {}\n", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start Axum server.");
}

async fn hello_world() -> &'static str {
    "Hello, World!"
}
