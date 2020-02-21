use std::net::Ipv4Addr;
use std::net::SocketAddrV4;

let mut ftp_conn =  FtpConnection::connect(SocketAddrV4::new(Ipv4Addr::new(4, 31, 198, 44), 21));
ftp_conn.login("anonymous", Some("fake@email.service")).unwrap();
