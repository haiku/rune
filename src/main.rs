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

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::error::Error;
use std::process;
use std::path::PathBuf;
use std::env;
use getopts::Options;
use apperror::AppError;
use mbr::partition;

mod boards;
mod image_tools;
mod apperror;


fn print_usage(program: &str, opts: Options) {
	let brief = format!("rune - bless and write Haiku mmc images\nUsage: {} [options] <output>", program);
	print!("{}", opts.usage(&brief));
}

fn flag_error(program: &str, opts: Options, error: &str) {
	print!("Error: {}\n\n", error);
	print_usage(&program, opts);
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	let mut opts = Options::new();
	opts.optopt("b", "board", "target board", "<board>");
	opts.optopt("i", "image", "source OS image", "<image>");
	opts.optflag("l", "list", "list supported target boards");
	opts.optflag("v", "verbose", "increase verbosity");
	opts.optflag("h", "help", "print this help");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m },
		Err(f) => {
			println!("Error: {}", f.to_string());
			process::exit(1);
		}
	};

	let verbose = matches.opt_present("v");

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
		Err(AppError::NotFound) => {
			flag_error(&program, opts, "Unknown target board!");
			process::exit(1);
		},
		Err(e) => {
			println!("Error: {}", e.description());
			process::exit(1);
		},
	};
	let source_image = match matches.opt_str("i") {
		Some(x) => PathBuf::from(&x),
		None => {
			flag_error(&program, opts, "Source OS image not provided!");
			process::exit(1);
		},
	};
	let partitions = match partition::read_from_file(source_image.clone()) {
		Ok(x) => x,
		Err(e) => {
			print!("Error: {}\n", e);
			process::exit(1);
		},
	};

	if verbose {
		print!("Scan partition table in source OS image...\n");
		partition::table_dump(partitions.clone());
	}

	image_tools::write(source_image, output_file);
}
