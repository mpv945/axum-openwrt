use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, Map};
use std::collections::BTreeMap;

pub struct JsonUtil;

impl JsonUtil {
    // =========================
    // JSON <-> String
    // =========================

    /// struct / json -> string
    pub fn to_string<T: Serialize>(data: &T) -> Result<String> {
        Ok(serde_json::to_string(data)?)
    }

    /// string -> struct
    pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T> {
        Ok(serde_json::from_str(s)?)
    }

    /// string -> Value
    pub fn str_to_value(s: &str) -> Result<Value> {
        Ok(serde_json::from_str(s)?)
    }

    /// Value -> string（pretty）
    pub fn to_pretty_string(value: &Value) -> Result<String> {
        Ok(serde_json::to_string_pretty(value)?)
    }

    // =========================
    // BTreeMap <-> JSON
    // =========================

    /// BTreeMap -> JSON Value
    pub fn map_to_json(map: &BTreeMap<String, String>) -> Value {
        let mut json_map = Map::new();

        for (k, v) in map {
            json_map.insert(k.clone(), Value::String(v.clone()));
        }

        Value::Object(json_map)
    }

    /// JSON Value -> BTreeMap<String, String>
    /// ⚠️ 只适用于扁平 JSON（value 必须是 string）
    pub fn json_to_map(value: &Value) -> Result<BTreeMap<String, String>> {
        let mut map = BTreeMap::new();

        if let Value::Object(obj) = value {
            for (k, v) in obj {
                let val = match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => {
                        return Err(anyhow::anyhow!(
                            "unsupported value type for key: {}",
                            k
                        ))
                    }
                };
                map.insert(k.clone(), val);
            }
        } else {
            return Err(anyhow::anyhow!("not a json object"));
        }

        Ok(map)
    }

    // =========================
    // String <-> BTreeMap
    // =========================

    /// JSON字符串 -> BTreeMap
    pub fn str_to_map(s: &str) -> Result<BTreeMap<String, String>> {
        let value: Value = serde_json::from_str(s)?;
        Self::json_to_map(&value)
    }

    /// BTreeMap -> JSON字符串
    pub fn map_to_string(map: &BTreeMap<String, String>) -> Result<String> {
        let value = Self::map_to_json(map);
        Self::to_string(&value)
    }
}