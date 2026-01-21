use crate::agents::tools::cost_database::CostDatabase;
use rig::{
    agent::{Agent, AgentBuilder},
    completion::{CompletionModel, Prompt},
    tool::Tool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
#[error("DamageSpecialist error: {0}")]
pub struct DamageError(String);

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct DamageReportArgs {
    /// Nombre o identificador del artículo dañado.
    pub item_name: String,
    /// Descripción detallada del daño observado por el usuario.
    pub description_of_damage: String,
}

#[derive(Serialize)]
pub struct DamageResponse {
    pub response: String,
}

pub struct DamageSpecialist<M: CompletionModel> {
    pub agent: Agent<M>,
}

impl<M: CompletionModel> DamageSpecialist<M> {
    pub fn new(model: M) -> Self {
        let agent = AgentBuilder::new(model)
            .preamble(include_str!("system_prompt.md"))
            .tool(CostDatabase)
            .build();

        Self { agent }
    }
}

impl<M: CompletionModel> Tool for DamageSpecialist<M> {
    const NAME: &'static str = "damage_specialist";

    type Error = DamageError;
    type Args = DamageReportArgs;
    type Output = DamageResponse;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Usa este agente cuando el usuario reporte un artículo dañado, roto o defectuoso."
                    .to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(DamageReportArgs))
                .expect("Failed to serialize schema"),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let prompt = format!(
            "Reporte de daño para el artículo '{}'. Descripción del daño: {}",
            args.item_name, args.description_of_damage
        );

        let response = self
            .agent
            .prompt(&prompt)
            .await
            .map_err(|e| DamageError(e.to_string()))?;

        Ok(DamageResponse { response })
    }
}
