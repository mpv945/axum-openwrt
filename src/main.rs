use axum::Router;
use mimalloc::MiMalloc;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod bootstrap;
mod router;
mod handler;
mod service;
mod repository;
mod model;
mod middleware;

mod config;

mod common;
mod state;
mod db;

mod redis;
mod mqtt;
mod utils;
mod r#static;
mod excel;

use utils::command::CommandExecutor;

use crate::config::{load_config, load_settings_plus};
use crate::model::app_state::AppState;

use tracing::info;

pub type AppRouter = Router<Arc<AppState>>;

use crate::redis::stream_consumer::start_stream_worker;
use db::mysql::init_pool;
use crate::mqtt::consumer::MqttConsumer;
use crate::mqtt::producer::MqttProducer;
use crate::redis::pub_sub_consumer::pub_sub_listener;
use crate::utils::http_client::HttpClient;
use tokio_cron_scheduler::{Job, JobScheduler};
use crate::excel::excel_util::ExcelUtil;

//#[tokio::main]
#[tokio::main(flavor="multi_thread")]
//#[tokio::main(flavor = "multi_thread", worker_threads = 0)]  // 0 = 自动 = CPU 核数,不能为0
async fn main() {

    //let settings = Arc::new(load_config());
    // 支持文件变量和可执行文件内嵌
    let settings = Arc::new(load_settings_plus());

    // 2. 环境变量覆盖（Docker 最爱）
    //     if let Ok(host) = std::env::var("HOST") {
    //         config.server.host = host;
    //     }

    let redis_url = settings.redis.url.clone(); // 克隆一个 String
    /*println!("server port {}", settings.server.port);
    println!("server host {}", settings.server.host);
    println!("database url {}", settings.database.url);*/
    println!("redis url {}", redis_url);

    let _log_guard = config::log::init_tracing();

    // 启动每日 00:00:00 清理日志任务
    config::log::spawn_daily_log_cleanup();

    /*let shared_state = Arc::new(model::app_state::AppState {
        config: model::app_state::Config {
            jwt_secret: "secret11111111".into(),
        },
    });*/

    let pool = init_pool(&settings.database.url).await;


    /*let redis_url = redis::redis_client::build_redis_url(
        "redis-19019.c283.us-east-1-4.ec2.cloud.redislabs.com",
        19019,
        "ezzBTBzKUwGp0VkKFsdbV6DHwZrMuiGu"
    );*/
    //let redis = redis::redis_client::RedisClient::new(&settings.redis.url).await.unwrap();
    let redis_client = redis::redis_client::RedisClient::new(&redis_url).await.expect("Redis init failed");
    let redis = Arc::new(redis_client);
    // 启动 MQ worker
    tokio::spawn({
        let redis0 = redis.clone(); // ✅ clone Arc
        async move {
            start_stream_worker(redis0.as_ref().clone()).await.expect("TODO: panic message");
        }
    });

    // 启动 pub sub listener
    /*tokio::spawn(async move {
        // move settings 进入闭包
        //let redis_url = &settings.redis.url;
        pub_sub_listener(&redis_url, "events").await.unwrap();
    });*/
    tokio::spawn(pub_sub_listener(redis_url, "events"));

    // mqtt
    // 创建生产客户端
    let mqtt_host = Arc::new(settings.mqtt.host.clone());
    let mqtt_port = Arc::new(settings.mqtt.port.clone());
    let mqtt_name = Arc::new(settings.mqtt.name.clone());
    let mqtt_pwd = Arc::new(settings.mqtt.password.clone());
    let producer = MqttProducer::new(
        mqtt_host.as_str(),
        *mqtt_port, //*解引用
        mqtt_name.as_str(),
        mqtt_pwd.as_str(),
    ).await;
    // 启动消费者
    tokio::spawn({
        let redis1 = redis.clone(); // ✅ clone Arc
        let host = mqtt_host.clone();
        let port = mqtt_port;
        let name = mqtt_name.clone();
        let pwd = mqtt_pwd.clone();
        async move {
            MqttConsumer::start(
                &host,
                *port, //*解引用
                &name,
                &pwd,
                redis1.as_ref().clone(),
            ).await;
        }
    });

    // http工具类
    let http_client = HttpClient::new(true).unwrap(); // 忽略 TLS

    let shared_state = Arc::new(AppState::new(
        model::app_state::Config {
            jwt_secret: "secret11111111".into(),
        },
        pool,
        producer,
        Arc::new(http_client),
        redis.clone(),
    ));

    let settings_clone = settings.clone();
    println!("mqtt host.{}", settings_clone.mqtt.host.clone());


    // 启动定时任务(间隔10秒）
    tokio::spawn(async {
        let mut ticker = interval(Duration::from_secs(10));

        loop {
            ticker.tick().await;
            println!("定时任务执行: {:?}", chrono::Local::now());

            // TODO: 你的业务逻辑
        }
    });
    // ✅ 方案二：Tokio + sleep（更灵活）: 动态调度 / 不规则间隔
    tokio::spawn(async {
        loop {
            println!("执行任务");

            // 动态计算下一次执行时间
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
    // tokio-cron-scheduler = "0.10": cron 表达式
    let sched = JobScheduler::new().await.unwrap();
    sched.add(
        Job::new_async("0/10 * * * * *", |_uuid, _l| {
            Box::pin(async move {
                println!("每10秒执行一次");
            })
        }).unwrap(),
    ).await.unwrap();
    sched.start().await.unwrap();

    tokio::spawn({
        let state = shared_state.clone();
        async move {
            let mut ticker = interval(Duration::from_secs(10));
            loop {
                ticker.tick().await;
                state.increment_counter();
                let count = *state.counter.read().unwrap(); // 读取 counter
                // 使用 state
                println!("使用共享状态执行任务 ，共享状态={}",count);
            }
        }
    });

    // 测试 c库调用
    println!("{}", foo_safe::add(1, 2));
    println!("{}", foo_safe::hello().unwrap());
    println!("{}", foo_safe::alloc_string().unwrap());
    println!("{}", foo_safe::sqrt(1.4));

    // 测试本地命令执行
    // 自动选择平台
    let result = CommandExecutor::exec_auto("echo hello world")
        .expect("command failed");
    println!("stdout: {}", result.stdout);
    println!("stderr: {}", result.stderr);
    println!("status: {}", result.status);
    // 带超时
    let result = CommandExecutor::exec(
        "ping",
        if cfg!(windows) { &["127.0.0.1"] } else { &["-c", "1", "127.0.0.1"] },
        Some(Duration::from_secs(5)),
    );
    println!("ping result: {:?}", result);

    // 读取内嵌文件
    // 读取 Excel 数据
    match ExcelUtil::read_embedded("test.xlsx", "Sheet1") {
        Ok(data) => {
            println!("Read data: {:?}", data);
        }
        Err(e) => {
            println!("Failed to read Excel file: {}", e);
        }
    }

    // 写入 Excel 数据
    /*let data_to_write = vec![
        vec!["Header1".to_string(), "Header2".to_string()],
        vec!["Data1".to_string(), "Data2".to_string()],
    ];

    if let Err(e) = ExcelUtil::edit("report.xlsx", "Sheet1", data_to_write) {
        println!("Failed to write Excel file: {}", e);
    }*/

    info!("server started at http://0.0.0.0:3000");

    // 1. 构建 runtime
    /*let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get()) // 或 get_physical()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024)
        .build()
        .unwrap();

    // 2. 启动 async 主逻辑
    rt.block_on(async {
        bootstrap::server::start(settings.as_ref(),shared_state).await;
    });*/
    bootstrap::server::start(settings.as_ref(),shared_state).await;
}