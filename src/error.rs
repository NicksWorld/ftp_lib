use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum FtpErrors {
	InvalidResponseError(InvalidResponseError),
	FtpConnectionError(FtpConnectionError)
}

#[derive(Debug, Clone)]
pub struct InvalidResponseError;

impl fmt::Display for InvalidResponseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Invalid response was recieved from the FTP server.")
	}
}

impl error::Error for InvalidResponseError {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		None
	}
}


#[derive(Debug, Clone)]
pub struct FtpConnectionError;

impl fmt::Display for FtpConnectionError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "There was an error connecting to the FTP server.")
	}
}

impl error::Error for FtpConnectionError {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		None
	}
}
