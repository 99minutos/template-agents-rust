use crate::agents::tools::geocoding::GeoCoding;
use rig::{
    agent::{Agent, AgentBuilder},
    completion::{CompletionModel, Prompt},
    tool::Tool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
#[error("AddressSpecialist error: {0}")]
pub struct AddressError(String);

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct AddressChangeArgs {
    /// Identificador único del cliente (ej. "CLI-12345").
    pub customer_id: String,

    /// La nueva dirección completa incluyendo calle, número, ciudad y código postal.
    pub new_address: String,

    /// Motivo del cambio de dirección (ej. "mudanza", "error en registro", "temporal").
    pub reason: String,
}

#[derive(Serialize)]
pub struct AddressResponse {
    pub response: String,
}

pub struct AddressSpecialist<M: CompletionModel> {
    pub agent: Agent<M>,
}

impl<M: CompletionModel> AddressSpecialist<M> {
    pub fn new(model: M) -> Self {
        let agent = AgentBuilder::new(model)
            .preamble(include_str!("system_prompt.md"))
            .tool(GeoCoding)
            .build();

        Self { agent }
    }
}

impl<M: CompletionModel> Tool for AddressSpecialist<M> {
    const NAME: &'static str = "address_specialist";

    type Error = AddressError;
    type Args = AddressChangeArgs;
    type Output = AddressResponse;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Usa este agente cuando el usuario quiera cambiar su dirección de entrega, modificar datos de envío, o tenga preguntas sobre logística.".to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(AddressChangeArgs))
                .expect("Failed to serialize schema"),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let prompt = format!(
            "Procesa la siguiente solicitud de cambio de dirección:\n- Cliente: {}\n- Nueva dirección: {}\n- Motivo: {}",
            args.customer_id, args.new_address, args.reason
        );

        let response = self
            .agent
            .prompt(&prompt)
            .await
            .map_err(|e| AddressError(e.to_string()))?;

        Ok(AddressResponse { response })
    }
}
