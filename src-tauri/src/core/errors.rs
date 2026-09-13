use thiserror::Error;

#[derive(Debug, Error, serde::Serialize)]
pub enum AppError {
    #[error("Error de instancia: {0}")]
    Instance(String),
    #[error("Error de autenticación: {0}")]
    Auth(String),
    #[error("Error de filesystem: {0}")]
    Fs(String),
    #[error("Error de descarga: {0}")]
    Download(String),
    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Fs(e.to_string())
    }
}
