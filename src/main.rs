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
extern crate fatr;
extern crate reqwest;
extern crate tempdir;
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
use std::io::Read;
use std::io;
use std::fs::File;
use getopts::Options;
use fatr::fat;
use tempdir::TempDir;
use url::Url;
use indicatif::{ProgressBar,ProgressStyle};

use itertools::Itertools;

mod boards;
mod image_tools;

fn print_usage(program: &str, opts: Options) {
	let brief = format!("rune - write bootable ARM Haiku mmc images\nUsage: {} [options] <output>", program);
	print!("{}", opts.usage(&brief));
}

fn flag_error(program: &str, opts: Options, error: &str) {
	print!("Error: {}\n\n", error);
	print_usage(&program, opts);
}

fn place_files(board: boards::Board, fatimage: &mut fat::Image, steps: u32) -> Result<u32, Box<Error>> {

	let temp_dir = TempDir::new("rune")?;
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

		// Don't overwrite a preexisting file.
		if let Ok(_) = fatimage.get_file_entry(filename.clone().to_string()) {
			//println!("  Skipping {} since it already exists in image.", i);
			continue;
		}

		//println!("  GET {} {:?}", url, filename);
		let file_path = temp_dir.path().join(filename);

		{
			// Download file per manifest to temporary path
			let mut resp = reqwest::get(url.clone())?;
			if !resp.status().is_success() {
				return Err(From::from(format!("Error obtaining {}", i)));
			}
			let mut new_file = File::create(file_path.clone())?;
			io::copy(&mut resp, &mut new_file)?;
			new_file.sync_all()?;
		}

		bar.set_message(&format!("Writing: {}", filename));
		bar.inc(1);

		let file = File::open(file_path.clone())?;
		let metadata = file.metadata()?;

		// Create a root dir entry.
		let (entry, index) =
			fatimage.create_file_entry(filename.to_string(), metadata.len() as u32)?;

		// Get free FAT entries, fill sectors with file data.
		for chunk in &file.bytes().chunks(fatimage.sector_size()) {
			let chunk = chunk
				.map(|b_res| b_res.unwrap_or(0))
				.collect::<Vec<_>>();

			// Get free sector.
			let entry_index: usize;
			match fatimage.get_free_fat_entry() {
				Some(i) => entry_index = i,
				None => {
					// TODO: Remove entries written so far.
					panic!("image ran out of space while writing file")
				},
			}

			// Write chunk.
			fatimage.write_data_sector(entry_index, &chunk)?;
		}

		fatimage.save_file_entry(entry, index)?;
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
	let mut image = match fat::Image::from_file_offset(output_file.clone(),
		boot_partition.p_lba as usize * sector_size,
		boot_partition.p_size as usize * sector_size) {
		Ok(x) => x,
		Err(e) => {
			print!("Error: {}\n", e);
			process::exit(1);
		}
	};

	let count = match place_files(board.clone(), &mut image, steps) {
		Ok(i) => i,
		Err(e) => {
			print!("Error: {}\n", e);
			process::exit(1);
		}
	};
	print!("Obtained {} boot-related files.\n", count);
	match image.save(output_file.clone()) {
		Ok(_) => {},
		Err(e) => {
			print!("Error: {}\n", e);
			process::exit(1);
		}
	};
}
