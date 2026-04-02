//AppError + Result

#[derive(Debug, thiserror::Error)]
pub enum AppError {}

pub type Result<T> = std::result::Result<T, AppError>;
