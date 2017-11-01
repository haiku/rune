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
use std::fs;
use fatr::fat;
use mbr::partition;

/// Write file at source to dest
pub fn write(source: PathBuf, dest: PathBuf) -> io::Result<u64> {
	return fs::copy(source.as_path(), dest.as_path());
}

/// Validate disk as containing Haiku and locate "boot" partition.
pub fn locate_boot_partition(disk: PathBuf) -> Result<partition::Partition,Box<Error>> {
	let partitions = partition::read_partitions(disk.clone())?;
	for (_, partition) in partitions.iter().enumerate() {
		let sector_size = 512;
		if partition.p_type != 12 {
			// Ignore non-fat partitions
			continue;
		}
		let image = fat::Image::from_file_offset(disk.clone(),
			(partitions[0].p_lba as usize * sector_size), partitions[0].p_size as usize * sector_size)?;
		let volume_id = match image.volume_label() {
			Ok(v) => v,
			Err(_) => continue,
		};
		if !volume_id.to_uppercase().contains("HAIKU") {
			continue;
		}
		// TODO: More checks?
		return Ok(partition.clone());
	}
	return Err(From::from("no Haiku boot partitions"));
}

