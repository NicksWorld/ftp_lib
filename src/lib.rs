use std::net::TcpStream;
use std::net::SocketAddrV4;

use std::str::FromStr;

use std::io::{BufRead};
use std::io::BufReader;

mod error;
use error::*;

// TODO: Add an enum containing FTP status codes for use in response expectations

#[derive(Debug)]
pub struct FtpResponse {
	pub status: usize,
	pub content: String
}

impl FromStr for FtpResponse {
	type Err = error::InvalidResponseError;
	
	fn from_str(s: &str) -> Result<FtpResponse, error::InvalidResponseError> {
		// Make sure the recieved content is long enough to contain a status code
		if s.len() >= 3 {
			// Grab the first 3 characters of the string
			let status_code = &s[0..3];

			// Try to parse the first 3 characters as a usize as FTP status codes are 3 digits
			match status_code.parse::<usize>() {
				Ok(status) => Ok(FtpResponse {status: status, content: s.to_string()}),
				_ => Err(error::InvalidResponseError)
			}
		} else {
			Err(error::InvalidResponseError)
		}
	}
}


pub struct FtpConnection {
	reader: BufReader<TcpStream>
}

impl FtpConnection {
	/// Initiates a connection to the specified FTP server.
	///
	/// The connection is established over TCP and waits until a 230 response code is recieved, signifying a successful connection to the FTP server.
	///
	/// # Examples
	/// Initiates a connection to the FTP server at 4.31.198.44 (ftp.ietf.org).
	/// ```
	/// use std::net::SocketAddrV4;
	/// use ftp_lib::FtpConnection;
	///
	/// let ftp_connection = FtpConnection::connect("4.31.198.44:21".parse().unwrap());
	/// ```
	pub fn connect(connection_addr: SocketAddrV4) -> Result<FtpConnection, error::FtpErrors> {
		match TcpStream::connect(connection_addr) {
			Ok(stream) => {
				let mut ftp_conn = FtpConnection {
					reader: BufReader::new(stream)
				};
				
				let res = ftp_conn.wait_for_response();

				match res {
					Ok(ftp_response) if ftp_response.status == 220 => {Ok(ftp_conn)},
					Ok(_) => Err(FtpErrors::InvalidResponseError(InvalidResponseError)),
					Err(_) => Err(FtpErrors::InvalidResponseError(InvalidResponseError))
				}	
			},
			Err(_) => Err(FtpErrors::FtpConnectionError(FtpConnectionError))
		}
	}

	fn wait_for_response(&mut self) -> Result<FtpResponse, error::InvalidResponseError> {
		let mut response = String::from("");
		self.reader.read_line(&mut response).unwrap();

		FtpResponse::from_str(response.as_str())
	}
}


#[test]
fn test_connect() {
	use std::net::SocketAddrV4;
	use std::net::Ipv4Addr;
	
	FtpConnection::connect(
		SocketAddrV4::new(Ipv4Addr::new(4,31,198,44), 21)
	).unwrap();
}
