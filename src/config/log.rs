use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{Duration as ChronosDuration, Local, TimeZone};
use time::macros::format_description;
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing_subscriber::fmt::time::OffsetTime;

const LOG_DIR: &str = "logs";
const LOG_FILE_PREFIX: &str = "app.log";
const MAX_LOG_FILES: usize = 10;

/// 初始化 tracing
pub fn init_tracing() -> WorkerGuard {
    fs::create_dir_all(LOG_DIR).expect("failed to create log directory");

    // 启动时先清理一次
    cleanup_old_logs(LOG_DIR, LOG_FILE_PREFIX, MAX_LOG_FILES);

    let file_appender = tracing_appender::rolling::daily(LOG_DIR, LOG_FILE_PREFIX);
    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,axum=info"));

    let local_offset = time::UtcOffset::current_local_offset()
        .unwrap_or(time::UtcOffset::UTC);
    // let offset = time::UtcOffset::from_hms(8, 0, 0).unwrap();

    let timer = OffsetTime::new(
        local_offset,
        format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
        ),
    );

    let console_layer = fmt::layer()
        .with_timer(timer.clone())
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .json();
    let file_layer = fmt::layer()
        .with_timer(timer)
        .with_ansi(false)
        // .without_time() // 如果不需要时间戳
        .with_target(true)  // 显示模块名称
        .with_thread_ids(true) // 显示线程ID
        .with_thread_names(true) // 显示线程名
        .with_file(true) // 显示文件名
        .with_line_number(true) // 显示行号
        .with_writer(non_blocking_writer)
        // .with_writer(non_blocking) // 同时输出到文件
        .json();
        //.compact();  // 不输出 JSON，输出简单格式

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    guard
}

/// 启动一个后台定时任务：每天本地时间 00:00:00 清理一次日志
pub fn spawn_daily_log_cleanup() {
    tokio::spawn(async move {
        loop {
            let wait_duration = duration_until_next_midnight();

            info!(
                "daily log cleanup task scheduled, next run after {:?}",
                wait_duration
            );

            sleep(wait_duration).await;

            info!("start daily log cleanup");

            let result = tokio::task::spawn_blocking(|| {
                cleanup_old_logs(LOG_DIR, LOG_FILE_PREFIX, MAX_LOG_FILES);
            })
                .await;

            match result {
                Ok(_) => info!("finish daily log cleanup"),
                Err(err) => error!("daily log cleanup task join error: {}", err),
            }
        }
    });
}

/// 计算距离下一个本地午夜 00:00:00 还有多久
fn duration_until_next_midnight() -> Duration {
    let now = Local::now();
    let tomorrow = now.date_naive() + ChronosDuration::days(1);
    let next_midnight = tomorrow.and_hms_opt(0, 0, 0).expect("invalid midnight");
    let next_midnight = Local
        .from_local_datetime(&next_midnight)
        .single()
        .expect("failed to get local midnight datetime");

    let diff = next_midnight - now;

    match diff.to_std() {
        Ok(d) => d,
        Err(_) => Duration::from_secs(60),
    }
}

/// 清理旧日志，只保留最近 max_files 个
pub fn cleanup_old_logs(log_dir: &str, file_prefix: &str, max_files: usize) {
    let dir = Path::new(log_dir);
    if !dir.exists() {
        return;
    }

    let mut log_files: Vec<(PathBuf, SystemTime)> = match fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| is_target_log_file(path, file_prefix))
            .filter_map(|path| {
                let modified = match fs::metadata(&path).and_then(|m| m.modified()) {
                    Ok(m) => m,
                    Err(err) => {
                        error!("failed to get modified time for {:?}: {}", path, err);
                        return None;
                    }
                };
                Some((path, modified))
            })
            .collect(),
        Err(err) => {
            error!("failed to read log dir {}: {}", log_dir, err);
            return;
        }
    };

    if log_files.len() <= max_files {
        info!(
            "log cleanup skipped, current file count = {}, max = {}",
            log_files.len(),
            max_files
        );
        return;
    }

    // 最新文件排前面
    log_files.sort_by(|a, b| b.1.cmp(&a.1));

    let files_to_delete: Vec<PathBuf> = log_files
        .into_iter()
        .skip(max_files)
        .map(|(path, _)| path)
        .collect();

    for path in files_to_delete {
        match fs::remove_file(&path) {
            Ok(_) => info!("removed old log file {:?}", path),
            Err(err) => error!("failed to remove old log file {:?}: {}", path, err),
        }
    }
}

/// 判断是否为目标日志文件
fn is_target_log_file(path: &Path, file_prefix: &str) -> bool {
    let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };

    file_name == file_prefix || file_name.starts_with(&format!("{file_prefix}."))
}