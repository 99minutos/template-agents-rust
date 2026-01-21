use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CostArgs {
    pub item_name: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Error db")]
pub struct CostError;

#[derive(Serialize)]
pub struct CostOutput {
    pub price: f64,
    pub currency: String,
    pub in_stock: bool,
}

pub struct CostDatabase;

impl rig::tool::Tool for CostDatabase {
    const NAME: &'static str = "cost_database";

    type Error = CostError;
    type Args = CostArgs;
    type Output = CostOutput;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Consulta una base de datos de precios para estimar costos de reparación."
                .to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(CostArgs)).unwrap(),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(CostOutput {
            price: 150.50,
            currency: "USD".to_string(),
            in_stock: true,
        })
    }
}
