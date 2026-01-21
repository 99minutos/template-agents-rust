use crate::agents::tools::cost_database::CostDatabase;
use rig::{
    agent::{Agent, AgentBuilder},
    completion::{CompletionModel, Prompt},
    tool::Tool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ================================================================
// 1. Definición de Argumentos (Input/Output)
// ================================================================

/// Argumentos que el Orquestador enviará a este especialista.
/// Contiene la información necesaria para procesar un reporte de daño.
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct DamageReportArgs {
    /// Nombre o identificador del artículo dañado.
    pub item_name: String,
    /// Descripción detallada del daño observado por el usuario.
    pub description_of_damage: String,
}

// ================================================================
// 2. Definición de Errores
// ================================================================

/// Error personalizado para el especialista de daños.
#[derive(Debug, thiserror::Error)]
#[error("Error en DamageSpecialist: {0}")]
pub struct DamageError(String);

// ================================================================
// 3. Estructura del Especialista
// ================================================================

/// Especialista en gestión de reportes de daños y garantías.
///
/// Este agente analiza reportes de productos dañados y determina
/// si procede una devolución o reemplazo basándose en la descripción.
///
/// # Herramientas disponibles
/// - `CostDatabase`: Consulta precios para estimar costos de reparación/reemplazo.
#[derive(Clone)]
pub struct DamageSpecialist<M: CompletionModel + Clone + Send + Sync + 'static> {
    agent: Arc<Agent<M>>,
}

impl<M: CompletionModel + Clone + Send + Sync + 'static> DamageSpecialist<M> {
    /// Crea una nueva instancia del especialista de daños.
    ///
    /// # Argumentos
    /// * `model` - El modelo de lenguaje a usar (inyectado por el Orquestador).
    pub fn new(model: M) -> Self {
        let agent = AgentBuilder::new(model)
            .preamble(include_str!("system_prompt.md"))
            .tool(CostDatabase)
            .build();

        Self {
            agent: Arc::new(agent),
        }
    }
}

// ================================================================
// 4. Implementación del Trait Tool (Para Rig)
// ================================================================

impl<M: CompletionModel + Clone + Send + Sync + 'static> Tool for DamageSpecialist<M> {
    const NAME: &'static str = "damage_specialist";

    type Args = DamageReportArgs;
    type Output = String;
    type Error = DamageError;

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

        self.agent
            .prompt(&prompt)
            .await
            .map_err(|e| DamageError(e.to_string()))
    }
}
