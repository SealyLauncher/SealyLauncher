//! Core errors with user-safe variants.

use thiserror::Error;

/// Errors produced by the core crate.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Filesystem failure.
    #[error("文件系统错误: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization failure.
    #[error("配置解析失败: {0}")]
    Json(#[from] serde_json::Error),

    /// Another launcher instance already owns the portable data directory.
    #[error("SealyLauncher 已经在运行: {0}")]
    AlreadyRunning(String),

    /// Logging initialization failure.
    #[error("日志初始化失败: {0}")]
    Logging(String),

    /// Portable root could not be determined.
    #[error("无法确定启动器目录: {0}")]
    RootUnavailable(String),
}
