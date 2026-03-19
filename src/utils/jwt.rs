use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const SECRET: &[u8] = b"your-secret-key-change-me";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // 用户ID
    pub exp: usize,   // 过期时间
    pub iat: usize,   // 签发时间
}

pub struct JwtUtil;

impl JwtUtil {
    /// 生成 JWT
    pub fn generate(user_id: &str, expire_seconds: u64) -> Result<String> {
        let now = current_ts();

        let claims = Claims {
            sub: user_id.to_string(),
            iat: now,
            exp: now + expire_seconds as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET),
        )?;

        Ok(token)
    }

    /// 校验 JWT
    pub fn verify(token: &str) -> Result<Claims> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(SECRET),
            &Validation::default(),
        )?;

        Ok(data.claims)
    }
}

fn current_ts() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}