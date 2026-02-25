pub struct Error {
    pub message: String,
    pub status: Option<reqwest::StatusCode>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(status) = self.status {
            writeln!(f, "status: {}", status)?;
        }

        write!(f, "{}", self.message)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(status) = self.status {
            writeln!(f, "status: {}", status)?;
        }

        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error {
            status: value.status(),
            message: value.to_string(),
        }
    }
}
