use anyhow::Result;
use reqwest::{Client, ClientBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct HttpClient {
    pub client: Client,
}

impl HttpClient {
    /// 创建客户端（支持忽略 TLS 证书）
    pub fn new(ignore_tls: bool) -> Result<Self> {
        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90));

        if ignore_tls {
            builder = builder
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true);
        }

        let client = builder.build()?;

        Ok(Self { client })
    }

    /// GET JSON
    /*pub async fn get_json<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let resp = self.client.get(url).send().await?;

        let data = resp.json::<T>().await?;

        Ok(data)
    }*/

    /// GET + query 参数
    pub async fn get_with_query<T, Q>(&self, url: &str, query: &Q) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
        Q: Serialize,
    {
        let resp = self.client
            .get(url)
            .query(query)
            //.bearer_auth(token)
            //.headers()
            .send()
            .await?;

        Ok(resp.json::<T>().await?)
    }

    /// POST JSON
    pub async fn post_json<T, R>(&self, url: &str, body: &T) -> Result<R>
    where
        T: Serialize + ?Sized,
        R: DeserializeOwned,
    {
        let resp = self.client.post(url).json(body).send().await?;

        let data = resp.json::<R>().await?;

        Ok(data)
    }
}