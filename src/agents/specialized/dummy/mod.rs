//! # Dummy Specialist - Template de Referencia
//!
//! Este módulo sirve como plantilla para crear nuevos agentes especialistas.
//! Copia esta carpeta completa y renómbrala para crear tu propio especialista.
//!
//! ## Arquitectura
//!
//! Un especialista es un agente que:
//! 1. Recibe argumentos estructurados del Orquestador
//! 2. Procesa la solicitud usando un LLM con su propio system prompt
//! 3. Puede usar herramientas (tools) para tareas específicas
//! 4. Devuelve una respuesta al Orquestador
//!
//! ## Checklist para crear un nuevo especialista
//!
//! - [ ] Copiar esta carpeta con un nuevo nombre
//! - [ ] Renombrar structs: `DummyArgs`, `DummyOutput`, `DummyError`, `DummySpecialist`
//! - [ ] Definir los argumentos que necesita en `Args`
//! - [ ] Definir la respuesta en `Output`
//! - [ ] Editar `system_prompt.md` con las instrucciones del especialista
//! - [ ] Agregar las tools necesarias en `AgentBuilder::new().tool(...)`
//! - [ ] Registrar en `specialized/mod.rs`: `pub mod mi_especialista;`
//! - [ ] Registrar en el Orquestador como `.tool(MiEspecialista::new(model))`

use crate::agents::tools::text_reverser::TextReverser;
use rig::{
    agent::{Agent, AgentBuilder},
    completion::{CompletionModel, Prompt},
    tool::Tool,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ============================================================================
// 1. ARGUMENTOS DE ENTRADA (Args)
// ============================================================================
//
// Define qué información necesita este especialista para trabajar.
// El Orquestador extraerá estos datos del mensaje del usuario y los enviará aquí.
//
// IMPORTANTE:
// - Usa `#[serde(rename = "...")]` si necesitas nombres diferentes en el JSON
// - Usa `Option<T>` para campos opcionales
// - Documenta cada campo con `///` para que el LLM entienda qué debe extraer

/// Argumentos que el Orquestador enviará a este especialista.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DummyArgs {
    /// El mensaje principal que se quiere procesar.
    pub message: String,

    /// Nivel de detalle en la respuesta: "brief", "normal", "detailed".
    /// Si no se especifica, se asume "normal".
    #[serde(default = "default_detail_level")]
    pub detail_level: String,
}

fn default_detail_level() -> String {
    "normal".to_string()
}

// ============================================================================
// 2. RESPUESTA DE SALIDA (Output)
// ============================================================================
//
// Define la estructura de la respuesta que devolverá este especialista.
// Puede ser un String simple o un struct complejo según necesites.

/// Respuesta estructurada del especialista.
#[derive(Debug, Clone, Serialize)]
pub struct DummyOutput {
    /// La respuesta generada por el agente.
    pub reply: String,

    /// Indica si la operación fue exitosa.
    pub success: bool,

    /// Metadata adicional (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<DummyMetadata>,
}

/// Metadata adicional de la respuesta.
#[derive(Debug, Clone, Serialize)]
pub struct DummyMetadata {
    /// Número de herramientas usadas en el proceso.
    pub tools_used: u32,

    /// Tokens consumidos (si está disponible).
    pub tokens_used: Option<u32>,
}

// ============================================================================
// 3. MANEJO DE ERRORES
// ============================================================================
//
// Define errores específicos para este especialista.
// Usa `thiserror` para implementar `std::error::Error` automáticamente.

/// Errores que pueden ocurrir durante la ejecución del especialista.
#[derive(Debug, thiserror::Error)]
pub enum DummyError {
    /// Error al comunicarse con el modelo de lenguaje.
    #[error("Error de comunicación con el LLM: {0}")]
    LlmError(String),

    /// Error al validar los argumentos de entrada.
    #[error("Argumentos inválidos: {0}")]
    ValidationError(String),

    /// Error al ejecutar una herramienta.
    #[error("Error en herramienta '{tool}': {message}")]
    ToolError { tool: String, message: String },
}

// ============================================================================
// 4. ESTRUCTURA DEL ESPECIALISTA
// ============================================================================
//
// El struct principal que envuelve al agente de Rig.
//
// NOTAS IMPORTANTES:
// - Usamos genéricos `<M: CompletionModel>` para aceptar cualquier modelo
// - `Arc<Agent<M>>` permite clonar el especialista sin duplicar el agente
// - El trait bound completo es necesario para async + threading

/// Especialista de prueba/demostración.
///
/// Este agente procesa mensajes de prueba y demuestra el flujo completo
/// de un especialista en el sistema.
#[derive(Clone)]
pub struct DummySpecialist<M>
where
    M: CompletionModel + Clone + Send + Sync + 'static,
{
    agent: Arc<Agent<M>>,
}

impl<M> DummySpecialist<M>
where
    M: CompletionModel + Clone + Send + Sync + 'static,
{
    /// Crea una nueva instancia del especialista.
    ///
    /// # Argumentos
    ///
    /// * `model` - El modelo de lenguaje a usar. Es inyectado por el Orquestador,
    ///   lo que permite cambiar modelos sin modificar este código.
    ///
    /// # Ejemplo
    ///
    /// ```ignore
    /// let specialist = DummySpecialist::new(gemini_model.clone());
    /// ```
    pub fn new(model: M) -> Self {
        let agent = AgentBuilder::new(model)
            // El system prompt define la personalidad y reglas del agente.
            // Se carga desde un archivo .md para facilitar edición.
            .preamble(include_str!("system_prompt.md"))
            // Registra las herramientas disponibles para este agente.
            // Puedes encadenar múltiples `.tool()` según necesites.
            .tool(TextReverser)
            // Aquí podrías agregar más herramientas:
            // .tool(OtraHerramienta)
            // .tool(MasHerramientas)
            .build();

        Self {
            agent: Arc::new(agent),
        }
    }

    /// Valida los argumentos antes de procesar.
    /// Útil para validaciones complejas que no se pueden expresar en el schema.
    fn validate_args(args: &DummyArgs) -> Result<(), DummyError> {
        if args.message.trim().is_empty() {
            return Err(DummyError::ValidationError(
                "El mensaje no puede estar vacío".to_string(),
            ));
        }

        let valid_levels = ["brief", "normal", "detailed"];
        if !valid_levels.contains(&args.detail_level.as_str()) {
            return Err(DummyError::ValidationError(format!(
                "Nivel de detalle inválido: '{}'. Usa: {:?}",
                args.detail_level, valid_levels
            )));
        }

        Ok(())
    }

    /// Construye el prompt interno que se enviará al LLM.
    fn build_prompt(args: &DummyArgs) -> String {
        format!(
            "Procesa el siguiente mensaje con nivel de detalle '{}':\n\n{}",
            args.detail_level, args.message
        )
    }
}

// ============================================================================
// 5. IMPLEMENTACIÓN DEL TRAIT TOOL
// ============================================================================
//
// Esta implementación permite que el Orquestador use este especialista

// como una herramienta más. El Orquestador decide cuándo invocarlo
// basándose en la `description` que definimos.

impl<M> Tool for DummySpecialist<M>
where
    M: CompletionModel + Clone + Send + Sync + 'static,
{
    /// Nombre único de la herramienta.
    /// El Orquestador usa este nombre para invocar al especialista.
    const NAME: &'static str = "dummy_specialist";

    type Error = DummyError;
    type Args = DummyArgs;
    type Output = DummyOutput;

    /// Define la "firma" de la herramienta para el Orquestador.
    ///
    /// La `description` es CRÍTICA: el LLM del Orquestador la usa para decidir
    /// cuándo invocar esta herramienta. Sé específico sobre cuándo usarla.
    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            // Esta descripción guía al Orquestador sobre cuándo usar este especialista.
            // Sé claro y específico. Incluye ejemplos de frases si es necesario.
            description: concat!(
                "Un agente de prueba para verificar el sistema. ",
                "Úsalo cuando el usuario quiera probar el sistema, ",
                "diga 'ping', 'test', o pida una demostración."
            )
            .to_string(),
            // El schema se genera automáticamente desde DummyArgs.
            // Los doc comments (`///`) en cada campo se usan como descripciones.
            parameters: serde_json::to_value(schemars::schema_for!(DummyArgs))
                .expect("Failed to serialize schema"),
        }
    }

    /// Ejecuta la lógica del especialista.
    ///
    /// Este método se invoca cuando el Orquestador decide usar esta herramienta.
    /// Aquí va toda la lógica de negocio del especialista.
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 1. Validar argumentos
        Self::validate_args(&args)?;

        // 2. Construir el prompt para el LLM interno
        let prompt = Self::build_prompt(&args);

        // 3. Ejecutar el agente
        let response = self
            .agent
            .prompt(&prompt)
            .await
            .map_err(|e| DummyError::LlmError(e.to_string()))?;

        // 4. Construir y devolver la respuesta
        Ok(DummyOutput {
            reply: response,
            success: true,
            metadata: Some(DummyMetadata {
                tools_used: 0,
                tokens_used: None,
            }),
        })
    }
}

// ============================================================================
// 6. TESTS (Opcional pero recomendado)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_message() {
        let args = DummyArgs {
            message: "   ".to_string(),
            detail_level: "normal".to_string(),
        };

        let result = DummySpecialist::<()>::validate_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_detail_level() {
        let args = DummyArgs {
            message: "test".to_string(),
            detail_level: "invalid".to_string(),
        };

        let result = DummySpecialist::<()>::validate_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_args() {
        let args = DummyArgs {
            message: "Hello".to_string(),
            detail_level: "brief".to_string(),
        };

        let result = DummySpecialist::<()>::validate_args(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_prompt() {
        let args = DummyArgs {
            message: "Test message".to_string(),
            detail_level: "detailed".to_string(),
        };

        let prompt = DummySpecialist::<()>::build_prompt(&args);
        assert!(prompt.contains("detailed"));
        assert!(prompt.contains("Test message"));
    }
}
