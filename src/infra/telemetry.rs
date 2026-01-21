use opentelemetry_gcloud_trace::GcpCloudTraceExporterBuilder;
use tracing_stackdriver::CloudTraceConfiguration;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub async fn init_tracing() -> anyhow::Result<()> {
    let config = crate::envs::get();
    let service_name = &config.service_name;
    let base_level = &config.debug_level;

    let env_filter = EnvFilter::new(format!(
        "h2=warn,hyper=warn,tokio_util=warn,tower_http=warn,rig=warn,axum=warn,{}",
        base_level
    ));

    let project_id = &config.project_id;

    // Stackdriver Logging Layer
    let stackdriver_layer =
        tracing_stackdriver::layer().with_cloud_trace(CloudTraceConfiguration {
            project_id: project_id.clone(),
        });

    // OpenTelemetry Trace Layer
    let builder = GcpCloudTraceExporterBuilder::for_default_project_id()
        .await?
        .with_resource(
            opentelemetry_sdk::Resource::builder()
                .with_attributes(vec![opentelemetry::KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    service_name.to_string(),
                )])
                .build(),
        );

    let provider = builder
        .create_provider()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create tracer provider: {}", e))?;

    let tracer_provider: opentelemetry_sdk::trace::Tracer = builder
        .install(&provider)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to install tracer: {}", e))?;

    // Open Telemetry
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer_provider);

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(stackdriver_layer)
        .with(telemetry_layer);

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    Ok(())
}
