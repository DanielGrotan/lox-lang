use std::result;

pub type Result<T> = result::Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    ScriptNotFound,
    InvalidEncoding,
}
