use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddBlockRequest {
    pub data: String,
}