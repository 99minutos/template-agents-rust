use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileAttachment {
    pub base64: String,
    pub mimetype: String,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub prompt: String,
    pub session_id: Option<String>,
    pub files: Option<Vec<FileAttachment>>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: String,
}
