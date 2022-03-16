/*
 * Rune - OS Image preperation tool
 *
 * Copyright, 2017-2020 Haiku, Inc. All rights Reserved.
 * Released under the terms of the MIT license.
 *
 * Authors:
 *   Alexander von Gluck IV <kallisti5@unixzen.com>
 */

extern crate serde_json;
extern crate curl;

use std::error::Error;
use curl::easy::Easy;

pub const MANIFEST_URI: &str = "https://github.com/haiku/firmware/raw/master/u-boot/manifest.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Board {
	pub arch: String,
	pub id:   String,
	pub soc:  String,
	pub name: String,
	pub files: Vec<String>,
}

pub fn get_boards(uri: String) -> Result<Vec<Board>, Box<dyn Error>> {
	// Download file per manifest
	let mut buffer = Vec::new();
	let mut curl = Easy::new();
	curl.url(uri.as_str())?;
	curl.follow_location(true)?;
	{
		let mut transfer = curl.transfer();
		transfer.write_function(|new_data| {
			buffer.extend_from_slice(new_data);
			Ok(new_data.len())
		})?;
		transfer.perform()?;
	}
	let content = String::from_utf8_lossy(&buffer);
	let results = serde_json::from_str(&content)?;
	return Ok(results);
}

pub fn get_arch(arch: String) -> Result<Vec<Board>, Box<dyn Error>> {
	let uri = MANIFEST_URI.to_string();
	let boards = get_boards(uri)?;
	let mut results: Vec<Board> = Vec::new();
	for i in boards {
		if i.arch == arch {
			results.push(i);
		}
	}
	return Ok(results)
}

pub fn get_board(board_id: String) -> Result<Board, Box<dyn Error>> {
	let uri = "https://github.com/haiku/firmware/raw/master/u-boot/manifest.json".to_string();
	let boards = get_boards(uri)?;
	for i in boards {
		if i.id == board_id {
			return Ok(i);
		}
	}
	return Err(From::from("Unknown target board!"));
}

pub fn print(arch: String) {
	print!("{}\n===\n", arch);
	let arch_boards = match get_arch(arch) {
		Ok(m) => { m },
		Err(e) => { println!("  Error: {}", e); return },
	};
	print!("  {:20} {:10} {:20}\n", "Board", "SOC", "Name");
	for board in arch_boards {
		print!("  {:20} {:10} {:20}\n", board.id, board.soc, board.name);
	}
}
