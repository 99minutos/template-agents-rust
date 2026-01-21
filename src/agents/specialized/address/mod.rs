use crate::agents::tools::geocoding::GeoCoding;
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
///
/// Cada campo debe tener un doc comment (`///`) que describa su propósito.
/// Estos comentarios se convierten en descripciones del schema JSON, ayudando
/// al LLM a entender qué datos debe extraer de la conversación del usuario.
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct AddressChangeArgs {
    /// Identificador único del cliente (ej. "CLI-12345").
    pub customer_id: String,

    /// La nueva dirección completa incluyendo calle, número, ciudad y código postal.
    pub new_address: String,

    /// Motivo del cambio de dirección (ej. "mudanza", "error en registro", "temporal").
    pub reason: String,
}

// ================================================================
// 2. Definición de Errores
// ================================================================

/// Error personalizado para el especialista de direcciones.
///
/// Incluye el mensaje de error original para facilitar debugging.
#[derive(Debug, thiserror::Error)]
#[error("AddressSpecialist error: {0}")]
pub struct AddressError(String);

// ================================================================
// 3. Estructura del Especialista
// ================================================================

/// Especialista en cambios de dirección y logística de envíos.
///
/// Este agente se encarga de:
/// - Validar direcciones usando el servicio de geocodificación
/// - Calcular costos adicionales por cambio de zona
/// - Procesar solicitudes de cambio de dirección
///
/// # Herramientas Disponibles
/// - `GeoCoding`: Obtiene coordenadas y código postal de una dirección.
#[derive(Clone)]
pub struct AddressSpecialist<M: CompletionModel + Clone + Send + Sync + 'static> {
    agent: Arc<Agent<M>>,
}

impl<M: CompletionModel + Clone + Send + Sync + 'static> AddressSpecialist<M> {
    /// Crea una nueva instancia del especialista de direcciones.
    ///
    /// # Argumentos
    /// * `model` - El modelo de lenguaje a usar (inyectado por el Orquestador).
    pub fn new(model: M) -> Self {
        let agent = AgentBuilder::new(model)
            .preamble(include_str!("system_prompt.md"))
            .tool(GeoCoding)
            .build();

        Self {
            agent: Arc::new(agent),
        }
    }
}

// ================================================================
// 4. Implementación del Trait Tool
// ================================================================

impl<M: CompletionModel + Clone + Send + Sync + 'static> Tool for AddressSpecialist<M> {
    const NAME: &'static str = "address_specialist";

    type Error = AddressError;
    type Args = AddressChangeArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Usa este agente cuando el usuario quiera cambiar su dirección de entrega, \
                          modificar datos de envío, o tenga preguntas sobre logística."
                    .to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(AddressChangeArgs))
                .expect("Failed to serialize schema"),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let prompt = format!(
            "Procesa la siguiente solicitud de cambio de dirección:\n\
             - Cliente: {}\n\
             - Nueva dirección: {}\n\
             - Motivo: {}",
            args.customer_id, args.new_address, args.reason
        );

        self.agent
            .prompt(&prompt)
            .await
            .map_err(|e| AddressError(e.to_string()))
    }
}
