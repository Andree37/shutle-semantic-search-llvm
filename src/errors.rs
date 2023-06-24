use std::fmt::Formatter;

#[derive(Debug)]
pub struct NotAvailableError;

impl std::error::Error for NotAvailableError {}

impl std::fmt::Display for NotAvailableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "File 'not available' error")
    }
}