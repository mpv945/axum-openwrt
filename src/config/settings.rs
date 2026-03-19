use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {

    // 定义名字要和配置文件一致
    pub server: ServerConfig,

    pub database: DatabaseConfig,

    pub redis: RedisConfig,
    
    pub mqtt: MqttConfig
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {

    pub host: String,
    pub port: u16
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {

    pub url: String
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {

    pub url: String
}

#[derive(Debug, Deserialize)]
pub struct MqttConfig {

    pub host: String,
    pub port: u16,
    pub name: String,
    pub password: String
}