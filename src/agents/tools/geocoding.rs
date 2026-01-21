use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct GeoArgs {
    pub address: String,
}

#[derive(Debug, thiserror::Error)]
#[error("Error geocoding")]
pub struct GeoError;

#[derive(Serialize)]
pub struct GeoOutput {
    pub lat: f64,
    pub lng: f64,
    pub zip_code: String,
}

pub struct GeoCoding;

impl rig::tool::Tool for GeoCoding {
    const NAME: &'static str = "geocoding_service";

    type Error = GeoError;
    type Args = GeoArgs;
    type Output = GeoOutput;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Obtiene coordenadas y código postal simulados para una dirección."
                .to_string(),
            parameters: serde_json::to_value(schemars::schema_for!(GeoArgs)).unwrap(),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Mock response
        Ok(GeoOutput {
            lat: 19.4326,
            lng: -99.1332,
            zip_code: "06000".to_string(),
        })
    }
}
