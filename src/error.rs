use crate::status::ftp_status;
use std::fmt;

/// Enum containing all FTP errors the library uses
#[derive(Debug, Clone)]
pub enum FtpError {
    /// Invalid response recieved from the FTP server
    InvalidResponseError(crate::FtpResponse),
    /// Invalid format of response
    InvalidResponseFormatError,
    /// Error connecting to the FTP server
    ConnectionError,
    /// Invalid request for the given status code
    InvalidTypeError,
    /// Issues connecting to the datastream (Passive)
    DatastreamConnectionError,
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
    /// File is unavailable (No permission, nonexistant)
    FileUnavailable,
    /// Account is required for the action
    AccountRequired,
    /// Invalid file name
    InvalidFileName,
    /// Bad sequence of commands given
    BadCommandSequence,
    /// Server not ready (Sent ready-in)
    ServiceNotReady,
    /// Action aborted by the server
    ActionAborted,
    /// Insufficient storage space
    InsufficientStorage,
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
            FtpError::InvalidResponseFormatError => "Invalid format of response".to_string(),
            FtpError::ConnectionError => "Error connecting to FTP server".to_string(),
            FtpError::InvalidTypeError => "An invalid request was made by the client".to_string(),
            FtpError::DatastreamConnectionError => {
                "Error connecting to the FTP datastream".to_string()
            }
            FtpError::SyntaxError(v) => format!("Invalid syntax in command: {:?}", v),
            FtpError::SyntaxErrorParameters(v) => {
                format!("Invalid syntax in command parameters: {:?}", v)
            }
            FtpError::CommandUnimplemented(v) => format!("Command is not implemented: {:?}", v),
            FtpError::ServiceUnavailable => "Service is unavailable at the moment".to_string(),
            FtpError::NotLoggedIn => "User is not authenticated with the server".to_string(),
            FtpError::FileUnavailable => "The requested file was unavailable".to_string(),
            FtpError::AccountRequired => "The requested action requires an account".to_string(),
            FtpError::InvalidFileName => "The file name provided has an invalid name".to_string(),
            FtpError::BadCommandSequence => "Bad command sequence".to_string(),
            FtpError::ServiceNotReady => "Service not ready".to_string(),
            FtpError::ActionAborted => "Action aborted by FTP server".to_string(),
            FtpError::InsufficientStorage => "Insufficient storage on server".to_string(),
        }
    }

    /// Converts a status code into the respected error.
    pub fn from_status_code(response: crate::FtpResponse, command: String) -> FtpError {
        use FtpError::*;
        match response.status {
            ftp_status::NOT_LOGGED_IN => NotLoggedIn,
            ftp_status::ACCOUNT_REQUIRED_LOGIN | ftp_status::ACCOUNT_REQUIRED_STORING => {
                AccountRequired
            }

            ftp_status::ACTION_NOT_TAKEN | ftp_status::FILE_ACTION_NOT_TAKEN => FileUnavailable,

            ftp_status::INSUFFICIENT_STORAGE | ftp_status::INSUFFICIENT_ALLOCATED_STORAGE => {
                InsufficientStorage
            }
            ftp_status::FILE_NAME_INVALID => InvalidFileName,

            ftp_status::DATA_CANNOT_CONNECT => DatastreamConnectionError,
            ftp_status::DATA_CLOSED_ABORTING
            | ftp_status::ACTION_ABORTED_UNKOWN_PAGE
            | ftp_status::ACTION_ABORTED_PROCESSING => ActionAborted,

            ftp_status::SYNTAX_ERROR => SyntaxError(command),
            ftp_status::SYNTAX_ERROR_ARGUMENTS => SyntaxErrorParameters(command),
            ftp_status::BAD_COMMAND_SEQUENCE => BadCommandSequence,

            ftp_status::COMMAND_NOT_IMPLEMENTED
            | ftp_status::COMMAND_NOT_IMPLEMENTED_UNNECESARY => CommandUnimplemented(command),

            ftp_status::SERVICE_NOT_AVAILABLE => ServiceUnavailable,

            _ => InvalidResponseError(response),
        }
    }
}
