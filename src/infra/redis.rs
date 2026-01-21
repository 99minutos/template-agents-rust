use anyhow::Result;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Role {
    User,
    System,
    Assistant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Clone)]
pub struct RedisProvider {
    connection: MultiplexedConnection,
    base_path: String,
    ttl: u64,
}

impl RedisProvider {
    pub async fn new() -> Result<Self> {
        let config = crate::envs::get();
        let redis_url = &config.redis_url;
        let base_path = config.redis_base_path.clone();
        let ttl = config.session_ttl;

        let client = redis::Client::open(redis_url.as_str())?;
        let connection = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            connection,
            base_path,
            ttl,
        })
    }

    fn get_key(&self, session_id: &str) -> String {
        format!("{}:{}", self.base_path, session_id)
    }

    pub async fn get_history(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        let mut con = self.connection.clone();
        let key = self.get_key(session_id);

        let messages: Vec<String> = con.lrange(&key, 0, -1).await?;

        messages
            .into_iter()
            .map(|json| serde_json::from_str(&json).map_err(Into::into))
            .collect()
    }

    pub async fn add_messages(&self, session_id: &str, messages: Vec<ChatMessage>) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }

        let mut con = self.connection.clone();
        let key = self.get_key(session_id);

        let serialized: Vec<String> = messages
            .iter()
            .map(|msg| serde_json::to_string(msg))
            .collect::<Result<Vec<_>, _>>()?;

        con.rpush::<_, _, ()>(&key, serialized).await?;
        con.expire::<_, ()>(&key, self.ttl as i64).await?;

        Ok(())
    }
}
