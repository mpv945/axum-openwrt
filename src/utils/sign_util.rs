use std::collections::BTreeMap;
use serde_json::Value;
use sha2::{Digest, Sha256};
use crate::utils::json_util::JsonUtil;

pub struct SignUtil;

impl SignUtil {
    /// MD5 签名（返回 hex）
    pub fn md5(input: &str) -> String {
        format!("{:x}", md5::compute(input))
    }

    /*
    use std::collections::BTreeMap;

    let mut params = BTreeMap::new();
    params.insert("name".to_string(), "alice".to_string());
    params.insert("age".to_string(), "30".to_string());

    let sign = SignUtil::sign_md5(&params, "secret123");
     */
    pub fn sign_json_md5(json: &Value, key: &str) -> String {
        let s = json.to_string(); // 注意字段顺序问题
        //let s = JsonUtil::to_string(json).unwrap();
        let map = JsonUtil::str_to_map(s.as_str()).unwrap();
        Self::sign_md5(&map, key)
    }

    /// SHA256 签名（返回 hex）
    pub fn sha256(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();

        hex::encode(result)
    }

    /// 参数排序 + MD5
    pub fn sign_md5(params: &BTreeMap<String, String>, key: &str) -> String {
        let query = build_query(params);

        let content = format!("{}&key={}", query, key);

        Self::md5(&content)
    }

    /// 参数排序 + SHA256
    pub fn sign_sha256(params: &BTreeMap<String, String>, key: &str) -> String {
        let query = build_query(params);

        let content = format!("{}&key={}", query, key);

        Self::sha256(&content)
    }

}

/// 构建 k=v&k=v
fn build_query(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
}