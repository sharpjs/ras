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

use super::State;
use super::reader::LogChar;

// Just a helper to define Char variants
const fn char(n: u16) -> u16 {
    n * State::COUNT as u16
}

/// Logical characters recognized by the main lexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
enum Char {
    // Ordered roughly by descending frequency.
    // space, newlines
    Space   = char( 0), // \s\t
    Cr      = char( 1), // \r
    Lf      = char( 2), // \n
    // identifiers, numbers
    Id      = char( 3), // A-Za-z., code points above U+007F
    LetB    = char( 4), // Bb
    LetD    = char( 5), // Dd
    LetO    = char( 6), // Oo
    LetX    = char( 7), // Xx
    Digit   = char( 8), // 0-9
    // open/close pairs
    LParen  = char( 9), // (
    RParen  = char(10), // )
    LSquare = char(11), // [
    RSquare = char(12), // ]
    LCurly  = char(13), // {
    RCurly  = char(14), // }
    // quotes
    DQuote  = char(15), // "
    SQuote  = char(16), // '
    // isolated characters
    Comma   = char(17), // ,
    Hash    = char(18), // #
    Equal   = char(19), // =
    Plus    = char(20), // +
    Minus   = char(21), // -
    Amper   = char(22), // &
    Pipe    = char(23), // |
    Caret   = char(24), // ^
    Lt      = char(25), // <
    Gt      = char(26), // >
    Tilde   = char(27), // ~
    Bang    = char(28), // !
    Star    = char(29), // *
    Slash   = char(30), // /
    Percent = char(31), // %
    Semi    = char(32), // ;
    Colon   = char(33), // :
    Quest   = char(34), // ?
    Dollar  = char(35), // $
    At      = char(36), // @    unsure if this will be used
    BSlash  = char(37), // \
    // rare
    Eof     = char(38), // end of file
    Other   = char(39), // everything else
}

impl Char {
    /// Count of `Char` logical characters.
    const COUNT: usize = Self::Other as usize / State::COUNT + 1;
}

impl LogChar for Char {
    const EXT: Self = Self::Id;
    const EOF: Self = Self::Eof;
}

/// Mapping of 7-bit ASCII bytes to `Char` logical characters.
static CHARS: [Char; 128] = {
    use Char::*;
[
//  7-bit ASCII characters
//  x0      x1      x2      x3      x4      x5      x6      x7      CHARS
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // ........
    Other,  Space,  Lf,     Other,  Other,  Cr,     Other,  Other,  // .tn..r..
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // ........
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // ........
    Space,  Bang,   DQuote, Hash,   Dollar, Percent,Amper,  SQuote, //  !"#$%&'
    LParen, RParen, Star,   Plus,   Comma,  Minus,  Id,     Slash,  // ()*+,-./
    Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  // 01234567
    Digit,  Digit,  Colon,  Semi,   Lt,     Equal,  Gt,     Quest,  // 89:;<=>?
    At,     Id,     LetB,   Id,     LetD,   Id,     Id,     Id,     // @ABCDEFG
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     LetO,   // HIJKLMNO
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // PQRSTUVW
    LetX,   Id,     Id,     LSquare,BSlash, RSquare,Caret,  Id,     // XYZ[\]^_
    Other,  Id,     LetB,   Id,     LetD,   Id,     Id,     Id,     // `abcdefg
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     LetO,   // hijklmno
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // pqrstuvw
    LetX,   Id,     Id,     LCurly, Pipe,   RCurly, Tilde,  Other,  // xyz{|}~. <- DEL
]};
