pub mod FtpStatus {
	/// Command okay.
	pub const CommandOkay: u32 = 200;
	/// Syntax error, command unrecognized.
	///
	/// This may include errors such as command line to long.
	pub const SyntaxError: u32 = 500;
	/// Syntax error in parameters or arguments.
	pub const SyntaxErrorArguments: u32 = 501;
	/// Command not implemented, superfluous at this site.
	pub const CommandNotImplementedUnnecesary: u32 = 202;
	/// Command not implemented.
	pub const CommandNotImplemented: u32 = 502;
	/// Bad sequence of commands.
	pub const BadCommandSequence: u32 = 503;
	/// Command not implemented for that parameter.
	pub const CommandNotImplementedParameter: u32 = 504;
	/// Restart marker reply.
	/// In this case, the text is exact and not left to the
	/// particular implementation; it must read:
	///     MARK yyyy = mmmm
	/// Wthere yyyy is User-process data stream market, and mmmm
	/// server's equivelent market (note the space between markers and "=")
	pub const RestartMarkerReply: u32 = 110;
	/// System status, or system help reply.
	pub const SystemStatus: u32 = 211;
	/// Directory status.
	pub const DirectoryStatus: u32 = 212;
	/// File status.
	pub const FileStatus: u32 = 213;
	/// Help message.
	/// On how to use the server or the meaning of a particular
	/// non-standard command. This reply is useful only to the
	/// human user.
	pub const HelpMessage: u32 = 214;
	/// NAME system type.
	/// Where NAME is an official system name from the list in the
	/// Assigned Numbers document.
	pub const SystemType: u32 = 215;
	/// Service ready in nnn minutes.
	pub const ReadyIn: u32 = 120;
	/// Servuce ready for new user.
	pub const ServiceReady: u32 = 220;
	/// Service closing control connection.
	/// Logged out if approperiate.
	pub const ServerClosingControl: u32 = 221;
	/// Service not available, closing control connection.
	/// This may be a reply to any command if the service knows it
	/// must shut down.
	pub const ServiceNotAvailable: u32 = 421;
	/// Data connection already open; transfer starting.
	pub const DataTransferStarting: u32 = 125;
	/// Data connection open; no transfer in progress.
	pub const DataNotTransfering: u32 = 225;
	/// Can't open data connection.
	pub const DataCannotConnect: u32 = 425;
	/// Closing data connection.
	/// Requested file action successful (for example, file
	/// transfer or file abort).
	pub const DataClosing: u32 = 226;
	/// Connection closed; transfer aborted.
	pub const DataClosedAborting: u32 = 426;
	/// Entering Passive Mode (h1,h2,h3,h4,p1,p2).
	pub const EnteringPassive: u32 = 227;
	/// User logged in, proceed.
	pub const LoggedIn: u32 = 230;
	/// Not logged in.
	pub const NotLoggedIn: u32 = 530;
	/// User name okay, need password.
	pub const PasswordNeeded: u32 = 331;
	/// Need account for login.
	pub const AccountRequiredLogin: u32 = 332;
	/// Need account for storing files.
	pub const AccountRequiredStoring: u32 = 532;
	/// File status okay; about to open the data connection.
	pub const FileOpeningData: u32 = 150;
	/// Requested file action okay, completed.
	pub const FileActionComplete: u32 = 250;
	/// "PATHNAME" created.
	pub const DirectoryCreated: u32 = 257;
	/// Requested file action pending further information.
	pub const FileNeedInformation: u32 = 350;
	/// File action not taken.
	/// File unavailable (e.g., file busy)
	pub const FileActionNotTaken: u32 = 450;
	/// Requested action not taken.
	/// File unavailable (e.g., file not found, no access).
	pub const ActionNotTaken: u32 = 550;
	/// Requested action aborted. Local error in processing.
	pub const ActionAbortedProccessing: u32 = 451;
	/// Requested action aborted. Page type unknown.
	pub const ActionAbortedUnknownPage: u32 = 551;
	/// Requested action not taken.
	/// Insufficient storage space in system.
	pub const InsufficientStorage: u32 = 452
		;/// Requested file action aborted.
	/// Exceeded storage allocation (for current directory or
	/// dataset).
	pub const InsufficientAllocatedStorage: u32 = 552;
	/// Requsted action not taken.
	/// File name not allowed.
	pub const FileNameInvalid: u32 = 553;
}
