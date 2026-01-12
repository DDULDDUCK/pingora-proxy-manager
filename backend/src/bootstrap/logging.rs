use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    // 1. 로깅 초기화 (File + Stdout)
    let file_appender = tracing_appender::rolling::daily("logs", "access.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .json() // 파일에는 JSON으로 저장 (구조화된 로그)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .init();

    guard
}
