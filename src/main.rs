/*
 * Rune - OS Image preperation tool
 *
 * Copyright, 2017 Haiku, Inc. All rights Reserved.
 * Released under the terms of the MIT license.
 *
 * Authors:
 *   Alexander von Gluck IV <kallisti5@unixzen.com>
 */

extern crate getopts;
extern crate mbr;
extern crate fatfs;
extern crate reqwest;
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
use std::io::Write;
use std::fs;
use std::fs::OpenOptions;
use fatfs::{BufStream, FileSystem, FsOptions};
use getopts::Options;
use url::Url;
use indicatif::{ProgressBar,ProgressStyle};
use partition::Partition;

mod boards;
mod partition;
mod image_tools;

fn print_usage(program: &str, opts: Options) {
	let brief = format!("rune - write bootable ARM Haiku mmc images\nUsage: {} [options] <output>", program);
	print!("{}", opts.usage(&brief));
}

fn flag_error(program: &str, opts: Options, error: &str) {
	print!("Error: {}\n\n", error);
	print_usage(&program, opts);
}

fn place_files(board: boards::Board, target_fs: &mut fatfs::FileSystem, steps: u32) -> Result<u32, Box<Error>> {
	let count = board.files.len() as u32;
	if count == 0 {
		return Err(From::from("No files found for board!"));
	}
	let bar = ProgressBar::new((count * 2) as u64);
	bar.set_style(ProgressStyle::default_bar()
		.template("{prefix} {spinner:.bold}[{bar:40.cyan/blue}] {msg:.bold.dim}")
		.tick_chars("◐◓◑◒")
		.progress_chars("#>-"));
	bar.set_prefix(&format!("[{}/{}] Provisioning filesystem...", steps, steps));
	for i in board.files {
		let url = Url::parse(i.as_str())?;
		let filename = match url.path_segments() {
			Some(x) => x.last().unwrap(),
			None => return Err(From::from(format!("Invalid URL {}", i))),
		};

		bar.set_message(&format!("Downloading: {}", filename));
		bar.inc(1);

		//println!("  GET {} {:?}", url, filename);

		// Download file per manifest
		let mut resp = reqwest::get(url.clone())?;
		if !resp.status().is_success() {
			return Err(From::from(format!("Error obtaining {}", i)));
		}
		bar.set_message(&format!("Writing: {}", filename));
		bar.inc(1);

		let mut target_file = target_fs.root_dir().create_file(filename)?;
		io::copy(&mut resp, &mut target_file)?;
	}
	bar.finish();
	return Ok(count);
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	let mut opts = Options::new();
	opts.optopt("b", "board", "target board", "<board>");
	opts.optopt("i", "image", "source OS image", "<image>");
	opts.optflag("l", "list", "list supported target boards");
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
	} else if matches.opt_present("l") {
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
			println!("Error: {}", e.description());
			process::exit(1);
		},
	};

	let mut steps = 2;
	if matches.opt_present("i") {
		steps = 3;
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

	let count = match place_files(board.clone(), &mut fs, steps) {
		Ok(i) => i,
		Err(e) => {
			print!("Error Placing Files: {}\n", e);
			process::exit(1);
		}
	};
	print!("Obtained {} boot-related files.\n", count);

	let boot_script
		= boards::get_boot_script("haiku_loader.ub".to_string(), "haiku-floppyboot.tgz.ub".to_string(), board.id);

	let mut boot_file = match fs.root_dir().create_file("boot.scr") {
		Ok(o) => o,
		Err(e) => {
			print!("Error Placing boot.scr: {}\n", e);
			process::exit(1);
		}
	};
	match boot_file.write_all(boot_script.as_bytes()) {
		Ok(_) => {},
		Err(e) => {
			print!("Error Placing boot.scr: {}\n", e);
			process::exit(1);
		}
	};
	println!("{} is ready to boot on the {}!", output_file.display(), board.name);
}
