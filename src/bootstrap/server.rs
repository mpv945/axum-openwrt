use axum::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use crate::AppRouter;
use crate::router::router;
use crate::config::Settings;
use crate::model::app_state::AppState;

pub async fn start(settings: &Settings, state: Arc<AppState>) {

    let app = router(state.clone());

    //let addr = SocketAddr::from(([0,0,0,0],8080));
    let addr = SocketAddr::new(
        settings.server.host.parse().unwrap(),
        settings.server.port
    );

    println!("Server running on {}", addr);

    /*axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app,
    )
    .await
    .unwrap();*/


    // 优雅关闭
    // 后台任务（例如定时任务）
    //let bg_task = tokio::spawn(background_worker(state.clone()));

    let server = axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app,
    ).with_graceful_shutdown(shutdown_signal());

    tokio::select! {
        _ = server => {
            println!("HTTP server 已停止");
        }
        /*_ = bg_task => {
            println!("后台任务异常退出");
        }*/
    }

    // 👇 统一资源清理
    let state_clean = state.clone();
    cleanup(state_clean.as_ref()).await;
    // 优雅关闭定时任务
    /*
    tokio-util = "0.7"

    use tokio_util::sync::CancellationToken;

let token = CancellationToken::new();

let bg = {
    let token = token.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    println!("任务退出");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    println!("执行任务");
                }
            }
        }
    })
};

shutdown 时：
token.cancel();
bg.await.ok();
     */

}

async fn shutdown_signal() {
    use tokio::signal;

    // Ctrl+C
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // Linux: SIGTERM（K8s / docker stop）
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("收到关闭信号，开始优雅关闭...");
}

async fn cleanup(state: &AppState) {
    println!("开始清理资源...");

    // 1️⃣ 关闭 DB 连接池（通常 drop 即可）
    // state.db_pool.close().await;

    // 2️⃣ flush Kafka / producer
    // state.kafka.flush().await;

    // 3️⃣ 停止定时任务（如果没用 channel）

    // 4️⃣ 关闭 Redis
    // state.redis.quit().await;

    println!("资源清理完成");
}