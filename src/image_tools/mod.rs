/*
 * Rune - OS Image preperation tool
 *
 * Copyright, 2017 Haiku, Inc. All rights Reserved.
 * Released under the terms of the MIT license.
 *
 * Authors:
 *   Alexander von Gluck IV <kallisti5@unixzen.com>
 */

use apperror::AppError;
use std::path::PathBuf;
use std::io;
use std::fs;

/// Write file at source to dest
pub fn write(source: PathBuf, dest: PathBuf) -> io::Result<u64> {
	return fs::copy(source.as_path(), dest.as_path());
}
