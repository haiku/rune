/*
 * Application error normalization
 * Copyright 2016-2017, Alexander von Gluck IV, All rights reserved.
 * Released under the terms of the MIT license.
 */

use std::io;
use std::num;

use std::fmt;

use std::error::Error;

// Error Conversions
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Parse(num::ParseIntError),
    NotFound,
    AlreadyExists,
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> AppError {
        AppError::Io(err)
    }
}

impl From<num::ParseIntError> for AppError {
    fn from(err: num::ParseIntError) -> AppError {
        AppError::Parse(err)
    }
}

// Normalization of Error's
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AppError::Io(ref err) => err.fmt(f),
            AppError::Parse(ref err) => err.fmt(f),
            AppError::NotFound => write!(f, "No matching entries were found"),
            AppError::AlreadyExists => write!(f, "Entry already exists"),
        }
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        match *self {
            AppError::Io(ref err) => err.description(),
            AppError::Parse(ref err) => err.description(),
            AppError::NotFound => "not found",
            AppError::AlreadyExists => "already exists",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            AppError::Io(ref err) => Some(err),
            AppError::Parse(ref err) => Some(err),
            AppError::NotFound => None,
            AppError::AlreadyExists => None,
        }
    }
}
