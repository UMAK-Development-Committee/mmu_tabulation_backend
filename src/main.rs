// Ignore unused imports for now to remove some noise
#![allow(unused_imports)]

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, Form, State,
    },
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use dotenv::dotenv;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgListener, PgPool};
use std::{
    collections::HashSet,
    env, io,
    net::SocketAddr,
    ops::ControlFlow,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

mod error;
mod web;

use web::{auth, candidate, category, criteria, event, judge, score};

// NOTE: will use .unwrap() or .expect() for now for most error handling situations, might change to a much
// better way of handling errors when polishing (if possible)

// Our shared state
struct AppState {
    // Idk if we need to track the judges
    user_set: Mutex<HashSet<String>>,
    // Channel used to send messages to all connected clients.
    tx: broadcast::Sender<String>,
}

// TODO: Add Postgres listener

#[tokio::main]
async fn main() {
    // Not sure if this is needed, will comment for now
    // initialize tracing
    // tracing_subscriber::fmt::init();

    dotenv().ok();

    let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(100);

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env not found.");
    let pool = sqlx::postgres::PgPool::connect(&db_url)
        .await
        .expect("Can't connect to database.");
    let mut pg_listener = PgListener::connect_with(&pool)
        .await
        .expect("Can't connect with listener.");

    pg_listener.listen_all(vec!["updates"]).await.unwrap();

    let app_state = Arc::new(AppState { user_set, tx });
    let app_state_clone = Arc::clone(&app_state);

    println!("\nNow listening to Postgres...\n");

    // WEB SOCKET (with db)
    // When something happens to the database, send it to the websocket for the frontend to be updated in real time
    tokio::spawn(async move {
        loop {
            while let Some(notification) = pg_listener.try_recv().await.unwrap() {
                let payload = notification.payload();
                app_state_clone.tx.send(payload.to_string()).unwrap();

                println!("Notification 8000: {payload:?}\n");
            }

            println!("Connection to Postgres lost.");
        }
    });

    // TODO: improve the way the routes are setup if possible
    let app = Router::new()
        // WEB SOCKET (without db)
        .route("/ws", get(ws_handler))
        .with_state(app_state)
        .route("/", get(hello_world))
        // CRUD routes
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
        .route("/candidates/:candidate_id", get(candidate::get_candidate))
        .route(
            "/candidates/:candidate_id/scores",
            get(score::get_candidate_scores),
        )
        // Judges
        .route("/judges", post(judge::create_judge))
        .route("/scores", post(score::submit_score))
        .route("/notes", post(candidate::create_note))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST]),
        )
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    println!("Server has started, listening on: {}\n", addr);
    // tracing::debug!("Listening on: {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start Axum server.");
}

async fn hello_world() -> &'static str {
    "Hello, World!"
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.tx.subscribe();

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
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
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = tx.send(format!("User: {}", text));
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

fn check_username(state: &AppState, new_username: &mut String, name: &str) {
    let mut user_set = state.user_set.lock().unwrap();

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());

        new_username.push_str(name);
    }
}
