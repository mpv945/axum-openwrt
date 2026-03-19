use std::time::Duration;
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

pub async fn init_pool(database_url: &str) -> MySqlPool {
    MySqlPoolOptions::new()
        //.max_connections(10)
        .max_connections(50)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        //.connect("mysql://root:password@localhost:3306/test")
        .connect(database_url)
        .await
        .expect("connect mysql failed")
}