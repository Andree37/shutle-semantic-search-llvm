use std::fmt::Formatter;

#[derive(Debug)]
pub struct NotAvailableError;

impl std::error::Error for NotAvailableError {}

impl std::fmt::Display for NotAvailableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "File 'not available' error")
    }
}

#[derive(Debug)]
pub struct SetupError(pub &'static str);

impl std::error::Error for SetupError {}

impl std::fmt::Display for SetupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Setup error: {}", self.0)
    }
}