use std::fmt;

/// Enum containing all FTP errors the library uses
#[derive(Debug, Clone)]
pub enum FtpError {
    /// Invalid response recieved from the FTP server
    InvalidResponseError(crate::FtpResponse),
    /// Error connecting to the FTP server
    ConnectionError,
    /// Incorrect user credentials
    InvalidCredentialsError,
    /// Invalid request for the given status code
    InvalidTypeError,
    /// Issues connecting to the datastream (Passive)
    DatastreamConnectionError,
    /// Invalid response format recieved from the FTP server
    InvalidResponseFormatError,
}

impl fmt::Display for FtpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.as_string())
    }
}

impl FtpError {
    pub fn as_string(&self) -> String {
        match self {
            FtpError::InvalidResponseError(v) => {
                format!("Invalid response recieved: {:?}", v.clone())
            }
            FtpError::ConnectionError => "Error connecting to FTP server".to_string(),
            FtpError::InvalidCredentialsError => "Invalid credentials when logging in".to_string(),
            FtpError::InvalidTypeError => "An invalid request was made by the client".to_string(),
            FtpError::DatastreamConnectionError => {
                "Error connecting to the FTP datastream".to_string()
            }
            FtpError::InvalidResponseFormatError => {
                "An invalid FTP response format was sent by the server".to_string()
            }
        }
    }
}
