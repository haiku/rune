/*
 * Rune - OS Image preperation tool
 *
 * Copyright, 2017-2019 Haiku, Inc. All rights Reserved.
 * Released under the terms of the MIT license.
 *
 * Authors:
 *   Alexander von Gluck IV <kallisti5@unixzen.com>
 */

extern crate curl;
extern crate getopts;
extern crate mbr;
extern crate fatfs;
extern crate regex;
extern crate url;
extern crate itertools;
extern crate indicatif;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use std::process;
use std::env;
use std::path::PathBuf;
use std::io;
use std::io::{SeekFrom,Seek};
use std::fs;
use std::fs::OpenOptions;
use curl::easy::Easy;
use fatfs::{BufStream, FileSystem, FsOptions};
use getopts::Options;
use url::Url;
use indicatif::{ProgressBar,ProgressStyle};
use crate::partition::Partition;
use regex::Regex;

mod boards;
mod partition;
mod image_tools;

fn print_version() {
	const VERSION: &str = env!("CARGO_PKG_VERSION");
	println!("Rune v{}", VERSION);
	println!("Copyright, 2017-2024 Haiku, Inc. All rights reserved.");
	println!("Released under the terms of the MIT license.");
	println!("Manifest URI: {}", boards::MANIFEST_URI);
}

fn print_usage(program: &str, opts: Options) {
	let brief = format!("rune - write bootable ARM Haiku mmc images\nUsage: {} [options] <output>", program);
	print!("{}", opts.usage(&brief));
}

fn flag_error(program: &str, opts: Options, error: &str) {
	print!("Error: {}\n\n", error);
	print_usage(&program, opts);
}

/// Write files which have an offset to the target block device / image
fn write_files(board: boards::Board, disk: PathBuf, steps: u32)
	-> Result<u32, Box<dyn Error>> {
	let count = board.files.len() as u32;
	if count == 0 {
		return Err(From::from("No files found for board!"));
	}
	let mut wrote = count;
	let bar = ProgressBar::new((count * 2) as u64);
	bar.set_style(ProgressStyle::default_bar()
		.template("{prefix} {spinner:.bold}[{bar:40.cyan/blue}] {msg:.bold.dim}")
		.tick_chars("◐◓◑◒")
		.progress_chars("#>-"));
	bar.set_prefix(format!("[{}/{}] Provisioning block device...", steps - 1, steps));
	let raw_re = Regex::new(r"^(\d+),(.+)$").unwrap();

	let mut output_fh = OpenOptions::new().read(true).write(true).open(disk)?;
	for i in board.files {
		if !raw_re.is_match(i.as_str()) {
			// This is not raw file which goes directly on the image. Skip it.
			wrote = wrote - 1;
			bar.inc(2);
			continue;
		}
		let matches = match raw_re.captures(i.as_str()) {
			Some(x) => x,
			None => {
				return Err(From::from(format!("Error: Invalid raw file: '{:?}'", i)));
			}
		};
		let url = Url::parse(&matches[2])?;
		let offset = &matches[1].parse::<u64>().unwrap();

		let filename = match url.path_segments() {
			Some(x) => x.last().unwrap(),
			None => return Err(From::from(format!("Invalid URL {}", i))),
		};

		bar.set_message(format!("Downloading: {}", filename));
		bar.inc(1);

		//println!("  GET {} {:?} to write at {}", url, filename, offset);

		// Download file per manifest
		let mut buffer = Vec::new();
		let mut curl = Easy::new();
		curl.url(url.as_str())?;
		curl.follow_location(true)?;
		{
			let mut transfer = curl.transfer();
			transfer.write_function(|new_data| {
				buffer.extend_from_slice(new_data);
				Ok(new_data.len())
			})?;
			transfer.perform()?;
		}

		bar.set_message(format!("Writing: {}", filename));
		bar.inc(1);

		// Jump to specified offset in file
		output_fh.seek(SeekFrom::Start(*offset))?;
		// Write the raw file to the block device.
		io::copy(&mut &buffer[..], &mut output_fh)?;
	}
	output_fh.sync_data()?;

	if wrote == 0 {
		bar.set_message(format!("None required."));
	}
	bar.set_message(format!("Success ({} offsets).", wrote));

	bar.finish();
	return Ok(wrote);
}

/// Place the files onto the fat32 boot partition on the block device or image.
fn place_files(board: boards::Board, target_fs: &mut fatfs::FileSystem, steps: u32)
	-> Result<u32, Box<dyn Error>> {
	let count = board.files.len() as u32;
	if count == 0 {
		return Err(From::from("No files found for board!"));
	}
	let mut wrote = count;
	let bar = ProgressBar::new((count * 2) as u64);
	bar.set_style(ProgressStyle::default_bar()
		.template("{prefix} {spinner:.bold}[{bar:40.cyan/blue}] {msg:.bold.dim}")
		.tick_chars("◐◓◑◒")
		.progress_chars("#>-"));
	bar.set_prefix(format!("[{}/{}] Provisioning filesystem...  ", steps, steps));
	let raw_re = Regex::new(r"^(\d+),(.+)$").unwrap();
	for i in board.files {
		if raw_re.is_match(i.as_str()) {
			// This is a raw file which goes directly on the image. Skip it.
			wrote = wrote - 1;
			bar.inc(2);
			continue;
		}
		let url = Url::parse(i.as_str())?;
		let filename = match url.path_segments() {
			Some(x) => x.last().unwrap(),
			None => return Err(From::from(format!("Invalid URL {}", i))),
		};

		bar.set_message(format!("Downloading: {}", filename));
		bar.inc(1);

		//println!("  GET {} {:?}", url, filename);

		// Download file per manifest
		let mut buffer = Vec::new();
		let mut curl = Easy::new();
		curl.url(url.as_str())?;
		curl.follow_location(true)?;
		{
			let mut transfer = curl.transfer();
			transfer.write_function(|new_data| {
				buffer.extend_from_slice(new_data);
				Ok(new_data.len())
			})?;
			transfer.perform()?;
		}

		bar.set_message(format!("Writing: {}", filename));
		bar.inc(1);

		let mut target_file = target_fs.root_dir().create_file(filename)?;
		io::copy(&mut &buffer[..], &mut target_file)?;
	}
	if wrote == 0 {
		bar.set_message(format!("Not required."));
	}
	bar.set_message(format!("Success ({} files).", wrote));
	bar.finish();
	return Ok(wrote);
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	let mut opts = Options::new();
	opts.optopt("b", "board", "target board", "<board>");
	opts.optopt("i", "image", "source OS image", "<image>");
	opts.optflag("l", "list", "list supported target boards");
	opts.optflag("v", "version", "show version");
	opts.optflag("h", "help", "print this help");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m },
		Err(f) => {
			println!("Error: {}", f.to_string());
			process::exit(1);
		}
	};

	// Validate flags
	if matches.opt_present("h") {
		print_usage(&program, opts);
		return;
	} else if matches.opt_present("v") {
		print_version();
		return;
	} else if matches.opt_present("l") {
        //XXX This needs to be better and dynamic!
		boards::print("arm".to_string());
		process::exit(1);
	}

	let output_file = if !matches.free.is_empty() {
		PathBuf::from(&matches.free[0])
	} else {
		print_usage(&program, opts);
		process::exit(1);
	};

	let board_id = match matches.opt_str("b") {
		Some(x) => x,
		None => {
			flag_error(&program, opts, "Target board not provided!");
			process::exit(1);
		},
	};
	let board = match boards::get_board(board_id) {
		Ok(x) => x,
		Err(e) => {
			println!("Error: {}", e);
			process::exit(1);
		},
	};

	let mut steps = 3;
	if matches.opt_present("i") {
		steps = 4;
	}

	println!("[1/{}] Calculating dependencies for {} ({}) media...", steps, board.name, board.soc);

	// If an input image was provided, write it out.
	match matches.opt_str("i") {
		Some(x) => {
			// Go ahead and write out base image to destination
			println!("[2/{}] Writing Haiku to {:?}...", steps, output_file);
			let source_image = PathBuf::from(&x);
			match image_tools::write(source_image, output_file.clone()) {
				Ok(x) => x,
				Err(e) => {
					print!("Error: {}\n", e);
					process::exit(1);
				}
			};
		},
		None => { },
	}

	let boot_partition = match image_tools::locate_boot_partition(output_file.clone()) {
		Ok(x) => x,
		Err(e) => {
			print!("Error: {}\n", e);
			process::exit(1);
		},
	};

	match write_files(board.clone(), output_file.clone(), steps) {
		Ok(_) => {},
		Err(e) => {
			print!("Error Writing Files: {}\n", e);
			process::exit(1);
		}
	}

	let sector_size = 512;
	let handle = match OpenOptions::new().read(true).write(true).open(output_file.clone()) {
		Ok(x) => x,
		Err(e) => {
			print!("Error: {}\n", e);
			process::exit(1);
		},
	};

	let partition = match Partition::<fs::File>::new(handle, boot_partition.p_lba as u64 * sector_size, boot_partition.p_size as u64 * sector_size) {
		Ok(p) => p,
		Err(e) => {
			print!("Error Reading Partitions: {}\n", e);
			process::exit(1);
		},
	};

	let mut buf_rdr = BufStream::new(partition);
	let mut fs = match FileSystem::new(&mut buf_rdr, FsOptions::new()) {
		Ok(x) => x,
		Err(e) => {
			print!("Filesystem Error: {}\n", e);
			process::exit(1);
		}
	};

	match place_files(board.clone(), &mut fs, steps) {
		Ok(_) => {},
		Err(e) => {
			print!("Error Placing Files: {}\n", e);
			process::exit(1);
		}
	}

    /*
    // XXX: We might need this in the future, however we use the built-in
    // u-boot FDT in all test cases at the moment. This also corrupts the
    // current uEnv.txt. It needs to be smarter, and maybe weigh in the
    // firmware.json
	let boot_env
		= boards::get_boot_env(board.id);

	let mut env_file = match fs.root_dir().create_file("uEnv.txt") {
		Ok(o) => o,
		Err(e) => {
			print!("Error creating uEnv.txt: {}\n", e);
			process::exit(1);
		}
	};
	match env_file.write_all(boot_env.as_bytes()) {
		Ok(_) => {},
		Err(e) => {
			print!("Error placing uEnv.txt: {}\n", e);
			process::exit(1);
		}
	};
    */
	println!("Success! {} is ready to boot on the {}! Enjoy Haiku!", output_file.display(), board.name);
}
