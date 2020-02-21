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
use status::ftp_status;

#[derive(Debug, Clone)]
pub struct FtpResponse {
    pub status: u32,
    pub content: String,
}

impl FtpResponse {
    pub fn parse_pasv_addr(&self) -> Result<SocketAddrV4, FtpError> {
        if self.status != ftp_status::ENTERING_PASSIVE {
            return Err(InvalidTypeError);
        }

        let pasv_raw = &self.content.as_str();
        let pasv_addr_section =
            &pasv_raw[pasv_raw.find('(').unwrap_or(0) + 1..pasv_raw.find(')').unwrap_or(0)];

        let pasv_unparsed: Vec<&str> = pasv_addr_section.split(',').collect();
        if pasv_unparsed.len() != 6 {
            return Err(InvalidResponseError(self.clone()));
        }

        let mut octets = vec![];
        for number in pasv_unparsed[0..4].iter() {
            match number.parse::<u8>() {
                Ok(v) => octets.push(v),
                Err(_) => return Err(InvalidResponseError(self.clone())),
            }
        }

        let mut port_data = vec![];
        for number in pasv_unparsed[4..].iter() {
            match number.parse::<u16>() {
                Ok(v) => port_data.push(v),
                Err(_) => return Err(InvalidResponseError(self.clone())),
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
        if s.len() >= 3 {
            let status_code = &s[0..3];

            if format!("{} ", &status_code) != s[0..4] {
                return Err(InvalidResponseFormatError);
            }

            match status_code.parse::<u32>() {
                Ok(status) => Ok(FtpResponse {
                    status,
                    content: s.to_string(),
                }),
                _ => Err(InvalidResponseFormatError),
            }
        } else {
            Err(InvalidResponseFormatError)
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
                    ftp_status::SERVICE_READY => Ok(ftp_conn),
                    _ => Err(InvalidResponseError(res)),
                }
            }
            Err(_) => Err(ConnectionError),
        }
    }

    pub fn login(&mut self, username: String, password: Option<String>) -> Result<(), FtpError> {
        self.write_command(format!("USER {}\r\n", username))?;
        let username_res = self.wait_for_response()?;
        match username_res.status {
            ftp_status::PASSWORD_NEEDED => {
                self.write_command(format!(
                    "PASS {}\r\n",
                    password.unwrap_or_else(|| "".to_string())
                ))?;
                let password_res = self.wait_for_response()?;
                match password_res.status {
                    ftp_status::LOGGED_IN => Ok(()),
                    ftp_status::NOT_LOGGED_IN => Err(InvalidCredentialsError),
                    _ => Err(InvalidResponseError(password_res)),
                }
            }
            _ => Err(InvalidResponseError(username_res)),
        }
    }

    pub fn cwd(&mut self, path: &str) -> Result<(), FtpError> {
        let command = format!("CWD {}\r\n", path);
        self.write_command(command.clone())?;

        let cwd_result = self.wait_for_response()?;

        match cwd_result.status {
            // Successful action
            ftp_status::FILE_ACTION_COMPLETE => Ok(()),
            // Error completing action
            ftp_status::ACTION_NOT_TAKEN => Err(NoPermission),

            ftp_status::SYNTAX_ERROR => Err(SyntaxError(command)),
            ftp_status::SYNTAX_ERROR_ARGUMENTS => Err(SyntaxErrorParameters(command)),
            ftp_status::COMMAND_NOT_IMPLEMENTED => Err(CommandUnimplemented(command)),
            ftp_status::SERVICE_NOT_AVAILABLE => Err(ServiceUnavailable),
            ftp_status::NOT_LOGGED_IN => Err(NotLoggedIn),
            _ => Err(InvalidResponseError(cwd_result)),
        }
    }

    pub fn cdup(&mut self) -> Result<(), FtpError> {
        let command = "CDUP\r\n".to_string();
        self.write_command(command.clone())?;

        let cdup_result = self.wait_for_response()?;

        match cdup_result.status {
            // Successful action
            ftp_status::FILE_ACTION_COMPLETE => Ok(()),
            // Error completing action
            ftp_status::ACTION_NOT_TAKEN => Err(NoPermission),

            ftp_status::SYNTAX_ERROR => Err(SyntaxError(command)),
            ftp_status::SYNTAX_ERROR_ARGUMENTS => Err(SyntaxErrorParameters(command)),
            ftp_status::COMMAND_NOT_IMPLEMENTED => Err(CommandUnimplemented(command)),
            ftp_status::SERVICE_NOT_AVAILABLE => Err(ServiceUnavailable),
            ftp_status::NOT_LOGGED_IN => Err(NotLoggedIn),
            _ => Err(InvalidResponseError(cdup_result)),
        }
    }

    pub fn pwd(&mut self) -> Result<String, FtpError> {
        let command = "PWD\r\n".to_string();
        self.write_command(command.clone())?;

        let pwd_result = self.wait_for_response()?;

        match pwd_result.status {
            // Successful action
            ftp_status::DIRECTORY_CREATED => {
                let split_quote: Vec<&str> = pwd_result.content.split('"').collect();

                if split_quote.len() < 2 {
                    Err(InvalidResponseFormatError)
                } else {
                    Ok(split_quote[1].to_string())
                }
            }
            // Error completing action
            ftp_status::ACTION_NOT_TAKEN => Err(NoPermission),

            ftp_status::SYNTAX_ERROR => Err(SyntaxError(command)),
            ftp_status::SYNTAX_ERROR_ARGUMENTS => Err(SyntaxErrorParameters(command)),
            ftp_status::COMMAND_NOT_IMPLEMENTED => Err(CommandUnimplemented(command)),
            ftp_status::SERVICE_NOT_AVAILABLE => Err(ServiceUnavailable),
            ftp_status::NOT_LOGGED_IN => Err(NotLoggedIn),
            _ => Err(InvalidResponseError(pwd_result)),
        }
    }

    pub fn mkd(&mut self, dir_name: &str) -> Result<(), FtpError> {
        let command = format!("MKD {}\r\n", dir_name);
        self.write_command(command.clone())?;

        let mkd_result = self.wait_for_response()?;

        match mkd_result.status {
            // Successful action
            ftp_status::DIRECTORY_CREATED => Ok(()),
            // Error completing action
            ftp_status::ACTION_NOT_TAKEN => Err(NoPermission),

            ftp_status::SYNTAX_ERROR => Err(SyntaxError(command)),
            ftp_status::SYNTAX_ERROR_ARGUMENTS => Err(SyntaxErrorParameters(command)),
            ftp_status::COMMAND_NOT_IMPLEMENTED => Err(CommandUnimplemented(command)),
            ftp_status::SERVICE_NOT_AVAILABLE => Err(ServiceUnavailable),
            ftp_status::NOT_LOGGED_IN => Err(NotLoggedIn),
            _ => Err(InvalidResponseError(mkd_result)),
        }
    }

    pub fn retr(&mut self, file_name: &str) -> Result<Vec<u8>, FtpError> {
        let datastream_addr = self.pasv()?;

        let command = format!("RETR {}\r\n", file_name);
        self.write_command(command.clone())?;

        let data = self.connect_datastream(datastream_addr)?;

        // NOTE: The following statuses do not cover all cases.
        // I am not sure where status can be returned for now.
        let res = self.wait_for_response()?;
        match res.status {
            ftp_status::FILE_OPENING_DATA => (),
            _ => return Err(InvalidResponseError(res)),
        }

        let res = self.wait_for_response()?;
        match res.status {
            ftp_status::DATA_CLOSING => (),
            _ => return Err(InvalidResponseError(res)),
        }

        Ok(data)
    }

    pub fn rename_file(&mut self, file: String, new_name: String) -> Result<(), FtpError> {
        let command = format!("RNFR {}\r\n", file);
        self.write_command(command.clone())?;

        let rnfr_result = self.wait_for_response()?;
        match rnfr_result.status {
            // Successful action
            ftp_status::FILE_ACTION_COMPLETE | ftp_status::FILE_NEED_INFORMATION => {
                let command = format!("RNTO {}\r\n", new_name);
                self.write_command(command.clone())?;

                let rnto_result = self.wait_for_response()?;

                match rnto_result.status {
                    // Successful action
                    ftp_status::FILE_ACTION_COMPLETE => Ok(()),
                    // Error completing action
                    ftp_status::ACCOUNT_REQUIRED_STORING => Err(AccountRequired),
                    ftp_status::FILE_NAME_INVALID => Err(InvalidFileName),
                    // TODO: Probably incomplete
                    ftp_status::SYNTAX_ERROR => Err(SyntaxError(command)),
                    ftp_status::SYNTAX_ERROR_ARGUMENTS => Err(SyntaxErrorParameters(command)),

                    _ => Err(InvalidResponseError(rnto_result)),
                }
            }
            // Error completing action
            ftp_status::FILE_ACTION_NOT_TAKEN => Err(FileUnavailable),
            ftp_status::ACTION_NOT_TAKEN => Err(FileUnavailable),

            ftp_status::SYNTAX_ERROR => Err(SyntaxError(command)),
            ftp_status::SYNTAX_ERROR_ARGUMENTS => Err(SyntaxErrorParameters(command)),
            ftp_status::COMMAND_NOT_IMPLEMENTED => Err(CommandUnimplemented(command)),
            ftp_status::SERVICE_NOT_AVAILABLE => Err(ServiceUnavailable),
            ftp_status::NOT_LOGGED_IN => Err(NotLoggedIn),
            _ => Err(InvalidResponseError(rnfr_result)),
        }
    }

    pub fn remove_directory(&mut self, directory: String) -> Result<(), FtpError> {
        self.write_command(format!("RMD {}\r\n", directory))?;

        let res = self.wait_for_response()?;
        match res.status {
            ftp_status::FILE_ACTION_COMPLETE => Ok(()),
            _ => Err(InvalidResponseError(res)),
        }
    }

    pub fn remove_file(&mut self, file: String) -> Result<(), FtpError> {
        self.write_command(format!("DELE {}\r\n", file))?;

        let res = self.wait_for_response()?;
        match res.status {
            ftp_status::COMMAND_OKAY => Ok(()),
            _ => Err(InvalidResponseError(res)),
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

            let res = self.wait_for_response()?;
            match res.status {
                150 => (),
                _ => return Err(InvalidResponseError(res)),
            }

            let res = self.wait_for_response()?;
            match res.status {
                226 => (),
                _ => return Err(InvalidResponseError(res)),
            }
        }

        Err(ConnectionError)
    }

    pub fn list(&mut self) -> Result<String, FtpError> {
        let datastream_addr = self.pasv()?;

        self.write_command("LIST\r\n".to_string())?;

        let datavec = self.connect_datastream(datastream_addr)?;

        let res = self.wait_for_response()?;
        match res.status {
            ftp_status::FILE_OPENING_DATA => (),
            _ => return Err(InvalidResponseError(res)),
        }

        let res = self.wait_for_response()?;
        match res.status {
            ftp_status::DATA_CLOSING => (),
            _ => return Err(InvalidResponseError(res)),
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
            ftp_status::ENTERING_PASSIVE => pasv.parse_pasv_addr(),
            _ => Err(InvalidResponseError(pasv)),
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

    match FtpConnection::connect(SocketAddrV4::new(Ipv4Addr::new(4, 31, 198, 44), 21)) {
        Ok(mut ftp_conn) => {
            match ftp_conn.login(
                "anonymous".to_string(),
                Some("fake@email.service".to_string()),
            ) {
                Ok(_) => (),
                Err(e) => println!("{}", e),
            }

            ftp_conn.cwd("rfc").unwrap();

            println!("{}", ftp_conn.list().unwrap());
        }
        Err(e) => println!("{}", e),
    }
}
