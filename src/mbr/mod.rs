
use std::path::Path;
use std::error::Error;
use std::fs::File;
use std::io::{Read,SeekFrom};
use std::io::prelude::*;

use apperror::AppError;

// Start +446
pub struct partition {
	pub p_status: u8,	  // 1 byte
	pub p_chs_begin: u32,  // 3 byte
	pub p_type: u8,		// 1 byte
	pub p_chs_end: u32,	// 3 byte
	pub p_lba: u32,		// 4 byte
	pub p_size: u32,	   // 4 byte
}

pub fn read_partition(path: String, index: u8) -> Result<partition, AppError> {
	let mut file = File::open(&Path::new(&path))?;
	print!("{}\n", index);
	assert!(index < 4);

	let mut new_part: partition;
	let mut position: u64 = 446 + (16 * (index as u64));

	file.seek(SeekFrom::Start(position))?;
	new_part.p_status = 100;
	new_part.p_chs_begin = 0;
	new_part.p_type = 0;
	new_part.p_chs_end = 0;
	new_part.p_lba = 0;
	new_part.p_size = 0;
	//file.read(&mut new_part.p_status)?;

	//return Ok(new_part);
	return Err(AppError::NotFound);
}

pub fn parse(path: String) -> Result<Vec<partition>, AppError> {
	let mut partitions: Vec<partition> = Vec::new();

	for i in [0,1,2,3].iter() {
		partitions.push(read_partition(path.clone(), *i)?);
	}

	return Ok(partitions);
}
