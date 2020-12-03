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

//! Primary lexical analyzer.

use crate::asm::Result;
use crate::lang::Base;
use crate::lang::token::Token::{self, self as T};

use super::reader::{LogicalChar, Reader};
use super::int::scan_int;

// ----------------------------------------------------------------------------

// Just a helper to define Char variants
const fn char(n: u16) -> u16 {
    n * State::COUNT as u16
}

/// Logical characters recognized by the main lexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
enum Char {
    // space, newlines
    Space   = char( 0), // \s\t
    Cr      = char( 1), // \r
    Lf      = char( 2), // \n
    // identifiers, numbers
    Ident   = char( 3), // A-Za-z., code points above U+007F
    Digit   = char( 4), // 0-9
    // open/close pairs
    LParen  = char( 5), // (
    RParen  = char( 6), // )
    LSquare = char( 7), // [
    RSquare = char( 8), // ]
    LCurly  = char( 9), // {
    RCurly  = char(10), // }
    // quotes
    DQuote  = char(11), // "
    SQuote  = char(12), // '
    // isolated characters, ordered by descending frequency
    Comma   = char(13), // ,
    Hash    = char(14), // #
    Equal   = char(15), // =
    Plus    = char(16), // +
    Minus   = char(17), // -
    Amp     = char(18), // &
    Pipe    = char(19), // |
    Caret   = char(20), // ^
    Lt      = char(21), // <
    Gt      = char(22), // >
    Tilde   = char(23), // ~
    Bang    = char(24), // !
    Star    = char(25), // *
    Slash   = char(26), // /
    Pct     = char(27), // %
    Semi    = char(28), // ;
    Colon   = char(29), // :
    Quest   = char(30), // ?
    Dollar  = char(31), // $
    At      = char(32), // @    unsure if this will be used
    BSlash  = char(33), // \
    // rare
    Eof     = char(34), // end of file
    Other   = char(35), // everything else

    // NOTE: backquote ` is not used
}

impl Char {
    /// Count of `Char` logical characters.
    const COUNT: usize = Self::Other as usize / State::COUNT + 1;
}

impl LogicalChar for Char {
    const NON_ASCII: Self = Self::Ident;
    const EOF:       Self = Self::Eof;
}

/// Mapping of 7-bit ASCII to logical characters.
static CHARS: [Char; 128] = {
    use Char::*;
    const __: Char = Other;
[
//  x0      x1      x2      x3      x4      x5      x6      x7
//  x8      x9      xA      xB      xC      xD      xE      xF
    __,     __,     __,     __,     __,     __,     __,     __,     // 0x │········│
    __,     Space,  Lf,     __,     __,     Cr,     __,     __,     // 0x │·tn··r··│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 1x │········│
    Space,  Bang,   DQuote, Hash,   Dollar, Pct,    Amp,    SQuote, // 2x │ !"#$%&'│
    LParen, RParen, Star,   Plus,   Comma,  Minus,  Ident,  Slash,  // 2x │()*+,-./│
    Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  // 3x │01234567│
    Digit,  Digit,  Colon,  Semi,   Lt,     Equal,  Gt,     Quest,  // 3x │89:;<=>?│
    At,     Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 4x │@ABCDEFG│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 4x │HIJKLMNO│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 5x │PQRSTUVW│
    Ident,  Ident,  Ident,  LSquare,BSlash, RSquare,Caret,  Ident,  // 5x │XYZ[\]^_│
    __,     Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 6x │`abcdefg│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 6x │hijklmno│
    Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  Ident,  // 7x │pqrstuvw│
    Ident,  Ident,  Ident,  LCurly, Pipe,   RCurly, Tilde,  __,     // 7x │xyz{|}~░│
]};

// ---------------------------------------------------------------------------- 

/// Lexer states.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    /// Normal state.  Any token is possible.
    Normal,

    /// At the begining of a line.  Any token is possible.
    Bol,

    /// After a carriage return (0x0D).  Line feed (0x0A) is expected.
    AfterCr,

    /// After a integer or magnitude.
    AfterInt,

    /// In a comment.
    Comment,
}

impl State {
    /// Count of lexer states.
    const COUNT: usize = Self::Comment as usize + 1;
}

// ----------------------------------------------------------------------------

// Transition IDs.  Each ID is an index into `TRANSITION_LUT`, which contains
// the details of the transition.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum TransitionId {
    /// Transition to `Normal` state and continue scanning.
    Normal,

    /// Transition to `Bol` state and continue scanning.
    Bol,

    /// Transition to `AfterCr` state and continue scanning.
    Cr,

    /// Transition to `AfterCr` state and emit an `Eos` token.
    CrEos,

    /// Transition to `Bol` state, increment line, and continue scanning.
    Eol,

    /// Transition to `Bol` state, increment line, and emit an `Eos` token.
    EolEos,

    /// Transition to `Comment` state and continue scanning.
    Comment,

    /// Transition to `Comment` state and emit an `Eos` token.
    CommentEos,

    /// Transition to `AfterInt` state, un-read a byte, and scan an integer.
    IntDec,

    /// Transition to `Normal` state, un-read a byte, and emit an `Int` token.
    Int,

    /// Transition to `Normal` state and emit a `LParen` token.
    LParen,

    /// Transition to `Normal` state and emit a `RParen` token.
    RParen,

    /// Transition to `Normal` state and emit a `LSquare` token.
    LSquare,

    /// Transition to `Normal` state and emit a `RSquare` token.
    RSquare,

    /// Transition to `Normal` state and emit a `LCurly` token.
    LCurly,

    /// Transition to `Normal` state and emit a `RCurly` token.
    RCurly,

    /// Transition to `Normal` state and emit a `Comma` token.
    Comma,

    /// Transition to `Normal` state and emit a `Colon` token.
    Colon,

    /// Terminate with failure.
    Error,

    /// Terminate with success.
    End,
}

impl TransitionId {
    /// Count of transition IDs.
    const COUNT: usize = Self::End as usize + 1;
}

/// Lexer state transition map.
static TRANSITION_MAP: [TransitionId; State::COUNT * Char::COUNT] = {
    use TransitionId::*;
[
//          Normal      Bol         AfterCr     AfterInt    Comment
//          ------------------------------------------------------------------------
/* Space */ Normal,     Bol,        Error,      Int,        Comment,
/* Cr    */ CrEos,      Cr,         Error,      Int,        Cr,
/* Lf    */ EolEos,     Eol,        Eol,        Int,        Eol,
                                                            
/* Ident */ Error,      Error,      Error,      Error,      Comment,
/* Digit */ IntDec,     IntDec,     Error,      Error,      Comment,
                                                            
/*   (   */ LParen,     LParen,     Error,      Int,        Comment,
/*   )   */ RParen,     RParen,     Error,      Int,        Comment,
/*   [   */ LSquare,    LSquare,    Error,      Int,        Comment,
/*   ]   */ RSquare,    RSquare,    Error,      Int,        Comment,
/*   {   */ LCurly,     LCurly,     Error,      Int,        Comment,
/*   }   */ RCurly,     RCurly,     Error,      Int,        Comment,
/*   "   */ Error,      Error,      Error,      Int,        Comment,
/*   '   */ Error,      Error,      Error,      Int,        Comment,

/*   ,   */ Comma,      Comma,      Error,      Int,        Comment,
/*   #   */ CommentEos, Comment,    Error,      Int,        Comment,
/*   =   */ Error,      Error,      Error,      Int,        Comment,
/*   +   */ Error,      Error,      Error,      Int,        Comment,
/*   -   */ Error,      Error,      Error,      Int,        Comment,
/*   &   */ Error,      Error,      Error,      Int,        Comment,
/*   |   */ Error,      Error,      Error,      Int,        Comment,
/*   ^   */ Error,      Error,      Error,      Int,        Comment,
/*   <   */ Error,      Error,      Error,      Int,        Comment,
/*   >   */ Error,      Error,      Error,      Int,        Comment,
/*   ~   */ Error,      Error,      Error,      Int,        Comment,
/*   !   */ Error,      Error,      Error,      Int,        Comment,
/*   *   */ Error,      Error,      Error,      Int,        Comment,
/*   /   */ Error,      Error,      Error,      Int,        Comment,
/*   %   */ Error,      Error,      Error,      Int,        Comment,
/*   ;   */ Error,      Error,      Error,      Int,        Comment,
/*   :   */ Colon,      Colon,      Error,      Int,        Comment,
/*   ?   */ Error,      Error,      Error,      Int,        Comment,
/*   $   */ Error,      Error,      Error,      Int,        Comment,
/*   @   */ Error,      Error,      Error,      Int,        Comment,
/*   \   */ Error,      Error,      Error,      Int,        Comment,
                                                            
/* Eof   */ End,        End,        Error,      Int,        End,
/* Other */ Error,      Error,      Error,      Error,      Comment,
]};

// ----------------------------------------------------------------------------

/// Lexer transition.
#[derive(Clone, Copy, Debug)]
struct Transition {
    state:  State,  // 1 byte
    action: Action, // 2 bytes
    flags:  u8,     // 1 byte
    // 0b000000LT
    //         │└Token increment
    //         └─Line increment
}

impl Transition {
    #[inline]
    fn token_inc(&self) -> usize {
        (self.flags & 1) as usize
    }

    #[inline]
    fn line_inc(&self) -> usize {
        (self.flags >> 1) as usize
    }
}

/// Lexer transitions in order by transition ID.
static TRANSITION_LUT: [Transition; TransitionId::COUNT] = {
    use TransitionId as X;
    use Action::*;
    use State::*;
    const fn t(_: TransitionId, state: State, action: Action, flags: u8) -> Transition {
        Transition { state, action, flags }
    }
[
//                      New                              +len┐
//    TransitionId      State       Action            +line┐ │
// ------------------------------------------------------------
// Whitespace                                              │ │
    t(X::Normal,        Normal,     Continue,           0b_0_0),
    t(X::Bol,           Bol,        Continue,           0b_0_0),
    t(X::Cr,            AfterCr,    Continue,           0b_0_0),
    t(X::CrEos,         AfterCr,    Yield(T::Eos),      0b_0_0),
    t(X::Eol,           Bol,        Continue,           0b_1_0),
    t(X::EolEos,        Bol,        Yield(T::Eos),      0b_1_0),
// Comments                                                │ │
    t(X::Comment,       Comment,    Continue,           0b_0_0),
    t(X::CommentEos,    Comment,    Yield(T::Eos),      0b_0_0),
// Numbers
    t(X::IntDec,        AfterInt,   ScanDec,            0b_0_1),
    t(X::Int,           Normal,     UYield(T::Int),     0b_0_0),
// Simple Tokens
    t(X::LParen,        Normal,     Yield(T::LParen),   0b_0_1),
    t(X::RParen,        Normal,     Yield(T::RParen),   0b_0_1),
    t(X::LSquare,       Normal,     Yield(T::LSquare),  0b_0_1),
    t(X::RSquare,       Normal,     Yield(T::RSquare),  0b_0_1),
    t(X::LCurly,        Normal,     Yield(T::LCurly),   0b_0_1),
    t(X::RCurly,        Normal,     Yield(T::RCurly),   0b_0_1),
    t(X::Comma,         Normal,     Yield(T::Comma),    0b_0_1),
    t(X::Colon,         Normal,     Yield(T::Colon),    0b_0_1),
// Termination                                             │ │
    t(X::Error,         Normal,     Fail,               0b_0_0),
    t(X::End,           Normal,     Succeed,            0b_0_0),
]};

// ----------------------------------------------------------------------------

/// Lexer actions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    /// Continue scanning.
    Continue,

    // === Sublexers ===
    
    /// Scan a binary numeric literal.
    ScanBin,

    /// Scan an octal numeric literal.
    ScanOct,

    /// Scan a decimal numeric literal.
    ScanDec,

    /// Scan a hexadecimal numeric literal.
    ScanHex,

    /// Scan a string.
    ScanStr,

    // === Tokens ===

    /// Yield an identifier.
    YieldIdent,

    /// Yield a label.
    YieldLabel,

    /// Yield a macro parameter.
    ///
    YieldParam,

    /// Yield a character literal.
    YieldChar,

    /// Yield a token.
    Yield(Token),

    /// Unread a byte and yield a token.
    UYield(Token),

    // === Terminators ===

    /// Terminate unsuccessfully.
    Fail,

    /// Terminate successfully.
    Succeed,
}

// ---------------------------------------------------------------------------- 

/// Lexical analyzer.  Reads input and yields a stream of lexical tokens.
#[derive(Debug)]
pub struct Lexer<'a> {
    input: Reader<'a>,
    state: State,

    // Token info
    line:  usize,
    len:   usize,
    mag:   u64,
}

impl<'a> Lexer<'a> {
    /// Creates a lexical analyzer that takes as input the contents of the
    /// given slice of bytes.
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input: Reader::new(input),
            state: State::Bol,
            line:  1,
            len:   0,
            mag:   0,
        }
    }

    /// Returns the source line number (1-indexed) of the current token.
    #[inline]
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the source text of the current token.
    #[inline]
    pub fn text(&self) -> &'a [u8] {
        self.input.preceding(self.len)
    }

    /// Returns the integer magnitude of the current token.
    #[inline]
    pub fn magnitude(&self) -> u64 {
        self.mag
    }

    /// Advances to the next token and returns its type.
    pub fn next(&mut self) -> Token {
        use Action::*;

        // Restore saved state and prepare for loop
        let mut state   = self.state;
        let mut line    = self.line;
        let mut len     = 0;
        let mut len_inc = 0;
        let mut action;

        // Discover next token
        let token = loop {
            // Get next transition
            let next = self.input.read(&CHARS).0;
            let next = TRANSITION_MAP[state as usize + next as usize];
            let next = TRANSITION_LUT[next  as usize];

            // Update state
            state    = next.state;
            action   = next.action;
            line    += next.line_inc();
            len_inc |= next.token_inc();
            len     += len_inc;

            // Perform action
            match action {
                Continue => continue,

                // Sublexers
                ScanBin => { if self.scan_mag(Base::Bin).is_err() { break Token::Error; } },
                ScanOct => { if self.scan_mag(Base::Oct).is_err() { break Token::Error; } },
                ScanDec => { if self.scan_mag(Base::Dec).is_err() { break Token::Error; } },
                ScanHex => { if self.scan_mag(Base::Hex).is_err() { break Token::Error; } },
                ScanStr => continue, // self.scan_str(),

                // Identifiers & Literals
                YieldIdent   => break Token::Ident,
                YieldLabel   => break Token::Label,
                YieldParam   => break Token::Param,
                YieldChar    => break Token::Char,

                // Simple Tokens
                Yield(token)  => break token,
                UYield(token) => { self.input.unread(); break token },

                // Terminators
                Succeed      => break Token::Eof,
                Fail         => break Token::Error,
            }
        };

        // Save state for subsequent invocation
        self.state = state;
        self.line  = line;
        self.len   = len;

        token
    }

    fn scan_mag(&mut self, base: Base) -> Result {
        // Un-read first digit so that sublexer sees it
        self.input.unread();

        match scan_int(&mut self.input, base) {
            (_, 0) => {
                // overflow
                self.mag = 0;
                self.len = 0;
                Err(())
            },
            (v, l) => {
                // success
                self.mag  = v;
                self.len += l as usize;
                Ok(())
            }
        }
    }
}
