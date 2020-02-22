extern crate regex;
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
        // `﻿-rw-r--r--   1 0        0        41 Feb 22 16:06 README.txt`

        println!("{:?}", &s);

        let item_type = match &s[0..1] {
            "d" => DirectoryItemType::Directory,
            "-" => DirectoryItemType::File,
            "l" => DirectoryItemType::Link,
            _ => return Err("Unknown type".to_string()),
        };

        let split_space: Vec<&str> = s.split(' ').collect();

        let space_seperated = split_space.join(" ");

        // FIXME: This is some hackish madness
        let re = Regex::new(
            r"[A-z]{3}[ ]{1,}[0-9]{1,2}[ ]{1,}([0-9]{4}|[0-9]{2}:[0-9]{2})[ ]{1,}(.{1,})",
        )
        .unwrap();

        let name_captures = match re.captures(&space_seperated) {
            Some(v) => v,
            None => return Err("Failure parsing".to_string()),
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
