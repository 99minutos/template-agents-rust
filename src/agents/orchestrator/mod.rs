use super::specialized::{address::AddressSpecialist, damage::DamageSpecialist};
use super::AnyModel;
use crate::infra::redis::{ChatMessage, Role};
use rig::agent::{Agent, AgentBuilder};
use rig::client::CompletionClient;
use rig::completion::{Chat, Message};
use rig::message::{AssistantContent, UserContent};
use rig::providers::gemini;
use rig::providers::gemini::completion::gemini_api_types::{
    AdditionalParameters, GenerationConfig,
};
use rig::OneOrMany;

pub struct Orchestrator {
    pub agent: Agent<AnyModel>,
}

impl Orchestrator {
    pub fn new() -> Self {
        // 1. Configuramos los Modelos
        // Aquí puedes seleccionar qué modelo usar para el orquestador y para cada herramienta.

        // Gemini
        // gemini-2.5-flash, gemini-2.5-pro, gemini-3-pro-preview
        let config = crate::envs::get();
        let gemini_client = gemini::client::Client::new(&config.gemini_api_key);
        let gemini_model = gemini_client.completion_model("gemini-2.5-flash");
        let gemini_model = AnyModel::new(Box::new(gemini_model));

        // 2. Inicializamos los Sub-Agentes (Tools)
        let address_tool = AddressSpecialist::new(gemini_model.clone());
        let damage_tool = DamageSpecialist::new(gemini_model.clone());

        // 3. Configuramos los Parámetros de Generación
        let gen_cfg = GenerationConfig {
            top_k: Some(1),
            top_p: Some(0.95),
            candidate_count: Some(1),
            ..Default::default()
        };
        let cfg = AdditionalParameters::default().with_config(gen_cfg);

        // 4.  Construimos el Agente Principal
        let agent = AgentBuilder::new(gemini_model.clone())
            .preamble(include_str!("system_prompt.md"))
            .tool(address_tool)
            .tool(damage_tool)
            .additional_params(serde_json::to_value(cfg).unwrap())
            .build();

        Self { agent }
    }

    pub async fn chat(&self, prompt: &str, history: Vec<ChatMessage>) -> String {
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

        match self.agent.chat(prompt, rig_history).await {
            Ok(response) => response,
            Err(e) => {
                tracing::error!("Orchestrator chat failed: {}", e);
                "Lo siento, ocurrió un error procesando tu solicitud. Intenta de nuevo.".to_string()
            }
        }
    }
}
