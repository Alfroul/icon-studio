use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Build error: {0}")]
    BuildError(String),

    #[error("Render error: {0}")]
    RenderError(String),

    #[error("Export error: {0}")]
    ExportError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFoundError(String),

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::ValidationError(s)
    }
}

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

impl From<AppError> for rmcp::model::ErrorData {
    fn from(err: AppError) -> Self {
        match &err {
            AppError::ValidationError(_) | AppError::NotFoundError(_) => {
                rmcp::model::ErrorData::invalid_params(err.to_string(), None)
            }
            _ => rmcp::model::ErrorData::internal_error(err.to_string(), None),
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
