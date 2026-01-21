use std::sync::OnceLock;

#[derive(Debug)]
pub struct EnvConfig {
    pub port: u16,
    pub service_name: String,
    pub debug_level: String,
    pub project_id: String,
    pub redis_base_path: String,
    pub redis_url: String,
    pub session_ttl: u64,
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    pub gemini_api_key: String,
}

static CONFIG: OnceLock<EnvConfig> = OnceLock::new();

pub fn get() -> &'static EnvConfig {
    CONFIG.get_or_init(EnvConfig::new)
}

impl EnvConfig {
    fn new() -> Self {
        dotenv::dotenv().ok();
        
        Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a number"),
            
            service_name: std::env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "rustlant-agent-local".to_string()),
            
            debug_level: std::env::var("DEBUG_LEVEL")
                .unwrap_or_else(|_| "INFO".to_string()),
            
            project_id: std::env::var("PROJECT_ID")
                .unwrap_or_else(|_| "local".to_string()),
            
            redis_base_path: std::env::var("REDIS_BASE_PATH")
                .unwrap_or_else(|_| "LCL".to_string()),
            
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://default@localhost:6379".to_string()),
            
            session_ttl: std::env::var("SESSION_TTL")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .expect("SESSION_TTL must be a number"),
            
            openai_api_key: std::env::var("OPENAI_API_KEY")
                .unwrap_or_default(),
                
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY")
                .unwrap_or_default(),
                
            gemini_api_key: std::env::var("GEMINI_API_KEY")
                .unwrap_or_default(),
        }
    }
}