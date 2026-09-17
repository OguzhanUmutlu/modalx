use thiserror::Error;

/// Error type for modalx operations.
#[derive(Error, Debug)]
pub enum TuiError {
    /// Standard I/O or terminal stream error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal sizing or capability constraint violation.
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Interaction was canceled by the user (e.g. Esc or Ctrl+C).
    #[error("Interaction was canceled by the user")]
    Canceled,

    /// Validation error during input prompt.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Generic error message.
    #[error("{0}")]
    Other(String),
}

/// Convenience Result alias for modalx operations.
pub type Result<T> = std::result::Result<T, TuiError>;
