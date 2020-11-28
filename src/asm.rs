//! Top-Level Assembler
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

//use std::fmt::{self, Display, Formatter};
use std::fmt::Display;
use std::fs;
use std::io::{stdin, stdout, Read, Write};

use crate::message::*;

/// The type returned by fallible assembler methods.
pub type Result<T=(), E=()> = std::result::Result<T, E>;

/// Top-level assembler interface.
#[derive(Debug)]
pub struct Assembler {
    output:        Vec<u8>,
    warning_count: u16,
    error_count:   u16,
}

impl Assembler {
    /// Creates a new assembler.
    pub fn new() -> Self {
        Self {
            output:        Vec::with_capacity(16 * 1024),
            warning_count: 0,
            error_count:   0,
        }
    }

    pub fn result(&self) -> Result {
        match self.error_count {
            0 => Ok  (()),
            _ => Err (()),
        }
    }

    /// Assembles the file at the given `path`.
    pub fn assemble_file(&mut self, path: &str) -> Result {
        match fs::read_to_string(path) {
            Ok (s) => self.assemble_bytes(path, s.as_bytes()),
            Err(e) => ReadError(path, &e).tell(self),
        }
    }

    /// Assembles the bytes read from standard input.
    pub fn assemble_stdin(&mut self) -> Result {
        self.assemble_from("stdin", stdin())
    }

    /// Assembles the bytes read from `src`, using `path` as the pathname.
    pub fn assemble_from<R: Read>(&mut self, path: &str, mut src: R) -> Result {
        let mut s = String::new();
        match src.read_to_string(&mut s) {
            Ok (_) => self.assemble_bytes(path, s.as_bytes()),
            Err(e) => ReadError(path, &e).tell(self),
        }
    }

    /// Assembles the given `bytes`, using `path` as the pathname.
    pub fn assemble_bytes(&mut self, path: &str, bytes: &[u8]) -> Result {
        // TODO: Everything.  For now just copy input to output.
        self.output.extend_from_slice(path.as_bytes());
        self.output.push(b':');
        self.output.push(b'\n');
        self.output.extend_from_slice(bytes);

        if self.error_count == 0 { Ok(()) } else { Err(()) }
    }

    pub fn write_output(&mut self) -> Result {
        match stdout().write_all(&self.output) {
            Ok (_) => Ok(()),
            Err(e) => WriteError("stdout", &e).tell(self),
        }
    }
}

impl Log for Assembler {
    #[inline]
    fn log<M: Display>(&mut self, msg: M) -> Result {
        eprintln!("{}", msg );
        Ok(())
    }

    #[inline]
    fn log_warning<M: Display>(&mut self, msg: M) -> Result {
        self.warning_count += 1;
        self.log(msg)
    }

    #[inline]
    fn log_error<M: Display>(&mut self, msg: M) -> Result {
        self.error_count += 1;
        self.log(msg)
    }
}
