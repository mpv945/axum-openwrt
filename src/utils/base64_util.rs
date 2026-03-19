use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

pub struct Base64Util;

/*
fn main() -> anyhow::Result<()> {
    let encoded = Base64Util::encode("rust");
    println!("encoded = {}", encoded);

    let decoded = Base64Util::decode(&encoded)?;
    println!("decoded = {}", decoded);

    Ok(())
}
// ✔ 二进制数据 : let encoded = general_purpose::STANDARD.encode(bytes);
// UTF-8 问题: String::from_utf8(bytes)?
// 如果不是字符串（比如图片），不要转 String： 如果不是字符串（比如图片），不要转 String：
 */
impl Base64Util {
    /// 标准 Base64 编码
    pub fn encode(input: &str) -> String {
        general_purpose::STANDARD.encode(input)
    }

    /// 标准 Base64 解码
    pub fn decode(input: &str) -> Result<String> {
        let bytes = general_purpose::STANDARD.decode(input)?;
        Ok(String::from_utf8(bytes)?)
    }

    /// URL Safe 编码
    pub fn encode_url(input: &str) -> String {
        general_purpose::URL_SAFE.encode(input)
    }

    /// URL Safe 解码
    pub fn decode_url(input: &str) -> Result<String> {
        let bytes = general_purpose::URL_SAFE.decode(input)?;
        Ok(String::from_utf8(bytes)?)
    }

    /// URL Safe 无 padding（JWT）
    pub fn encode_url_no_pad(input: &str) -> String {
        general_purpose::URL_SAFE_NO_PAD.encode(input)
    }

    /// URL Safe 无 padding 解码
    pub fn decode_url_no_pad(input: &str) -> Result<String> {
        let bytes = general_purpose::URL_SAFE_NO_PAD.decode(input)?;
        Ok(String::from_utf8(bytes)?)
    }
}