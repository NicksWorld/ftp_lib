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
    /// Syntax error (general)
    SyntaxError(String),
    /// Syntax error (parameters)
    SyntaxErrorParameters(String),
    /// Command unimplemented
    CommandUnimplemented(String),
    /// Service unavailable
    ServiceUnavailable,
    /// Not logged in
    NotLoggedIn,
    /// No permission
    NoPermission,
    /// File is unavailable
    FileUnavailable,
    /// Account is required for the action
    AccountRequired,
    /// Invalid file name
    InvalidFileName,
    /// Bad sequence of commands given
    BadCommandSequence,
}

impl fmt::Display for FtpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.as_string())
    }
}

impl FtpError {
    /// Converts FtpError to a human readable error message.
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
            FtpError::SyntaxError(v) => format!("Invalid syntax in command: {:?}", v),
            FtpError::SyntaxErrorParameters(v) => {
                format!("Invalid syntax in command parameters: {:?}", v)
            }
            FtpError::CommandUnimplemented(v) => format!("Command is not implemented: {:?}", v),
            FtpError::ServiceUnavailable => "Service is unavailable at the moment".to_string(),
            FtpError::NotLoggedIn => "User is not authenticated with the server".to_string(),
            FtpError::NoPermission => {
                "User does not have permision to access the file or it does not exist".to_string()
            }
            FtpError::FileUnavailable => "The requested file was unavailable".to_string(),
            FtpError::AccountRequired => "The requested action requires an account".to_string(),
            FtpError::InvalidFileName => "The file name provided has an invalid name".to_string(),
            FtpError::BadCommandSequence => "Bad command sequence".to_string(),
        }
    }
}
