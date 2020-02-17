use std::error;
use std::fmt;

/// Enum containing all FTP errors the library uses
#[derive(Debug, Clone)]
pub enum FtpError {
    /// Invalid response recieved from the FTP server
    InvalidResponseError,
    /// Error connecting to the FTP server
    ConnectionError,
    /// Incorrect user credentials
    InvalidCredentialsError,
    /// Invalid request for the given status code
    InvalidTypeError,
    /// Issues connecting to the datastream (Passive)
    DatastreamConnectionError,
}

impl error::Error for FtpError {
    fn description(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for FtpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FtpError {
    pub fn as_str(&self) -> &str {
        match *self {
            FtpError::InvalidResponseError => "Invalid response recieved",
            FtpError::ConnectionError => "Error connecting to FTP server",
            FtpError::InvalidCredentialsError => "Invalid credentials when logging in",
            FtpError::InvalidTypeError => "An invalid request was made by the client",
            FtpError::DatastreamConnectionError => "Error connecting to the FTP datastream",
        }
    }
}
