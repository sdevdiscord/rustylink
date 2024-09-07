mod discord;
mod ws;

use std::time;

use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use discord::{opcodes::VoiceOpcode, payloads::HelloPayload};
use serde::Deserialize;
use serde_json::json;
use tokio::task;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};


#[tokio::main]
async fn main() {
    let app = Router::new()
        // `POST /` goes to `root`
        .route("/", post(root));

    // run our app with hyper, listening globally on port 2333
    let listener = tokio::net::TcpListener::bind("0.0.0.0:2333").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize, Debug)]
struct VoiceUpdatePayload {
    token: String,
    endpoint: String,

    #[serde(rename = "sessionId")]
    session_id: String,
    
    #[serde(rename = "userId")]
    user_id: String,

    #[serde(rename = "guildId")]
    guild_id: String,
}

async fn root(
    Json(payload): Json<VoiceUpdatePayload>
) -> StatusCode {
    println!("payload: {:?}", payload);

    let ws = ws::VoiceWebsocket::new(payload.endpoint, payload.token, payload.session_id, payload.user_id, payload.guild_id);
    ws.await;

    StatusCode::OK
}
