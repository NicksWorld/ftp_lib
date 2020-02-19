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

            if format!("{} ", &status_code) != &s[0..4] {
                return Err(InvalidResponseError);
            }

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

pub enum Ftp {
    /// Command okay.
    CommandOkay = 200,
    /// Syntax error, command unrecognized.
    ///
    /// This may include errors such as command line to long.
    SyntaxError = 500,
    /// Syntax error in parameters or arguments.
    SyntaxErrorArguments = 501,
    /// Command not implemented, superfluous at this site.
    CommandNotImplementedUnnecesary = 202,
    /// Command not implemented.
    CommandNotImplemented = 502,
    /// Bad sequence of commands.
    BadCommandSequence = 503,
    /// Command not implemented for that parameter.
    CommandNotImplementedParameter = 504,
    /// Restart marker reply.
    /// In this case, the text is exact and not left to the
    /// particular implementation; it must read:
    ///     MARK yyyy = mmmm
    /// Wthere yyyy is User-process data stream market, and mmmm
    /// server's equivelent market (note the space between markers and "=")
    RestartMarkerReply = 110,
    /// System status, or system help reply.
    SystemStatus = 211,
    /// Directory status.
    DirectoryStatus = 212,
    /// File status.
    FileStatus = 213,
    /// Help message.
    /// On how to use the server or the meaning of a particular
    /// non-standard command. This reply is useful only to the
    /// human user.
    HelpMessage = 214,
    /// NAME system type.
    /// Where NAME is an official system name from the list in the
    /// Assigned Numbers document.
    SystemType = 215,
    /// Service ready in nnn minutes.
    ReadyIn = 120,
    /// Servuce ready for new user.
    ServiceReady = 220,
    /// Service closing control connection.
    /// Logged out if approperiate.
    ServerClosingControl = 221,
    /// Service not available, closing control connection.
    /// This may be a reply to any command if the service knows it
    /// must shut down.
    ServiceNotAvailable = 421,
    /// Data connection already open; transfer starting.
    DataTransferStarting = 125,
    /// Data connection open; no transfer in progress.
    DataNotTransfering = 225,
    /// Can't open data connection.
    DataCannotConnect = 425,
    /// Closing data connection.
    /// Requested file action successful (for example, file
    /// transfer or file abort).
    DataClosing = 226,
    /// Connection closed; transfer aborted.
    DataClosedAborting = 426,
    /// Entering Passive Mode (h1,h2,h3,h4,p1,p2).
    EnteringPassive = 227,
    /// User logged in, proceed.
    LoggedIn = 230,
    /// Not logged in.
    NotLoggedIn = 530,
    /// User name okay, need password.
    PasswordNeeded = 331,
    /// Need account for login.
    AccountRequiredLogin = 332,
    /// Need account for storing files.
    AccountRequiredStoring = 532,
    /// File status okay; about to open the data connection.
    FileOpeningData = 150,
    /// Requested file action okay, completed.
    FileActionComplete = 250,
    /// "PATHNAME" created.
    DirectoryCreated = 257,
    /// Requested file action pending further information.
    FileNeedInformation = 350,
    /// File action not taken.
    /// File unavailable (e.g., file busy)
    FileActionNotTaken = 450,
    /// Requested action not taken.
    /// File unavailable (e.g., file not found, no access).
    ActionNotTaken = 550,
    /// Requested action aborted. Local error in processing.
    ActionAbortedProccessing = 451,
    /// Requested action aborted. Page type unknown.
    ActionAbortedUnknownPage = 551,
    /// Requested action not taken.
    /// Insufficient storage space in system.
    InsufficientStorage = 452,
    /// Requested file action aborted.
    /// Exceeded storage allocation (for current directory or
    /// dataset).
    InsufficientAllocatedStorage = 552,
    /// Requsted action not taken.
    /// File name not allowed.
    FileNameInvalid = 553,
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

    // FIXME: Support ACCT
    pub fn login(&mut self, username: String, password: Option<String>) -> Result<(), FtpError> {
        self.write_command(format!("USER {}\r\n", username))?;
        let username_res = self.wait_for_response()?;
        match username_res.status {
            331 => {
                self.write_command(format!(
                    "PASS {}\r\n",
                    password.unwrap_or_else(|| "".to_string())
                ))?;
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

    /// Changes the working directory
    pub fn change_working_directory(&mut self, path: &str) -> Result<(), FtpError> {
        self.write_command(format!("CWD {}\r\n", path))?;
        let cd_result = self.wait_for_response()?;

        match cd_result.status {
            250 => Ok(()),
            _ => Err(InvalidResponseError), // FIXME: Add other possible responses.
        }
    }

    pub fn cd_up(&mut self) -> Result<(), FtpError> {
        self.write_command("CDUP\r\n".to_string())?;
        let res = self.wait_for_response()?;

        match res.status {
            250 => Ok(()),
            _ => Err(InvalidResponseError),
        }
    }

    pub fn pwd(&mut self) -> Result<String, FtpError> {
        self.write_command("PWD\r\n".to_string())?;
        let res = self.wait_for_response()?;
        match res.status {
            257 => {
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
            257 => Ok(()),
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

    /// Connects to a passive datastream
    ///
    /// Passive mode opens a port on the host for the client to connect to.
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
fn test_connect() -> Result<(), FtpError> {
    use std::net::Ipv4Addr;
    use std::net::SocketAddrV4;

    let mut ftp_conn = FtpConnection::connect(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 21))
        .expect("Ftp connection failed");

    ftp_conn
        .login(
            "anonymous".to_string(),
            Some("fake@email.service".to_string()),
        )
        .expect("Login failed");
    let files = ftp_conn.list().expect("File listing failed.");
    println!("{}", files);

    ftp_conn
        .change_working_directory("/rfc")
        .expect("CWD failed");

    println!("{}", ftp_conn.pwd()?);

    ftp_conn.cd_up().expect("Failed to cd up");

    println!("{}", ftp_conn.pwd()?);

    ftp_conn.mkdir("Cool Directories Only".to_string())?;

    //let files = ftp_conn.list().expect("File listing failed.");
    //println!("{}", files);
    Ok(())
}
