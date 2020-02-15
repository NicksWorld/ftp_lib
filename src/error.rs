use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum FtpErrorType {
	/// Invalid response recieved from the FTP server
	InvalidResponseError,
	/// Error connecting to the FTP server
	FtpConnectionError
}

impl FtpErrorType {
	pub fn as_str(&self) -> &'static str {
		match *self {
			FtpErrorType::InvalidResponseError => "Invalid response recieved",
			FtpErrorType::FtpConnectionError => "Error connecting to FTP server"
		}
	}
}

#[derive(Debug, Clone)]
pub struct FtpError {
	err_type: FtpErrorType
}

impl FtpError {
	pub fn new(kind: FtpErrorType) -> FtpError {
		FtpError {
			err_type: kind
		}
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
