use super::specialized::{address::AddressSpecialist, damage::DamageSpecialist};
use crate::api::request::FileAttachment;
use crate::infra::redis::{ChatMessage, Role};
use rig::agent::{Agent, AgentBuilder};
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::{Chat, Message};
use rig::message::{
    AssistantContent, Document, DocumentMediaType, DocumentSourceKind, ImageMediaType, UserContent,
};
use rig::providers::gemini;
use rig::providers::gemini::completion::gemini_api_types::{
    AdditionalParameters, GenerationConfig,
};

use rig::OneOrMany;

pub struct Orchestrator {
    pub agent: Agent<gemini::completion::CompletionModel>,
}

/// Orchestrator is a struct that manages the interaction between the user and the agent.
///         OpenAI
/// let client = openai::client::Client::from_env();
///
/// gpt-5-nano
/// gpt-5-mini
/// gpt-5
/// gpt-5.1
///
///         Anthropic
/// let client = anthropic::client::Client::from_env();
/// claude-3-7-sonnet-latest
/// claude-sonnet-4-0
/// claude-opus-4-0
///
///         Google
/// let client = google::client::Client::from_env();
/// gemini-3-flash
/// gemini-3-pro
/// gemini-3-deep-think
///
///
impl Orchestrator {
    pub fn new() -> Self {
        let client = gemini::client::Client::from_env();
        let gemini_model = client.completion_model("gemini-3-flash");
        let address_tool = AddressSpecialist::new(gemini_model.clone());
        let damage_tool = DamageSpecialist::new(gemini_model.clone());

        let gen_cfg = GenerationConfig {
            top_k: Some(1),
            top_p: Some(0.95),
            candidate_count: Some(1),
            ..Default::default()
        };
        let params = AdditionalParameters::default().with_config(gen_cfg);

        let agent = AgentBuilder::new(gemini_model)
            .preamble(include_str!("system_prompt.md"))
            .tool(address_tool)
            .tool(damage_tool)
            .additional_params(serde_json::to_value(params).unwrap())
            .build();

        Self { agent }
    }

    pub async fn chat(
        &self,
        prompt: &str,
        history: Vec<ChatMessage>,
        files: Vec<FileAttachment>,
    ) -> String {
        let rig_history: Vec<Message> = history
            .into_iter()
            .map(|msg| match msg.role {
                Role::User => Message::User {
                    content: OneOrMany::one(UserContent::text(msg.content)),
                },
                Role::System => Message::User {
                    content: OneOrMany::one(UserContent::text(format!(
                        "[System Context]: {}",
                        msg.content
                    ))),
                },
                Role::Assistant => Message::Assistant {
                    content: OneOrMany::one(AssistantContent::text(msg.content)),
                    id: None,
                },
            })
            .collect();

        let user_message: Message = Self::build_user_content(prompt, files).into();

        match self.agent.chat(user_message, rig_history).await {
            Ok(response) => response,
            Err(e) => {
                tracing::error!("Orchestrator chat failed: {}", e);
                "Lo siento, ocurrió un error procesando tu solicitud. Intenta de nuevo.".to_string()
            }
        }
    }

    fn build_user_content(prompt: &str, files: Vec<FileAttachment>) -> OneOrMany<UserContent> {
        if files.is_empty() {
            return OneOrMany::one(UserContent::text(prompt));
        }

        let mut contents: Vec<UserContent> = Vec::with_capacity(files.len() + 1);

        for file in files {
            if let Some(content) = Self::file_to_user_content(&file) {
                contents.push(content);
            } else {
                tracing::warn!("Unsupported mimetype: {}", file.mimetype);
            }
        }

        contents.push(UserContent::text(prompt));

        OneOrMany::many(contents).unwrap_or_else(|_| OneOrMany::one(UserContent::text(prompt)))
    }

    fn file_to_user_content(file: &FileAttachment) -> Option<UserContent> {
        let mimetype = file.mimetype.to_lowercase();

        if let Some(image_type) = Self::parse_image_mimetype(&mimetype) {
            return Some(UserContent::image_base64(
                &file.base64,
                Some(image_type),
                None,
            ));
        }

        if let Some(doc_type) = Self::parse_document_mimetype(&mimetype) {
            return Some(UserContent::Document(Document {
                data: DocumentSourceKind::Base64(file.base64.clone()),
                media_type: Some(doc_type),
                additional_params: None,
            }));
        }

        None
    }

    fn parse_image_mimetype(mimetype: &str) -> Option<ImageMediaType> {
        match mimetype {
            "image/jpeg" | "image/jpg" => Some(ImageMediaType::JPEG),
            "image/png" => Some(ImageMediaType::PNG),
            "image/gif" => Some(ImageMediaType::GIF),
            "image/webp" => Some(ImageMediaType::WEBP),
            "image/heic" => Some(ImageMediaType::HEIC),
            "image/heif" => Some(ImageMediaType::HEIF),
            "image/svg+xml" => Some(ImageMediaType::SVG),
            _ => None,
        }
    }

    fn parse_document_mimetype(mimetype: &str) -> Option<DocumentMediaType> {
        match mimetype {
            "application/pdf" => Some(DocumentMediaType::PDF),
            "text/plain" => Some(DocumentMediaType::TXT),
            "text/html" => Some(DocumentMediaType::HTML),
            "text/css" => Some(DocumentMediaType::CSS),
            "text/markdown" | "text/x-markdown" => Some(DocumentMediaType::MARKDOWN),
            "text/csv" => Some(DocumentMediaType::CSV),
            "application/xml" | "text/xml" => Some(DocumentMediaType::XML),
            "application/rtf" | "text/rtf" => Some(DocumentMediaType::RTF),
            "application/javascript" | "text/javascript" => Some(DocumentMediaType::Javascript),
            "text/x-python" | "application/x-python" => Some(DocumentMediaType::Python),
            _ => None,
        }
    }
}
