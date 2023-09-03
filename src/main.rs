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
    Json, Router,
};
use dotenv::dotenv;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    // We require unique usernames. This tracks which usernames have been taken.
    user_set: Mutex<HashSet<String>>,
    // Channel used to send messages to all connected clients.
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    // Not sure if this is needed, will comment for now
    // initialize tracing
    // tracing_subscriber::fmt::init();

    dotenv().ok();

    let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(100);
    let app_state = Arc::new(AppState { user_set, tx });

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env not found.");

    let pool = sqlx::postgres::PgPool::connect(&db_url)
        .await
        .expect("Can't connect to database.");

    // TODO: Have relationships between candidates and other stuff, put these in separate
    // route for now
    let app = Router::new()
        // WEB SOCKET
        .route("/ws", get(ws_handler))
        .with_state(app_state)
        // CRUD routes
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
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST]),
        )
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    println!("\nServer has started, listening on: {}\n", addr);
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
    // ws.on_upgrade(move |socket| handle_socket(socket, addr))
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = socket.split();

    // Username gets set in the receive loop, if it's valid.
    // let mut username = String::new();

    // while let Some(Ok(message)) = receiver.next().await {
    //     if let Message::Text(name) = message {
    //         check_username(&state, &mut username, &name);
    //
    //         // If not empty we want to quit the loop else we want to quit function.
    //         if !username.is_empty() {
    //             break;
    //         } else {
    //             // Only send our client that username is taken.
    //             let _ = sender
    //                 .send(Message::Text(String::from("Username already taken.")))
    //                 .await;
    //
    //             return;
    //         }
    //     }
    // }

    let mut rx = state.tx.subscribe();

    // Now send the "joined" message to all subscribers.
    // let msg = format!("{} joined.", username);
    // tracing::debug!("{}", msg);
    // let _ = state.tx.send(msg);

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
    // let name = username.clone();

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Add username before message.
            let _ = tx.send(format!("User: {}", text));
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // Send "user left" message (similar to "joined" above).
    // let msg = format!("{} left.", username);
    // tracing::debug!("{}", msg);
    // let _ = state.tx.send(msg);

    // while let Some(msg) = socket.recv().await {
    //     let msg = if let Ok(msg) = msg {
    //         println!("{msg:?}");
    //         msg
    //     } else {
    //         println!("Client disconnected!");
    //         // client disconnected
    //         return;
    //     };
    //
    //     if socket.send(msg).await.is_err() {
    //         println!("Client disconnected!");
    //         // client disconnected
    //         return;
    //     }
    // }
}

fn check_username(state: &AppState, new_username: &mut String, name: &str) {
    let mut user_set = state.user_set.lock().unwrap();

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());

        new_username.push_str(name);
    }
}

// async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
//     if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
//         println!("Pinged {who}...");
//     } else {
//         println!("Could not send ping {who}.");
//
//         return;
//     }
//
//     if let Some(msg) = socket.recv().await {
//         if let Ok(msg) = msg {
//             if process_message(msg, who).is_break() {
//                 return;
//             }
//         } else {
//             println!("Client {who} disconnected!");
//
//             return;
//         }
//     }
// }
//
// // NOTE: Copy pasted
// /// helper to print contents of messages to stdout. Has special treatment for Close.
// fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
//     match msg {
//         Message::Text(t) => {
//             println!(">>> {} sent str: {:?}", who, t);
//         }
//         Message::Binary(d) => {
//             println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
//         }
//         Message::Close(c) => {
//             if let Some(cf) = c {
//                 println!(
//                     ">>> {} sent close with code {} and reason `{}`",
//                     who, cf.code, cf.reason
//                 );
//             } else {
//                 println!(">>> {} somehow sent close message without CloseFrame", who);
//             }
//             return ControlFlow::Break(());
//         }
//
//         Message::Pong(v) => {
//             println!(">>> {} sent pong with {:?}", who, v);
//         }
//         // You should never need to manually handle Message::Ping, as axum's websocket library
//         // will do so for you automagically by replying with Pong and copying the v according to
//         // spec. But if you need the contents of the pings you can see them here.
//         Message::Ping(v) => {
//             println!(">>> {} sent ping with {:?}", who, v);
//         }
//     }
//     ControlFlow::Continue(())
// }
