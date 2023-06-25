use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct NotAvailableError;

impl std::error::Error for NotAvailableError {}

impl Display for NotAvailableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "File 'not available' error")
    }
}

#[derive(Debug)]
pub struct SetupError(pub &'static str);

impl std::error::Error for SetupError {}

impl Display for SetupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Setup error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct EmbeddingError;

impl std::error::Error for EmbeddingError {}

impl Display for EmbeddingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Embedding error")
    }
}