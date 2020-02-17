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

/// Stores the status code recieved from the FTP server along with the whole message
///
/// "220 FTP server ready\r\n"
///     => FtpResponse {status: 220, content: "220 FTP server ready\r\n"}
#[derive(Debug)]
pub struct FtpResponse {
    pub status: usize,
    pub content: String,
}

impl FtpResponse {
    pub fn parse_pasv_addr(&self) -> Result<SocketAddrV4, FtpError> {
        if self.status != 227 {
            return Err(InvalidTypeError);
        }

        let pasv_raw = &self.content.as_str();
        let pasv_addr_section =
            &pasv_raw[pasv_raw.find("(").unwrap_or(0) + 1..pasv_raw.find(")").unwrap_or(0)];

        let pasv_unparsed: Vec<&str> = pasv_addr_section.split(",").collect();
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

    /// Converts a message from the FTP server to a instance of FtpResponse.
    ///
    /// The first three characters are the response code, the rest of the string contains potentially important data.
    ///
    /// # Examples
    /// Converts "220 FTP server ready\r\n" to a FtpResponse
    /// ```
    /// use std::str::FromStr;
    /// use ftp_lib::FtpResponse;
    ///
    /// let source_str = "220 FTP server ready\r\n";
    ///
    /// println!("{:?}", FtpResponse::from_str(source_str).unwrap());
    /// // => FtpResponse {status: 220, content: "220 FTP server ready\r\n"}
    /// ```
    fn from_str(s: &str) -> Result<FtpResponse, FtpError> {
        // Make sure the recieved content is long enough to contain a status code
        if s.len() >= 3 {
            // Grab the first 3 characters of the string
            let status_code = &s[0..3];

            // Try to parse the first 3 characters as a usize as FTP status codes are 3 digits
            match status_code.parse::<usize>() {
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

/// Main structure for handling the connection to a FTP server.
///
/// # Examples
/// ```
/// use std::net::SocketAddrV4;
/// use ftp_lib::FtpConnection;
///
/// let ftp_connection = FtpConnection::connect("4.31.198.44:21".parse().unwrap());
/// ```
pub struct FtpConnection {
    reader: BufReader<TcpStream>,
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
    /// let ftp_connection = FtpConnection::connect("4.31.198.44:21".parse().unwrap()).unwrap();
    /// ```
    pub fn connect(connection_addr: SocketAddrV4) -> Result<FtpConnection, FtpError> {
        match TcpStream::connect(connection_addr) {
            Ok(stream) => {
                let mut ftp_conn = FtpConnection {
                    reader: BufReader::new(stream),
                };

                let res = ftp_conn.wait_for_response();

                match res {
                    Ok(ftp_response) if ftp_response.status == 220 => Ok(ftp_conn),
                    Ok(_) => Err(InvalidResponseError),
                    Err(_) => Err(InvalidResponseError),
                }
            }
            Err(_) => Err(ConnectionError),
        }
    }

    /// Authenticates the client with the FTP server.
    ///
    /// # Examples
    /// Logs into the FTP server at 4.31.198.44
    /// ```
    /// use std::net::SocketAddrV4;
    /// use ftp_lib::FtpConnection;
    ///
    /// let mut ftp_connection = FtpConnection::connect("4.31.198.44:21".parse().unwrap()).unwrap();
    ///
    /// ftp_connection.login("anonymous".to_string(), Some("fake@email.service".to_string())).unwrap();
    /// ```
    pub fn login(&mut self, username: String, password: Option<String>) -> Result<(), FtpError> {
        self.write_command(format!("USER {}\r\n", username))?;
        let username_res = self.wait_for_response()?;
        match username_res.status {
            331 => {
                self.write_command(format!("PASS {}\r\n", password.unwrap_or("".to_string())))?;
                let password_res = self.wait_for_response()?;
                match password_res.status {
                    230 => Ok(()),
                    530 => Err(InvalidCredentialsError),
                    _ => Err(InvalidResponseError),
                }
            }
            _ => Err(InvalidResponseError),
        }
    }
    /// Lists all files in the current working directory.
    ///
    /// # Examples
    /// Lists all files in the default working directory of the server at 4.31.198.41 (ftp.ietf.org).
    /// ```
    /// use std::net::SocketAddrV4;
    /// use ftp_lib::FtpConnection;
    ///
    /// let mut ftp_connection = FtpConnection::connect("4.31.198.44:21".parse().unwrap()).unwrap();
    /// ftp_connection.login("anonymous".to_string(), Some("fake@email.service".to_string())).unwrap();
    ///
    /// let file_list = ftp_connection.list().unwrap();
    /// println!("{}", file_list);
    /// ```
    pub fn list(&mut self) -> Result<String, FtpError> {
        let datastream_addr = self.pasv()?;

        self.write_command("LIST\r\n".to_string())?;

        let datavec = self.connect_datastream(datastream_addr)?;

        match self.wait_for_response() {
            Ok(res) if res.status == 150 => (),
            _ => return Err(InvalidResponseError), // FIXME: Probably not the case, needs fix
        }

        match self.wait_for_response() {
            Ok(res) if res.status == 226 => (),
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

    /// Puts the FTP server into passive mode.
    ///
    /// Passive mode makes the next request using a datastream to send data through the specified port.
    fn pasv(&mut self) -> Result<SocketAddrV4, FtpError> {
        self.write_command("PASV\r\n".to_string())?;
        let pasv = self.wait_for_response()?;
        match pasv.status {
            227 => pasv.parse_pasv_addr(),
            _ => Err(InvalidResponseError),
        }
    }

    /// Writes a telnet command to the FTP server.
    ///
    /// The \r\n is still required at the end of the string.
    fn write_command(&mut self, command: String) -> Result<(), FtpError> {
        match self.reader.get_mut().write_all(command.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(ConnectionError),
        }
    }

    /// Waits until a telnet response is recieved from the FTP server.
    ///
    /// Ex: "220 FTP server ready.\r\n"
    fn wait_for_response(&mut self) -> Result<FtpResponse, FtpError> {
        let mut response = String::from("");
        match self.reader.read_line(&mut response) {
            Ok(_) => FtpResponse::from_str(&response),
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
