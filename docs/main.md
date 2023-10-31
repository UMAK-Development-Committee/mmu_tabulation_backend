# MMU Tabulation Backend

## main.rs

### Structs

#### AppState

| Name | Type             | Crate | Description                                                                                    |
| ---- | ---------------- | ----- | ---------------------------------------------------------------------------------------------- |
| tx   | `Sender<String>` | tokio | A sender for a broadcast channel that can send messages of type `String`. Used in Web Sockets. |

### Functions

#### main()

The main function of the backend. This is where everything is set up, including the database and WebSocket connections, all routes for each CRUD operation, etc.

##### Returns

A `Result` enum from the `anyhow` crate. If an error occurs, return an `Error`. Otherwise, return a unit type `()` which is nothing.

#### health()

The function used to check if the server is running.

##### Returns

A tuple containing an HTTP Status Code and a String

#### db_ws_listen(mut pg_listener, app_state)

Listens to Postgres. If a change is made in the database, a notification will be received which will be sent to the client via WebSockets.

| Name        | Type            | Crate | Description                                                                                                                       |
| ----------- | --------------- | ----- | --------------------------------------------------------------------------------------------------------------------------------- |
| pg_listener | `PgListener`    | sqlx  | Used for listening to Postgres notifications                                                                                      |
| app_state   | `Arc<AppState>` | _N/A_ | An [Arc (Atomically Reference Counted)](https://doc.rust-lang.org/std/sync/struct.Arc.html) smart pointer to the AppState struct. |

##### Returns

Void

#### ws_handler(ws, State(state))

Establish and manage WebSocket connections

| Name         | Type                   | Crate           | Description                                                                                           |
| ------------ | ---------------------- | --------------- | ----------------------------------------------------------------------------------------------------- |
| ws           | `WebSocketUpgrade`     | axum            | The WebSocket upgrade request.                                                                        |
| State(state) | `State<Arc<AppState>>` | _State_<br>axum | A extractor for the app's state. In this case, [`AppState`](#AppState) is being extracted to get `tx` |

##### Returns

The WebSocket connection that will be sent to the client.
