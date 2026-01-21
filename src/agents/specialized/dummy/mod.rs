use crate::agents::tools::text_reverser::TextReverser;
use rig::{
    agent::{Agent, AgentBuilder},
    completion::{CompletionModel, Prompt},
    tool::Tool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
#[error("DummySpecialist error: {0}")]
pub struct DummyError(String);

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct DummyArgs {
    pub message: String,
    pub detail_level: String,
}

#[derive(Serialize)]
pub struct DummyResponse {
    pub response: String,
}

pub struct DummySpecialist<M: CompletionModel> {
    pub agent: Agent<M>,
}

impl<M: CompletionModel> DummySpecialist<M> {
    pub fn new(model: M) -> Self {
        let agent = AgentBuilder::new(model)
            .preamble(include_str!("system_prompt.md"))
            .tool(TextReverser)
            .build();

        Self { agent }
    }
}

impl<M: CompletionModel> Tool for DummySpecialist<M> {
    const NAME: &'static str = "dummy_specialist";

    type Error = DummyError;
    type Args = DummyArgs;
    type Output = DummyResponse;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Un agente de prueba para verificar el sistema. Úsalo cuando el usuario quiera probar el sistema, diga 'ping', 'test', o pida una demostración.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(DummyArgs))
                .expect("Failed to serialize schema"),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let prompt = format!(
            "Procesa el siguiente mensaje con nivel de detalle '{}':\n\n{}",
            args.detail_level, args.message
        );

        let response = self
            .agent
            .prompt(&prompt)
            .await
            .map_err(|e| DummyError(e.to_string()))?;

        Ok(DummyResponse { response })
    }
}
