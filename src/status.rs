pub mod ftp_status {
    /// Command okay.
    pub const COMMAND_OKAY: u32 = 200;
    /// Syntax error, command unrecognized.
    ///
    /// This may include errors such as command line to long.
    pub const SYNTAX_ERROR: u32 = 500;
    /// Syntax error in parameters or arguments.
    pub const SYNTAX_ERROR_ARGUMENTS: u32 = 501;
    /// Command not implemented, superfluous at this site.
    pub const COMMAND_NOT_IMPLEMENTED_UNNECESARY: u32 = 202;
    /// Command not implemented.
    pub const COMMAND_NOT_IMPLEMENTED: u32 = 502;
    /// Bad sequence of commands.
    pub const BAD_COMMAND_SEQUENCE: u32 = 503;
    /// Command not implemented for that parameter.
    pub const COMMAND_NOT_IMPLEMENTED_PARAMETER: u32 = 504;
    /// Restart marker reply.
    /// In this case, the text is exact and not left to the
    /// particular implementation; it must read:
    ///     MARK yyyy = mmmm
    /// Wthere yyyy is User-process data stream market, and mmmm
    /// server's equivelent market (note the space between markers and "=")
    pub const RESTART_MARKER_REPLY: u32 = 110;
    /// System status, or system help reply.
    pub const SYSTEM_STATUS: u32 = 211;
    /// Directory status.
    pub const DIRECTORY_STATUS: u32 = 212;
    /// File status.
    pub const FILE_STATUS: u32 = 213;
    /// Help message.
    /// On how to use the server or the meaning of a particular
    /// non-standard command. This reply is useful only to the
    /// human user.
    pub const HELP_MESSAGE: u32 = 214;
    /// NAME system type.
    /// Where NAME is an official system name from the list in the
    /// Assigned Numbers document.
    pub const SYSTEM_TYPE: u32 = 215;
    /// Service ready in nnn minutes.
    pub const READY_IN: u32 = 120;
    /// Servuce ready for new user.
    pub const SERVICE_READY: u32 = 220;
    /// Service closing control connection.
    /// Logged out if approperiate.
    pub const SERVER_CLOSING_CONTROL: u32 = 221;
    /// Service not available, closing control connection.
    /// This may be a reply to any command if the service knows it
    /// must shut down.
    pub const SERVICE_NOT_AVAILABLE: u32 = 421;
    /// Data connection already open; transfer starting.
    pub const DATA_TRANSFER_STARTING: u32 = 125;
    /// Data connection open; no transfer in progress.
    pub const DATA_NOT_TRANSFERING: u32 = 225;
    /// Can't open data connection.
    pub const DATA_CANNOT_CONNECT: u32 = 425;
    /// Closing data connection.
    /// Requested file action successful (for example, file
    /// transfer or file abort).
    pub const DATA_CLOSING: u32 = 226;
    /// Connection closed; transfer aborted.
    pub const DATA_CLOSED_ABORTING: u32 = 426;
    /// Entering Passive Mode (h1,h2,h3,h4,p1,p2).
    pub const ENTERING_PASSIVE: u32 = 227;
    /// User logged in, proceed.
    pub const LOGGED_IN: u32 = 230;
    /// Not logged in.
    pub const NOT_LOGGED_IN: u32 = 530;
    /// User name okay, need password.
    pub const PASSWORD_NEEDED: u32 = 331;
    /// Need account for login.
    pub const ACCOUNT_REQUIRED_LOGIN: u32 = 332;
    /// Need account for storing files.
    pub const ACCOUNT_REQUIRED_STORING: u32 = 532;
    /// File status okay; about to open the data connection.
    pub const FILE_OPENING_DATA: u32 = 150;
    /// Requested file action okay, completed.
    pub const FILE_ACTION_COMPLETE: u32 = 250;
    /// "PATHNAME" created.
    pub const DIRECTORY_CREATED: u32 = 257;
    /// Requested file action pending further information.
    pub const FILE_NEED_INFORMATION: u32 = 350;
    /// File action not taken.
    /// File unavailable (e.g., file busy)
    pub const FILE_ACTION_NOT_TAKEN: u32 = 450;
    /// Requested action not taken.
    /// File unavailable (e.g., file not found, no access).
    pub const ACTION_NOT_TAKEN: u32 = 550;
    /// Requested action aborted. Local error in processing.
    pub const ACTION_ABORTED_PROCESSING: u32 = 451;
    /// Requested action aborted. Page type unknown.
    pub const ACTION_ABORTED_UNKOWN_PAGE: u32 = 551;
    /// Requested action not taken.
    /// Insufficient storage space in system.
    pub const INSUFFICIENT_STORAGE: u32 = 452;
    /// Requested file action aborted.
    /// Exceeded storage allocation (for current directory or
    /// dataset).
    pub const INSUFFICIENT_ALLOCATED_STORAGE: u32 = 552;
    /// Requsted action not taken.
    /// File name not allowed.
    pub const FILE_NAME_INVALID: u32 = 553;
}
