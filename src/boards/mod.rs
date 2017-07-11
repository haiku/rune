/*
 * Rune - OS Image preperation tool
 *
 * Copyright, 2017 Haiku, Inc. All rights Reserved.
 * Released under the terms of the MIT license.
 *
 * Authors:
 *   Alexander von Gluck IV <kallisti5@unixzen.com>
 */

#[derive(Clone, Debug)]
pub struct Board {
    pub id:   &'static str,
    pub soc:  &'static str,
    pub name: &'static str,
    pub arch: &'static str,
    pub files: &'static [&'static str],
}

const KNOWN_BOARDS: [Board; 2] = [
    Board {
        id:   "rpi2",
        soc:  "BCM2836",
        name: "Raspberry Pi 2",
        arch: "arm",
        files: &["http://google.com"],
    },
    Board {
        id:   "rpi3",
        soc:  "BCM2837",
        name: "Raspberry Pi 3",
        arch: "arm",
        files: &["http://google.com"],
    },
];

pub fn available(arch: String) -> Option<Vec<Board>> {
    let mut results: Vec<Board> = Vec::new();
    for board in KNOWN_BOARDS.iter() {
        if board.arch == &arch {
            results.push(board.clone());
        }
    }
    return Some(results);
}

pub fn print() {
    let arm_boards = match available("arm".to_string()) {
        Some(m) => { m },
        None => return,
    };
    print!("arm\n===\n");
    print!("  {:10} {:10} {:20}\n", "Board", "SOC", "Name");
    for board in arm_boards {
        print!("  {:10} {:10} {:20}\n", board.id, board.soc, board.name);
    }
}

