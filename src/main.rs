//! Program Entry Point and Crate Root
//
// This file is part of ras, an assembler.
// Copyright 2020 Jeffrey Sharp
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
mod asm;
mod lang;
mod mem;
mod message;
mod num;
mod util;
mod value;

use std::env::args;
use std::process::exit;

use self::asm::{Assembler, Result};

/// The name of the assembler.
pub const PROGRAM_NAME: &str = "ras";

fn main() {
    if run().is_err() { exit(1) }
}

fn run() -> Result {
    // This function translates command-line arguments into actions that
    // configure, invoke, and save the output of the assembler.
    let mut args = args();

    // Ignore the first argument (usually executable path, but not guaranteed).
    args.next();

    // The Assembler type provides a facade for the assembler.  It provides a
    // method for each command-line option.
    let mut asm = Assembler::new();

    // TODO: Process non-source-file arguments here.

    // Consume source files.
    if args.len() == 0 {
        asm.assemble_stdin()?;
    } else {
        for arg in args {
            if arg == "-" {
                asm.assemble_stdin()?;
            } else {
                asm.assemble_file(&arg)?;
            }
        }
    }

    // Write output
    if asm.result().is_ok() {
        asm.write_output()?;
    }

    asm.result()
}
