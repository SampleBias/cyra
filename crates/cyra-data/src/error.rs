use std::fmt;

#[derive(Debug)]
pub struct DataError {
    pub exit_code: u8,
    pub code: &'static str,
    pub message: String,
}

impl DataError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            exit_code: 2,
            code: "invalid_arguments",
            message: message.into(),
        }
    }
    pub fn ineligible(message: impl Into<String>) -> Self {
        Self {
            exit_code: 5,
            code: "ineligible_data",
            message: message.into(),
        }
    }
    pub fn io(message: impl Into<String>) -> Self {
        Self {
            exit_code: 8,
            code: "io_failure",
            message: message.into(),
        }
    }
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            exit_code: 3,
            code: "unsupported_operation",
            message: message.into(),
        }
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DataError {}
