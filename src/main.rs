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

use std::env;
use getopts::Options;

mod boards;
mod apperror;

fn print_usage(program: &str, opts: Options) {
	let brief = format!("rune - bless and write Haiku mmc images\nUsage: {} [options] <output>", program);
	print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
	let mut opts = Options::new();
	opts.optopt("b", "board", "target board", "<board>");
	opts.optopt("i", "image", "mmc image", "<image>");
    opts.optflag("l", "list", "list supported target boards");
	opts.optflag("h", "help", "print this help");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m },
		Err(f) => {
			println!("Error: {}", f.to_string());
			return;
		}
	};

	if matches.opt_present("h") {
		print_usage(&program, opts);
		return;
	} else if matches.opt_present("l") {
        boards::print();
        return;
    }

    println!("Hello!")
}
