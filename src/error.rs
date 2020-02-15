use std::error;
use std::fmt;

/// Enum containing all FTP errors the library uses
#[derive(Debug, Clone)]
pub enum FtpErrorType {
    /// Invalid response recieved from the FTP server
    InvalidResponseError,
    /// Error connecting to the FTP server
    FtpConnectionError,
    /// Incorrect user credentials
    InvalidCredentialsError,
    /// Invalid request for the given status code
    InvalidTypeError,
}

impl FtpErrorType {
    /// Converts the FtpErrorType to a string for displaying
    pub fn as_str(&self) -> &'static str {
        match *self {
            FtpErrorType::InvalidResponseError => "Invalid response recieved",
            FtpErrorType::FtpConnectionError => "Error connecting to FTP server",
            FtpErrorType::InvalidCredentialsError => "Invalid credentials when logging in",
            FtpErrorType::InvalidTypeError => "An invalid request was made by the client",
        }
    }
}

/// The main struct holding the errors related to FTP
#[derive(Debug, Clone)]
pub struct FtpError {
    err_type: FtpErrorType,
}

impl FtpError {
    /// Creates an FtpError of type `kind`
    ///
    /// # Examples
    /// Creates a new instance of a FtpConnectionError
    /// ```
    /// use ftp_lib::error::{FtpError, FtpErrorType};
    ///
    /// let error = FtpError::new(FtpErrorType::FtpConnectionError);
    /// println!("{}", error);
    /// ```
    pub fn new(kind: FtpErrorType) -> FtpError {
        FtpError { err_type: kind }
    }
}

impl fmt::Display for FtpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.err_type.as_str())
    }
}

impl error::Error for FtpError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
