use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ReverserArgs {
    pub text: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Error reversing")]
pub struct ReverserError;

pub struct TextReverser;

impl rig::tool::Tool for TextReverser {
    const NAME: &'static str = "text_reverser";

    type Error = ReverserError;
    type Args = ReverserArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Invierte una cadena de texto. Útil para demostrar manipulación de strings."
                    .to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(ReverserArgs)).unwrap(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(args.text.chars().rev().collect())
    }
}
