/*
 * Rune - OS Image preperation tool
 *
 * Copyright, 2017 Haiku, Inc. All rights Reserved.
 * Released under the terms of the MIT license.
 *
 * Authors:
 *   Alexander von Gluck IV <kallisti5@unixzen.com>
 */

use std::error::Error;
use std::path::PathBuf;
use std::io;
use std::io::{Seek, SeekFrom};
use std::fs;
use std::fs::File;
use fatfs::{FileSystem, FsOptions, BufStream};
use mbr::partition;

/// Write file at source to dest
pub fn write(source: PathBuf, dest: PathBuf) -> io::Result<u64> {
	return fs::copy(source.as_path(), dest.as_path());
}

/// Validate disk as containing Haiku and locate "boot" partition.
pub fn locate_boot_partition(disk: PathBuf) -> Result<partition::Partition,Box<dyn Error>> {
	let partitions = partition::read_partitions(disk.clone())?;
	for (_, partition) in partitions.iter().enumerate() {
		let sector_size = 512;
		// Ignore non-efi or non-fat partitions
		match partition.p_type {
			0x0b | 0xef => {},
			_ => continue,
		}
		let disk_handle = File::open(disk.clone())?;
		let mut buf_rdr = BufStream::new(disk_handle);
		buf_rdr.seek(SeekFrom::Start(partitions[0].p_lba as u64 * sector_size))?;
		let fs = match FileSystem::new(&mut buf_rdr, FsOptions::new()) {
			Ok(x) => x,
			Err(_) => continue,
		};
		if !fs.volume_label().to_uppercase().contains("HAIKU") {
			continue;
		}
		// TODO: More checks?
		return Ok(partition.clone());
	}
	return Err(From::from("no Haiku boot partitions"));
}

