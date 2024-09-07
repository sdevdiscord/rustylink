use serde::Deserialize;

#[derive(Deserialize)]
pub struct HelloPayloadData {
    pub heartbeat_interval: u64
}

#[derive(Deserialize)]
pub struct HelloPayload {
    pub op: u8,
    pub d: HelloPayloadData
}