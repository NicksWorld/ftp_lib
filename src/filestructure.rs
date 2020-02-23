use regex::Regex;

use std::str::FromStr;

#[derive(Debug)]
pub enum DirectoryItemType {
	Link,
	File,
	Directory,
}

#[derive(Debug)]
pub struct DirectoryItem {
	name: String,
	item_type: DirectoryItemType,
}

impl FromStr for DirectoryItem {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		// Process the first character for the file type
		let item_type = match &s[0..1] {
			"d" => DirectoryItemType::Directory,
			"-" => DirectoryItemType::File,
			"l" => DirectoryItemType::Link,
			_ => return Err("Unknown type".to_string()),
		};

		lazy_static! {
			static ref RE: Regex = Regex::new(
				r"[A-z]{3}[ ]{1,}[0-9]{1,2}[ ]{1,}([0-9]{4}|[0-9]{2}:[0-9]{2})[ ]{1,}(.{1,})",
			).unwrap();
		}

		let name_captures = match RE.captures(&s) {
			Some(v) => v,
			None => return Err("No name found".to_string()),
		};

		if name_captures.len() < 3 {
			return Err("No name found".to_string());
		}

		Ok(DirectoryItem {
			name: name_captures[2].to_string(),
			item_type,
		})
	}
}
