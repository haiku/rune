
/*
 * Copyright, 2017, Alexander von Gluck IV. All rights reserved.
 * Released under the terms of the MIT license.
 *
 * Authors:
 *   Alexander von Gluck IV <kallisti5@unixzen.com>
 */


use std::path::Path;
use std::fs::File;
use std::io::{Read,SeekFrom};
use std::io::prelude::*;

use apperror::AppError;

// Start +446
#[derive(Debug, Clone)]
pub struct Partition {
	pub p_status: u8,
	pub p_cyl_begin: u8,
	pub p_head_begin: u8,
	pub p_sect_begin: u8,
	pub p_type: u8,
	pub p_cyl_end: u8,
	pub p_head_end: u8,
	pub p_sect_end: u8,
	pub p_lba: u32,
	pub p_size: u32,
}

fn read1<R: Read>(r: &mut R) -> u8 {
	let mut buf = [0];
	r.read(&mut buf).unwrap();
	buf[0]
}

fn read4<R: Read>(r: &mut R) -> u32 {
	let mut buf = [0, 0, 0, 0];
	r.read(&mut buf).unwrap();
	// TODO: Endian issues on non-x86 platforms? (maybe use byteorder crate?)
	//original: (buf[0] as u32) << 24 | (buf[1] as u32) << 16 | (buf[2] as u32) << 8 | (buf[3] as u32)
	(buf[3] as u32) << 24 | (buf[2] as u32) << 16 | (buf[1] as u32) << 8 | (buf[0] as u32)
}

pub fn read_partition(path: String, index: u8) -> Result<Partition, AppError> {
	let mut f = File::open(&Path::new(&path))?;
	assert!(index < 4);

	let position: u64 = 446 + (16 * (index as u64));

	f.seek(SeekFrom::Start(position))?;
	let b = &mut f;

	let new_part = Partition {
		p_status: read1(b),
		p_head_begin: read1(b),
		p_sect_begin: read1(b),
		p_cyl_begin: read1(b),
		p_type: read1(b),
		p_head_end: read1(b),
		p_sect_end: read1(b),
		p_cyl_end: read1(b),
		p_lba: read4(b),
		p_size: read4(b),
	};

	return Ok(new_part);
}

pub fn parse(path: String) -> Result<Vec<Partition>, AppError> {
	let mut partitions: Vec<Partition> = Vec::new();

	for i in [0,1,2,3].iter() {
		partitions.push(read_partition(path.clone(), *i)?);
	}

	return Ok(partitions);
}

pub fn table_dump(partitions: Vec<Partition>) {
    for i in partitions.iter() {
        print!("{:?}\n", i);
    }
}
