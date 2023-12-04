// Ignore unused imports for now to remove some noise
#![allow(unused_imports)]
#![allow(warnings)]

use anyhow::Context;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http,
    response::Response,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use futures::{sink::SinkExt, stream::StreamExt};
use sqlx::postgres::PgListener;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, sync::broadcast};
use tower_http::cors::CorsLayer;

mod error;
mod handlers;

use handlers::{auth, candidate, category, college, criteria, event, judge, note, score};

struct AppState {
    // Channel used to send messages to all connected clients.
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    dotenv().ok();

    let (tx, _rx) = broadcast::channel(50);

    let db_url = env::var("DATABASE_URL").context("DATABASE_URL env not found.")?;
    let ip_addr = env::var("IP_ADDRESS").unwrap_or("127.0.0.1".to_string());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(50)
        .connect(&db_url)
        .await?;

    let mut pg_listener = PgListener::connect_with(&pool).await?;

    pg_listener.listen_all(vec!["updates"]).await?;

    let app_state = Arc::new(AppState { tx });

    println!("\nNow listening to Postgres...\n");

    db_ws_listen(pg_listener, app_state.clone());

    let app = Router::new()
        // WebSocket
        .route("/ws", get(ws_handler))
        .with_state(app_state)
        .route("/", get(health))
        // Auth
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
        .route("/candidates/score", get(score::get_candidate_score))
        .route("/candidates/:candidate_id", get(candidate::get_candidate))
        .route("/judges", post(judge::create_judge).get(judge::get_judges))
        .route("/judges/:judge_id", get(judge::get_judge))
        .route(
            "/scores",
            post(score::submit_score).get(score::get_candidate_scores),
        )
        .route("/scores/update", post(score::update_score))
        .route("/scores/final", get(score::get_candidate_final_scores))
        .route("/scores/download", get(score::generate_score_spreadsheet))
        .route("/notes", post(note::create_note).get(note::get_note))
        .route("/college", get(college::get_colleges))
        // EXPERIMENTAL
        .route("/scores/foo", get(score::foo))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    // For local development (NOT EXPOSURE TO THE NETWORK) it must be [127.0.0.1]
    let listener = TcpListener::bind(format!("{}:8000", ip_addr)).await?;

    println!(
        "Server has started, listening on: {:?}\n",
        listener.local_addr()?
    );

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn health() -> (http::StatusCode, String) {
    (http::StatusCode::OK, "Hello, World!".to_string())
}

// Listen to the database in real-time and send the notification to the websocket
fn db_ws_listen(mut pg_listener: PgListener, app_state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            while let Some(notification) = pg_listener
                .try_recv()
                .await
                .context("Failed to receive notification.")
                .unwrap()
            {
                let payload = notification.payload();

                app_state
                    .tx
                    .send(payload.to_string())
                    .context("Failed to send payload.")
                    .unwrap();

                println!("Notification:\n{payload:?}\n");
            }

            println!("Connection to Postgres lost.");
        }
    });
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.tx.subscribe();

    // Spawn the first task that will receive broadcast messages and send text messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();

    // Spawn a task that takes messages from the websocket and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(event))) = receiver.next().await {
            println!("Sending Event:\n{event}\n");

            tx.send(event).unwrap();
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
