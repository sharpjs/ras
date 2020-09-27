// This file is part of ras, an assembler.
// Copyright (C) 2020 Jeffrey Sharp
//
// ras is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published
// by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
//
// ras is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See
// the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ras.  If not, see <http://www.gnu.org/licenses/>.

#![allow(dead_code)]

mod arch;
mod lang;
mod mem;
mod message;
mod num;
mod util;
mod value;

use std::env::args;
use std::io::{self, stdin, stdout, stderr, Read, Write};
use std::fs::File;

/// The name of the assembler.
pub const PROGRAM_NAME: &str = "ras";

fn main() -> io::Result<()> {
    let mut args = args();
    args.next();

    let mut buffer = String::new();

    if args.len() == 0 {
        writeln!(stderr(), "reading stdin")?;
        stdin().read_to_string(&mut buffer)?;
    } else {
        for arg in args {
            if arg == "-" {
                writeln!(stderr(), "reading stdin")?;
                stdin().read_to_string(&mut buffer)?;
            } else {
                writeln!(stderr(),"reading {}", arg)?;
                File::open(arg)?.read_to_string(&mut buffer)?;
            }
        }
    }

    write!(stdout(), "{}", buffer)?;
    Ok(())
}

