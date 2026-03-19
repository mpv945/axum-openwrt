mod settings;

pub mod log;

pub use settings::Settings;

use config::{Config, File};

use std::{env, fs, path::PathBuf};
use regex::Regex;
use once_cell::sync::Lazy;
static ENV_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{([A-Z0-9_]+)(?::([^}]*))?\}").unwrap());


pub fn load_config() -> Settings {

    let builder = Config::builder()
        //.add_source(File::with_name("config/default"));
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name("config/dev").required(false))
        .add_source(config::Environment::with_prefix("APP")); // 支持： APP_SERVER_PORT=9000

    builder
        .build()
        .unwrap()
        .try_deserialize::<Settings>()
        .unwrap()
}

pub fn load_settings_plus() -> Settings {
    let raw = load_raw_config();
    let replaced = replace_env_vars(&raw);

    toml::from_str(&replaced).expect("配置解析失败")
}
fn load_raw_config() -> String {
    // 1️⃣ 当前运行目录 ./configs/default.toml
    let runtime_path = PathBuf::from("./configs/default.toml");

    if runtime_path.exists() {
        println!("使用运行目录配置: {:?}", runtime_path);
        return fs::read_to_string(runtime_path).unwrap();
    }

    // 2️⃣ 可执行文件同级 configs
    if let Ok(exe_path) = env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            let exe_config = dir.join("configs/default.toml");
            if exe_config.exists() {
                println!("使用可执行目录配置: {:?}", exe_config);
                return fs::read_to_string(exe_config).unwrap();
            }
        }
    }

    // 3️⃣ 项目根目录（开发环境）
    let project_config = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("configs/default.toml");

    if project_config.exists() {
        println!("使用项目目录配置: {:?}", project_config);
        return fs::read_to_string(project_config).unwrap();
    }

    // 4️⃣ fallback：内嵌配置
    println!("使用内嵌默认配置");

    return include_str!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/config/default.toml"
        )
    ).to_string();
}
fn replace_env_vars(input: &str) -> String {
    ENV_REGEX
        .replace_all(input, |caps: &regex::Captures| {
            let key = &caps[1];
            let default = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            std::env::var(key).unwrap_or_else(|_| default.to_string())
        })
        .to_string()
}