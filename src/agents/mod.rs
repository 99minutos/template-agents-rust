pub mod orchestrator;
pub mod specialized;
pub mod tools;

use rig::client::builder::FinalCompletionResponse;
use rig::completion::{
    CompletionError, CompletionModel, CompletionModelDyn, CompletionRequest, CompletionResponse,
};
use rig::streaming::StreamingCompletionResponse;
use std::sync::Arc;

#[derive(Clone)]
pub struct AnyModel(Arc<Box<dyn CompletionModelDyn>>);

impl AnyModel {
    pub fn new(model: Box<dyn CompletionModelDyn>) -> Self {
        Self(Arc::new(model))
    }
}

impl CompletionModel for AnyModel {
    type Response = ();
    type StreamingResponse = FinalCompletionResponse;

    async fn completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse<Self::Response>, CompletionError> {
        self.0.completion(request).await
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError> {
        self.0.stream(request).await
    }
}
