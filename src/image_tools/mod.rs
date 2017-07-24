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

 pub fn write(source: PathBuf, dest: PathBuf) {
	 	println!("Writing {:?} to {:?}...", source, dest);
 }
