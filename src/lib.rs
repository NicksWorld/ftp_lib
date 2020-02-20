use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::net::TcpStream;

use std::str::FromStr;

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

pub mod error;
use error::FtpError;
use error::FtpError::*;

pub mod status;
use status::FtpStatus;

#[derive(Debug)]
pub struct FtpResponse {
	pub status: u32,
	pub content: String,
}

impl FtpResponse {
    pub fn parse_pasv_addr(&self) -> Result<SocketAddrV4, FtpError> {
            if self.status != FtpStatus::EnteringPassive {
		    return Err(InvalidTypeError);
            }

            let pasv_raw = &self.content.as_str();
            let pasv_addr_section =
		    &pasv_raw[pasv_raw.find('(').unwrap_or(0) + 1..pasv_raw.find(')').unwrap_or(0)];

            let pasv_unparsed: Vec<&str> = pasv_addr_section.split(',').collect();
            if pasv_unparsed.len() != 6 {
		    return Err(InvalidResponseError);
            }

            let mut octets = vec![];
            for number in pasv_unparsed[0..4].iter() {
		    match number.parse::<u8>() {
			    Ok(v) => octets.push(v),
			    Err(_) => return Err(InvalidResponseError),
		    }
            }

            let mut port_data = vec![];
            for number in pasv_unparsed[4..].iter() {
		    match number.parse::<u16>() {
			    Ok(v) => port_data.push(v),
			    Err(_) => return Err(InvalidResponseError),
		    }
            }

            let datastream_addr = SocketAddrV4::new(
		    Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]),
		    (port_data[0] * 256) + port_data[1],
            );
            Ok(datastream_addr)
    }
}

impl FromStr for FtpResponse {
	type Err = FtpError;

	fn from_str(s: &str) -> Result<FtpResponse, FtpError> {
		// Make sure the recieved content is long enough to contain a status code
		if s.len() >= 3 {
			// Grab the first 3 characters of the string
			let status_code = &s[0..3];

			if format!("{} ", &status_code) != s[0..4] {
				return Err(InvalidResponseError);
			}

			// Try to parse the first 3 characters as a usize as FTP status codes are 3 digits
			match status_code.parse::<u32>() {
				Ok(status) => Ok(FtpResponse {
					status,
					content: s.to_string(),
				}),
				_ => Err(InvalidResponseError),
			}
		} else {
			Err(InvalidResponseError)
		}
	}
}

pub struct FtpConnection {
    reader: BufReader<TcpStream>,
}

impl FtpConnection {
    pub fn connect(connection_addr: SocketAddrV4) -> Result<FtpConnection, FtpError> {
        match TcpStream::connect(connection_addr) {
            Ok(stream) => {
                let mut ftp_conn = FtpConnection {
                    reader: BufReader::new(stream),
                };

                let res = ftp_conn.wait_for_response()?;

                match res.status {
			FtpStatus::ServiceReady => Ok(ftp_conn),
			_ => Err(InvalidResponseError),
                }
            }
            Err(_) => Err(ConnectionError),
        }
    }

    pub fn login(&mut self, username: String, password: Option<String>) -> Result<(), FtpError> {
        self.write_command(format!("USER {}\r\n", username))?;
        let username_res = self.wait_for_response()?;
        match username_res.status {
            FtpStatus::PasswordNeeded => {
                self.write_command(format!(
                    "PASS {}\r\n",
                    password.unwrap_or_else(|| "".to_string())
                ))?;
                let password_res = self.wait_for_response()?;
                match password_res.status {
                    FtpStatus::LoggedIn => Ok(()),
                    FtpStatus::NotLoggedIn => Err(InvalidCredentialsError),
                    _ => Err(InvalidResponseError),
                }
            }
            _ => Err(InvalidResponseError),
        }
    }

    /// Changes the working directory
    pub fn change_working_directory(&mut self, path: &str) -> Result<(), FtpError> {
        self.write_command(format!("CWD {}\r\n", path))?;
        let cd_result = self.wait_for_response()?;

        match cd_result.status {
            FtpStatus::FileActionComplete => Ok(()),
            _ => Err(InvalidResponseError), // FIXME: Add other possible responses.
        }
    }

    pub fn cd_up(&mut self) -> Result<(), FtpError> {
        self.write_command("CDUP\r\n".to_string())?;
        let res = self.wait_for_response()?;

        match res.status {
            FtpStatus::FileActionComplete => Ok(()),
            _ => Err(InvalidResponseError),
        }
    }

    pub fn pwd(&mut self) -> Result<String, FtpError> {
        self.write_command("PWD\r\n".to_string())?;
        let res = self.wait_for_response()?;
        match res.status {
            FtpStatus::DirectoryCreated => {
                // Fetch the path
                let split: Vec<&str> = res.content.split('"').collect();
                if split.len() < 2 {
                    Err(InvalidResponseError)
                } else {
                    Ok(split[1].to_string())
                }
            }
            _ => Err(InvalidResponseError),
        }
    }

    pub fn mkdir(&mut self, dir_name: String) -> Result<(), FtpError> {
        self.write_command(format!("MKD {}\r\n", dir_name))?;
        let res = self.wait_for_response()?;
        match res.status {
            FtpStatus::DirectoryCreated => Ok(()),
            _ => Err(InvalidResponseError),
        }
    }

    pub fn fetch_file(&mut self, file_name: String) -> Result<Vec<u8>, FtpError> {
        let datastream_addr = self.pasv()?;

        self.write_command(format!("RETR {}\r\n", file_name))?;

        let datavec = self.connect_datastream(datastream_addr)?;

        match self.wait_for_response()?.status {
            FtpStatus::FileOpeningData => (),
            _ => return Err(InvalidResponseError)
        }

        match self.wait_for_response()?.status {
            FtpStatus::DataClosing => (),
            _ => return Err(InvalidResponseError), // FIXME: probably incorrect
        }

        Ok(datavec)
    }

    pub fn rename_file(&mut self, file: String, new_name: String) -> Result<(), FtpError> {
        self.write_command(format!("RNFR {}\r\n", file))?;

        let start_res = self.wait_for_response()?;
        match start_res.status {
            FtpStatus::FileActionComplete => {
                self.write_command(format!("RNTO {}\r\n", new_name))?;
                let end_res = self.wait_for_response()?;
                match end_res.status {
                    FtpStatus::FileActionComplete => Ok(()),
                    _ => Err(InvalidResponseError),
                }
            }
            _ => Err(InvalidResponseError),
        }
    }

    pub fn remove_directory(&mut self, directory: String) -> Result<(), FtpError> {
        self.write_command(format!("RMD {}\r\n", directory))?;

        let res = self.wait_for_response()?;
        match res.status {
            FtpStatus::FileActionComplete => Ok(()),
            _ => Err(InvalidResponseError),
        }
    }

    pub fn remove_file(&mut self, file: String) -> Result<(), FtpError> {
        self.write_command(format!("DELE {}\r\n", file))?;

        let res = self.wait_for_response()?;
        match res.status {
            FtpStatus::CommandOkay => Ok(()),
            _ => Err(InvalidResponseError),
        }
    }

    pub fn write_file(&mut self, file_name: String, data: Vec<u8>) -> Result<(), FtpError> {
        let datastream_addr = self.pasv()?;

        self.write_command(format!("STOR {}\r\n", file_name))?;

        if let Ok(mut datastream) = TcpStream::connect(datastream_addr) {
            let data_write_res = datastream.write_all(&data);
            match data_write_res {
                Ok(_) => (),
                Err(_) => return Err(ConnectionError),
            }

            match datastream.shutdown(std::net::Shutdown::Both) {
                Ok(_) => (),
                Err(_) => return Err(DatastreamConnectionError),
            }

            match self.wait_for_response() {
                Ok(res) if res.status == 150 => (),
                _ => return Err(InvalidResponseError), // FIXME: Probably not the case, needs fix
            }

            match self.wait_for_response() {
                Ok(res) if res.status == 226 => (),
                _ => return Err(InvalidResponseError), // FIXME: probably incorrect
            }
        }

        Err(ConnectionError)
    }

    pub fn list(&mut self) -> Result<String, FtpError> {
        let datastream_addr = self.pasv()?;

        self.write_command("LIST\r\n".to_string())?;

        let datavec = self.connect_datastream(datastream_addr)?;

        match self.wait_for_response()?.status {
            FtpStatus::FileOpeningData => (),
            _ => return Err(InvalidResponseError), // FIXME: Probably not the case, needs fix
        }

        match self.wait_for_response()?.status {
            FtpStatus::DataClosing => (),
            _ => return Err(InvalidResponseError), // FIXME: probably incorrect
        }

        Ok(String::from_utf8_lossy(&datavec).to_string())
    }

    fn connect_datastream(&self, datastream_addr: SocketAddrV4) -> Result<Vec<u8>, FtpError> {
        match TcpStream::connect(datastream_addr) {
            Ok(mut datastream) => {
                let mut datavec = vec![];
                match datastream.read_to_end(&mut datavec) {
                    Ok(_) => (),
                    Err(_) => return Err(DatastreamConnectionError),
                }

                match datastream.shutdown(std::net::Shutdown::Both) {
                    Ok(_) => (),
                    Err(_) => return Err(DatastreamConnectionError),
                }

                Ok(datavec)
            }
            Err(_) => Err(DatastreamConnectionError),
        }
    }

    fn pasv(&mut self) -> Result<SocketAddrV4, FtpError> {
        self.write_command("PASV\r\n".to_string())?;
        let pasv = self.wait_for_response()?;
        match pasv.status {
            FtpStatus::EnteringPassive => pasv.parse_pasv_addr(),
            _ => Err(InvalidResponseError),
        }
    }

    fn write_command(&mut self, command: String) -> Result<(), FtpError> {
        match self.reader.get_mut().write_all(command.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ConnectionError),
        }
    }

    fn wait_for_response(&mut self) -> Result<FtpResponse, FtpError> {
        let mut response = String::from("");
        match self.reader.read_line(&mut response) {
            Ok(_) => {
                match FtpResponse::from_str(&response) {
                    Ok(v) => Ok(v),
                    Err(_) => {
                        // Process multiline reply
                        let expected_end = format!("{} ", &response[0..3]);
                        while response.len() < 5 || response[0..4] != expected_end {
                            response.clear();
                            match self.reader.read_line(&mut response) {
                                Ok(_) => (),
                                Err(_) => return Err(ConnectionError),
                            }
                        }
                        FtpResponse::from_str(&response)
                    }
                }
            }
            Err(_) => Err(ConnectionError),
        }
    }
}


#[test]
fn test_connect() {
    use std::net::Ipv4Addr;
    use std::net::SocketAddrV4;

    let mut ftp_conn =
        FtpConnection::connect(SocketAddrV4::new(Ipv4Addr::new(4, 31, 198, 44), 21)).unwrap();

    ftp_conn
        .login(
            "anonymous".to_string(),
            Some("fake@email.service".to_string()),
        )
        .unwrap();
    let working_dir = ftp_conn.list().unwrap();
    println!("{}", working_dir);
}
