use crate::{
    api::request::{ChatRequest, ChatResponse, FileAttachment},
    infra::{
        errors::{DomainError, DomainResult},
        redis::{ChatMessage, Role},
    },
    state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;
use uuid::Uuid;

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatRequest>,
) -> Result<impl IntoResponse, DomainError> {
    let prompt = validate_prompt(&payload.prompt)?;
    let files = validate_files(payload.files)?;

    let session_id = payload
        .session_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let history = state
        .redis
        .get_history(&session_id)
        .await
        .unwrap_or_default();

    let response_text = state.orchestrator.chat(&prompt, history, files).await;

    let new_messages = vec![
        ChatMessage {
            role: Role::User,
            content: prompt,
        },
        ChatMessage {
            role: Role::Assistant,
            content: response_text.clone(),
        },
    ];

    if let Err(e) = state.redis.add_messages(&session_id, new_messages).await {
        tracing::warn!("Failed to save chat history: {}", e);
    }

    Ok((
        StatusCode::OK,
        Json(ChatResponse {
            response: response_text,
            session_id,
        }),
    ))
}

fn validate_prompt(prompt: &str) -> DomainResult<String> {
    let trimmed = prompt.trim();

    if trimmed.is_empty() {
        return Err(DomainError::validation("El prompt no puede estar vacío"));
    }

    if trimmed.len() > 10_000 {
        return Err(DomainError::validation(
            "El prompt excede el límite de 10,000 caracteres",
        ));
    }

    Ok(trimmed.to_string())
}

fn validate_files(files: Option<Vec<FileAttachment>>) -> DomainResult<Vec<FileAttachment>> {
    let files = files.unwrap_or_default();

    if files.len() > 10 {
        return Err(DomainError::validation(
            "No se pueden enviar más de 10 archivos por mensaje",
        ));
    }

    const MAX_FILE_SIZE: usize = 20 * 1024 * 1024; // 20MB en base64

    let cleaned_files: Vec<FileAttachment> = files
        .into_iter()
        .enumerate()
        .map(|(i, file)| {
            let base64 = strip_data_uri_prefix(&file.base64);

            if base64.is_empty() {
                return Err(DomainError::validation(format!(
                    "El archivo {} tiene contenido vacío",
                    i + 1
                )));
            }

            if base64.len() > MAX_FILE_SIZE {
                return Err(DomainError::validation(format!(
                    "El archivo {} excede el límite de 20MB",
                    i + 1
                )));
            }

            if file.mimetype.is_empty() {
                return Err(DomainError::validation(format!(
                    "El archivo {} no tiene mimetype definido",
                    i + 1
                )));
            }

            Ok(FileAttachment {
                base64,
                mimetype: file.mimetype,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(cleaned_files)
}

fn strip_data_uri_prefix(base64: &str) -> String {
    if let Some(comma_pos) = base64.find(',') {
        if base64[..comma_pos].contains("base64") {
            return base64[comma_pos + 1..].to_string();
        }
    }
    base64.to_string()
}
