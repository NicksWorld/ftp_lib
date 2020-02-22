//! # ftp_lib
//! ftp_lib is a FTP implementation for a school project
//!
//! Examples:
//! ```rust
//! use ftp_lib::FtpConnection;
//! use std::net::SocketAddrV4;
//!
//! let mut ftp_conn = FtpConnection::connect(
//!     "127.0.0.1:21".parse().unwrap()
//! ).unwrap();
//!
//! ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
//!
//! // Create a directory
//! ftp_conn.mkdir("test_dir").unwrap();
//!
//! // Move into the newly created directory and print the working directory
//! ftp_conn.cd("test_dir").unwrap();
//! println!("PWD: {:?}", ftp_conn.pwd().unwrap());
//!
//! // Return to / from /test_dir and print the working directory
//! ftp_conn.cdup().unwrap();
//! println!("PWD: {:?}", ftp_conn.pwd().unwrap());
//!
//! // Remove the directory created
//! ftp_conn.rmdir("test_dir").unwrap();
//!
//! ftp_conn.quit().unwrap();
//! ```

use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::net::TcpStream;

use std::str::FromStr;

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

/// Module containing all errors returned by ftp_lib.
pub mod error;
use error::FtpError;
use error::FtpError::*;

/// Module containing all the status codes required to handle responses.
pub mod status;
use status::ftp_status;

/// Module for handling the outputs of ftp_conn.list()
pub mod filestructure;

/// Data structure that contains a response from the FTP server.
#[derive(Debug, Clone)]
pub struct FtpResponse {
    /// The status code recieved (see status.rs for more information)
    pub status: u32,
    /// The content of the message recieved (FTP often has very human readable responses)
    pub content: String,
}

impl FtpResponse {
    /// Parses the ip and port from the PASV command's result.
    ///
    /// Examples:
    /// ```rust
    /// use ftp_lib::FtpResponse;
    ///
    /// let response = FtpResponse {
    ///     status: 227,
    ///     content: "227 Entering passive (127,0,0,1,250,29)".to_string()
    /// };
    /// println!("{:?}", response.parse_pasv_addr().unwrap());
    /// ```
    pub fn parse_pasv_addr(&self) -> Result<SocketAddrV4, FtpError> {
        // Make sure the type being converted really is a PASV response
        if self.status != ftp_status::ENTERING_PASSIVE {
            return Err(InvalidTypeError);
        }

        // Fetch the area within the parentheses of the PASV response
        let pasv_raw = &self.content.as_str();
        let pasv_addr_section =
            &pasv_raw[pasv_raw.find('(').unwrap_or(0) + 1..pasv_raw.find(')').unwrap_or(0)];

        // Make sure the right number of parameters are supplied (4 for the IP, 2 for the port)
        let pasv_unparsed: Vec<&str> = pasv_addr_section.split(',').collect();
        if pasv_unparsed.len() != 6 {
            return Err(InvalidResponseError(self.clone()));
        }

        // Parse the octets of the IP address into numbers
        let mut octets = vec![];
        for number in pasv_unparsed[0..4].iter() {
            match number.parse::<u8>() {
                Ok(v) => octets.push(v),
                Err(_) => return Err(InvalidResponseError(self.clone())),
            }
        }

        // Parse the data on the data port
        let mut port_data = vec![];
        for number in pasv_unparsed[4..].iter() {
            match number.parse::<u16>() {
                Ok(v) => port_data.push(v),
                Err(_) => return Err(InvalidResponseError(self.clone())),
            }
        }

        let datastream_addr = SocketAddrV4::new(
            Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]),
            (port_data[0] * 256) + port_data[1], // Finds the port number
        );
        Ok(datastream_addr)
    }
}

impl FromStr for FtpResponse {
    type Err = FtpError;

    fn from_str(s: &str) -> Result<FtpResponse, FtpError> {
        if s.len() >= 3 {
            let status_code = &s[0..3];

            // Make sure the reponse is in the format `000 Desctiption`
            if !s.starts_with(&format!("{} ", status_code)) {
                return Err(InvalidResponseFormatError);
            }

            // As long as the status code is a number, return the response
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

/// The state of the ftp connection, connected or not
#[derive(Debug, Clone, Copy)]
pub enum FtpState {
    /// If the connection is active
    Connected,
    /// If the connection is inactive
    Disconnected,
}

/// The main type used for communication with the FTP server.
///
/// Examples:
/// Connection
/// ```rust
/// use ftp_lib::FtpConnection;
/// use std::net::SocketAddrV4;
///
/// let mut ftp_conn = FtpConnection::connect(
///     "127.0.0.1:21".parse().unwrap()
/// ).unwrap(); // Initiate the connection
///
/// ftp_conn.quit().unwrap();
/// ```
#[derive(Debug)]
pub struct FtpConnection {
    reader: BufReader<TcpStream>,
    state: FtpState,
}

impl FtpConnection {
    /// Initiates the connection to the FTP server.
    ///
    /// Example:
    /// Connect to 127.0.0.1 then close the connection.
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap(); // Initiate the connection
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn connect(connection_addr: SocketAddrV4) -> Result<FtpConnection, FtpError> {
        // Initiate connection to the FTP server
        match TcpStream::connect(connection_addr) {
            Ok(stream) => {
                //  Initiate a new instance for user use.
                let mut ftp_conn = FtpConnection {
                    reader: BufReader::new(stream),
                    state: FtpState::Connected,
                };

                let res = ftp_conn.wait_for_response()?;

                // TODO: Missing other possible responses
                match res.status {
                    // Successful connection
                    ftp_status::SERVICE_READY => Ok(ftp_conn),
                    // Error connecting
                    ftp_status::READY_IN => {
                        let res = ftp_conn.wait_for_response()?;

                        match res.status {
                            ftp_status::SERVICE_READY => Ok(ftp_conn),
                            _ => Err(ServiceNotReady),
                        }
                    }
                    ftp_status::SERVICE_NOT_AVAILABLE => Err(ServiceUnavailable),
                    _ => Err(InvalidResponseError(res)),
                }
            }
            Err(_) => Err(ConnectionError),
        }
    }

    /// Initiates the connection to the FTP server.
    ///
    /// Example:
    /// Connect to 127.0.0.1 then close the connection.
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// // login(username, password)
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap(); // Login to the server
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn login(&mut self, username: &str, password: Option<&str>) -> Result<(), FtpError> {
        let command = format!("USER {}\r\n", username);
        self.write_command(command.clone())?;

        let user_result = self.wait_for_response()?;

        match user_result.status {
            // Successful action
            ftp_status::PASSWORD_NEEDED => {
                let command = format!("PASS {}\r\n", password.unwrap_or_else(|| ""));
                self.write_command(command.clone())?;
                let pass_result = self.wait_for_response()?;

                match pass_result.status {
                    ftp_status::LOGGED_IN => Ok(()),

                    _ => Err(FtpError::from_status_code(pass_result, command)),
                }
            }
            ftp_status::LOGGED_IN => Ok(()),
            // Error completing action
            _ => Err(FtpError::from_status_code(user_result, command)),
        }
    }

    /// Terminates the connection to the FTP server.
    ///
    /// Example:
    /// Connect to localhost then close the connection.
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// // Preform actions with the FTP server
    ///
    /// ftp_conn.quit().unwrap(); // End the connection to the server.
    /// ```
    pub fn quit(&mut self) -> Result<(), FtpError> {
        let command = "QUIT\r\n".to_string();
        self.write_command(command)?;

        // Shut the connection down even if the server does not respond nicely
        match self.reader.get_mut().shutdown(std::net::Shutdown::Both) {
            Ok(_) => Ok(()),
            Err(_) => Err(ConnectionError),
        }
    }

    /// Changes the current working directory in the FTP server.
    ///
    /// Example:
    /// Connect to 127.0.0.1 then change to the /test_dir directory.
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// # // Create the directory
    /// # ftp_conn.mkdir("test_dir").unwrap();
    /// ftp_conn.cd("test_dir").unwrap(); // Change the working directory to rfc
    /// # // Remove dir
    /// # ftp_conn.cdup().unwrap();
    /// # ftp_conn.rmdir("test_dir").unwrap();
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn cd(&mut self, path: &str) -> Result<(), FtpError> {
        let command = format!("CWD {}\r\n", path);
        self.write_command(command.clone())?;

        let cwd_result = self.wait_for_response()?;

        match cwd_result.status {
            // Successful action
            ftp_status::FILE_ACTION_COMPLETE => Ok(()),
            // Error completing action
            _ => Err(FtpError::from_status_code(cwd_result, command)),
        }
    }

    /// Changes the current working directory in the FTP server to the parent directory.
    ///
    /// Example:
    /// Connect to localhost then go into /test_dir and return to /.
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// # // Create the directory
    /// # ftp_conn.mkdir("test_dir").unwrap();
    /// ftp_conn.cd("test_dir").unwrap();
    /// println!("PWD: {:?}", ftp_conn.pwd());
    ///
    /// ftp_conn.cdup().unwrap(); // Return to / from /test_dir
    /// println!("PWD: {:?}", ftp_conn.pwd());
    /// # // Remove dir
    /// # ftp_conn.rmdir("test_dir").unwrap();
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn cdup(&mut self) -> Result<(), FtpError> {
        let command = "CDUP\r\n".to_string();
        self.write_command(command.clone())?;

        let cdup_result = self.wait_for_response()?;

        match cdup_result.status {
            // Successful action
            ftp_status::FILE_ACTION_COMPLETE => Ok(()),
            // Error completing action
            _ => Err(FtpError::from_status_code(cdup_result, command)),
        }
    }

    /// Gets the current working directory on the FTP server.
    ///
    /// Example:
    /// Connect to localhost then fetch the current working directory
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// println!("PWD: {:?}", ftp_conn.pwd()); // Fetch the current working directory
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
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
            _ => Err(FtpError::from_status_code(pwd_result, command)),
        }
    }

    /// Lists files in the current directory
    ///
    /// Example:
    /// Lists all files in /
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// # // Add file to show
    /// # ftp_conn.write_file("cool.txt", "".as_bytes().to_vec()).unwrap();
    /// println!("{:?}", ftp_conn.list().unwrap()); // Print files in current directory
    /// # // Clean up file
    /// # ftp_conn.rm("cool.txt").unwrap();
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn list(&mut self) -> Result<Vec<filestructure::DirectoryItem>, FtpError> {
        let datastream_addr = self.pasv()?;

        let command = "LIST\r\n".to_string();
        self.write_command(command.clone())?;

        let datavec = self.connect_datastream(datastream_addr)?;

        let res = self.wait_for_response()?;
        match res.status {
            // Successful action
            ftp_status::FILE_OPENING_DATA | ftp_status::DATA_TRANSFER_STARTING => (),
            // Error completing action
            _ => return Err(FtpError::from_status_code(res, command)),
        }

        let res = self.wait_for_response()?;
        match res.status {
            // Successful action
            ftp_status::DATA_CLOSING | ftp_status::FILE_ACTION_COMPLETE => (),
            // Error completing action
            _ => return Err(FtpError::from_status_code(res, command)),
        }

        // FIXME: Rewrite this?
        let files: Vec<String> = String::from_utf8_lossy(&datavec)
            .split("\r\n")
            .filter(|&x| x != "")
            .map(|x| x.to_string())
            .collect();
        let mut file_output = vec![];
        for file in files {
            match filestructure::DirectoryItem::from_str(&file) {
                Ok(v) => file_output.push(v),
                Err(_) => return Err(InvalidResponseError(res)),
            }
        }
        Ok(file_output)
    }

    /// Lists files in the current directory (Names only)
    ///
    /// Example:
    /// Lists all files in /
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// # // Add file to show
    /// # ftp_conn.write_file("cool.txt", "".as_bytes().to_vec()).unwrap();
    /// println!("{:?}", ftp_conn.name_list().unwrap()); // Print files in current directory
    /// # // Clean up file
    /// # ftp_conn.rm("cool.txt").unwrap();
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn name_list(&mut self) -> Result<Vec<String>, FtpError> {
        let datastream_addr = self.pasv()?;

        let command = "NLST\r\n".to_string();
        self.write_command(command.clone())?;

        let datavec = self.connect_datastream(datastream_addr)?;

        let res = self.wait_for_response()?;
        match res.status {
            // Successful action
            ftp_status::FILE_OPENING_DATA | ftp_status::DATA_TRANSFER_STARTING => (),
            // Error completing action
            _ => return Err(FtpError::from_status_code(res, command)),
        }

        let res = self.wait_for_response()?;
        match res.status {
            // Successful action
            ftp_status::DATA_CLOSING | ftp_status::FILE_ACTION_COMPLETE => (),
            // Error completing action
            _ => return Err(FtpError::from_status_code(res, command)),
        }

        let output_vec = String::from_utf8_lossy(&datavec)
            .trim_end()
            .split("\r\n")
            .map(String::from)
            .collect();

        Ok(output_vec)
    }

    /// Creates a new directory on the FTP server.
    ///
    /// Example:
    /// Connect to localhost then create the directory cool_directory
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// ftp_conn.mkdir("cool_directory").unwrap(); // Create the directory cool_directory
    /// println!("{:?}", ftp_conn.list().unwrap()); // Show the files in the current directory
    /// # // Cleanup
    /// # ftp_conn.rmdir("cool_directory").unwrap();
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn mkdir(&mut self, dir_name: &str) -> Result<(), FtpError> {
        let command = format!("MKD {}\r\n", dir_name);
        self.write_command(command.clone())?;

        let mkd_result = self.wait_for_response()?;

        match mkd_result.status {
            // Successful action
            ftp_status::DIRECTORY_CREATED => Ok(()),
            // Error completing action
            _ => Err(FtpError::from_status_code(mkd_result, command)),
        }
    }

    /// Removees a directory on the FTP server.
    ///
    /// Example:
    /// Connect to localhost then close the connection.
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// # // Create directory to remove
    /// # ftp_conn.mkdir("test_dir").unwrap();
    /// println!("{:?}", ftp_conn.list().unwrap()); // Show before
    /// ftp_conn.rmdir("test_dir").unwrap(); // Remove the directory
    /// println!("{:?}", ftp_conn.list().unwrap()); // Show after
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn rmdir(&mut self, directory: &str) -> Result<(), FtpError> {
        let command = format!("RMD {}\r\n", directory);
        self.write_command(command.clone())?;

        let rmd_result = self.wait_for_response()?;
        match rmd_result.status {
            // Successful action
            ftp_status::FILE_ACTION_COMPLETE => Ok(()),
            // Error completing action
            _ => Err(FtpError::from_status_code(rmd_result, command)),
        }
    }

    /// Fetches the contents of the specified file
    ///
    /// Example:
    /// Connect to localhost then read README.txt
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// # ftp_conn.write_file("README.txt", "This is the README.txt file. Cool, right?".as_bytes().to_vec()).unwrap();
    /// println!("{}",
    ///     String::from_utf8_lossy(&ftp_conn.fetch_file("README.txt").unwrap()) // Fetch the contents of the file
    /// );
    /// # ftp_conn.rm("README.txt").unwrap();
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn fetch_file(&mut self, file_name: &str) -> Result<Vec<u8>, FtpError> {
        let datastream_addr = self.pasv()?;

        let command = format!("RETR {}\r\n", file_name);
        self.write_command(command.clone())?;

        let data = self.connect_datastream(datastream_addr)?;

        // NOTE: The following statuses do not cover all cases.
        // I am not sure where status can be returned for now.
        let res = self.wait_for_response()?;
        match res.status {
            // Successful action
            ftp_status::FILE_OPENING_DATA | ftp_status::DATA_TRANSFER_STARTING => (),
            // Error completing action
            _ => return Err(FtpError::from_status_code(res, command)),
        }

        let res = self.wait_for_response()?;
        match res.status {
            // Successful action
            ftp_status::DATA_CLOSING | ftp_status::FILE_ACTION_COMPLETE => (),
            // Error completing action
            _ => return Err(FtpError::from_status_code(res, command)),
        }

        Ok(data)
    }

    /// Removees a file on the FTP server.
    ///
    /// Example:
    /// Writes test.txt
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// ftp_conn.write_file("test.txt", "Cool Data here".as_bytes().to_vec()).unwrap();
    /// println!("{}",
    ///     String::from_utf8_lossy(&ftp_conn.fetch_file("test.txt").unwrap())
    /// );
    /// # // Remove file
    /// # ftp_conn.rm("test.txt").unwrap();
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn write_file(&mut self, file_name: &str, data: Vec<u8>) -> Result<(), FtpError> {
        let datastream_addr = self.pasv()?;

        let command = format!("STOR {}\r\n", file_name);
        self.write_command(command.clone())?;

        if let Ok(mut datastream) = TcpStream::connect(datastream_addr) {
            let data_write_res = datastream.write_all(&data);
            match data_write_res {
                Ok(_) => (),
                Err(_) => return Err(ConnectionError),
            }

            // I have absolutely no idea what the following could return
            match datastream.shutdown(std::net::Shutdown::Both) {
                Ok(_) => (),
                Err(_) => return Err(DatastreamConnectionError),
            }

            let res = self.wait_for_response()?;
            match res.status {
                // Sucessful action
                ftp_status::FILE_OPENING_DATA | ftp_status::DATA_TRANSFER_STARTING => (),
                // Error completing action
                _ => return Err(FtpError::from_status_code(res, command)),
            }

            let res = self.wait_for_response()?;
            match res.status {
                // Successful action
                ftp_status::DATA_CLOSING | ftp_status::FILE_ACTION_COMPLETE => (),
                // Error completing action
                _ => return Err(FtpError::from_status_code(res, command)),
            }
            Ok(())
        } else {
            Err(ConnectionError)
        }
    }

    /// Removes a file on the FTP server.
    ///
    /// Example:
    /// Removes test.txt
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// # // Create file to remove
    /// # ftp_conn.write_file("test.txt", "".as_bytes().to_vec()).unwrap();
    /// println!("{:?}", ftp_conn.list().unwrap()); // Show before
    /// ftp_conn.rm("test.txt").unwrap(); // Remove the directory
    /// println!("{:?}", ftp_conn.list().unwrap()); // Show after
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn rm(&mut self, file: &str) -> Result<(), FtpError> {
        let command = format!("DELE {}\r\n", file);
        self.write_command(command.clone())?;

        let dele_result = self.wait_for_response()?;
        match dele_result.status {
            // Successful action
            ftp_status::COMMAND_OKAY | ftp_status::FILE_ACTION_COMPLETE => Ok(()),
            // Error completing action
            _ => Err(FtpError::from_status_code(dele_result, command)),
        }
    }

    /// Renames the specified file on the FTP server
    ///
    /// Example:
    /// Connect to localhost rename a new file
    /// ```rust
    /// use ftp_lib::FtpConnection;
    /// use std::net::SocketAddrV4;
    ///
    /// let mut ftp_conn = FtpConnection::connect(
    ///     "127.0.0.1:21".parse().unwrap()
    /// ).unwrap();
    ///
    /// ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
    ///
    /// # // Create file
    /// # ftp_conn.write_file("test.txt", "".as_bytes().to_vec()).unwrap();
    /// println!("{:?}", ftp_conn.list().unwrap()); // Display begining file structure
    /// ftp_conn.rename("test.txt", "cool.txt"); // Rename from test.txt to cool.txt
    /// println!("{:?}", ftp_conn.list().unwrap()); // Verify the change
    /// # // Cleanup
    /// # ftp_conn.rm("cool.txt").unwrap();
    ///
    /// ftp_conn.quit().unwrap();
    /// ```
    pub fn rename(&mut self, src: &str, dst: &str) -> Result<(), FtpError> {
        let command = format!("RNFR {}\r\n", src);
        self.write_command(command.clone())?;

        let rnfr_result = self.wait_for_response()?;
        match rnfr_result.status {
            // Successful action
            ftp_status::FILE_ACTION_COMPLETE | ftp_status::FILE_NEED_INFORMATION => {
                let command = format!("RNTO {}\r\n", dst);
                self.write_command(command.clone())?;

                let rnto_result = self.wait_for_response()?;

                match rnto_result.status {
                    // Successful action
                    ftp_status::FILE_ACTION_COMPLETE => Ok(()),
                    // Error completing action
                    _ => Err(FtpError::from_status_code(rnto_result, command)),
                }
            }
            // Error completing action
            _ => Err(FtpError::from_status_code(rnfr_result, command)),
        }
    }

    fn connect_datastream(&self, datastream_addr: SocketAddrV4) -> Result<Vec<u8>, FtpError> {
        // Connect to the datastream on the specified port
        match TcpStream::connect(datastream_addr) {
            Ok(mut datastream) => {
                // Read from then kill the connection to the datastream
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
        // PASV opens a port on the host where the datastream is waiting
        let command = "PASV\r\n".to_string();
        self.write_command(command.clone())?;

        let pasv_result = self.wait_for_response()?;

        match pasv_result.status {
            // Successful action
            ftp_status::ENTERING_PASSIVE => pasv_result.parse_pasv_addr(),
            // Error completing action
            _ => Err(FtpError::from_status_code(pasv_result, command)),
        }
    }

    fn write_command(&mut self, command: String) -> Result<(), FtpError> {
        // Send the command in bytes to the FTP server
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
                    Ok(v) => Ok(v), // The response was single line
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
