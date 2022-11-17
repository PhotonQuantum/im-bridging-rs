use figment::providers::{Env, Serialized};
use figment::Figment;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub qrcode: Option<QRCodeConfig>,
    pub mongodb: MongoDBConfig,
    pub session_file: String,
    pub device_file: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            qrcode: None,
            mongodb: MongoDBConfig::default(),
            session_file: "session.token".to_string(),
            device_file: "device.json".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        Figment::from(Serialized::defaults(Self::default()))
            .merge(Env::prefixed("IM_BRIDGE_").split("_"))
            .extract::<Self>()
            .unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDBConfig {
    pub uri: String,
    pub database: String,
}

impl Default for MongoDBConfig {
    fn default() -> Self {
        Self {
            uri: "mongodb://localhost:27017".to_string(),
            database: "im-bridging".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeConfig {
    pub apikey: String,
    pub domain: String,
    pub to: String,
}
