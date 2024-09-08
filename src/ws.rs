use std::time;

use tokio::{sync::mpsc::UnboundedSender, task};
use tokio_websockets::{ClientBuilder, Message};
use futures_util::StreamExt;
use serde_json::json;

use crate::discord::{opcodes::VoiceOpcode, payloads::HelloPayload};

pub struct VoiceWebsocket {
    token: String,
    session_id: String,
    user_id: String,
    server_id: String,
    endpoint: String
}

impl VoiceWebsocket {
    pub async fn new(endpoint: String, token: String, session_id: String, user_id: String, server_id: String) -> Self {
        const VOICE_WEBSOCKET_VERSION: u8 = 8;

        let url = format!("wss://{}?v={}", endpoint, VOICE_WEBSOCKET_VERSION);

        let (ws, _) = ClientBuilder::from_uri(&url).connect().await.expect("Failed to connect");

        let (write, read) = ws.split();

        let (write_tx, write_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

        let identify_msg = Message::text(json!({
            "op": VoiceOpcode::Identify as u8,
            "d": {
                "server_id": server_id,
                "user_id": user_id,
                "session_id": session_id,
                "token": token
            }
        }).to_string());

        let write_to_ws = tokio_stream::wrappers::UnboundedReceiverStream::new(write_rx).map(Ok).forward(write);

        write_tx.clone().send(identify_msg).unwrap();

        let ws_to_recv = {
            read.fold(write_tx, |write_tx, msg| async move {
                let data: serde_json::Value = serde_json::from_str(msg.unwrap().into_text().unwrap().as_str()).unwrap();

                match data["op"].as_u64().unwrap() {
                    8 => { // VoiceOpcodes::Hello
                        let data: HelloPayload = serde_json::from_value(data).unwrap();
                        let heartbeat_interval = data.d.heartbeat_interval;

                        task::spawn(heartbeat(write_tx.clone(), heartbeat_interval));
                    }
                    _ => {}
                }

                write_tx
            })
        };

        task::spawn(write_to_ws);
        task::spawn(ws_to_recv);

        Self {
            token,
            session_id,
            user_id,
            server_id,
            endpoint
        }
    }
}

#[inline]
#[allow(dead_code)]
async fn heartbeat(tx: UnboundedSender<Message>, interval: u64) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;
        tx.send(Message::text(json!({
            "op": VoiceOpcode::Heartbeat as u8,
            "d": {
                "t": time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_millis(),
                "seq_ack": 1
            }
        }).to_string())).unwrap();
    }
}