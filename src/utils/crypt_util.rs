use anyhow::Result;
use pwhash::unix;

pub struct CryptUtil;

impl CryptUtil {
    pub fn crypt(password: &str, alg: &str, salt: &str) -> Result<String> {
        let salt_str = format!("${}${}", alg, salt);

        // ✅ 正确写法
        let hash = unix::crypt(password, &salt_str)?;

        Ok(hash)
    }

    pub fn verify(password: &str, hash: &str) -> bool {
        unix::verify(password, hash)
    }
}