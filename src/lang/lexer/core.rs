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
    Id      = char( 3), // A-Za-z., code points above U+007F
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
    Amper   = char(18), // &
    Pipe    = char(19), // |
    Caret   = char(20), // ^
    Lt      = char(21), // <
    Gt      = char(22), // >
    Tilde   = char(23), // ~
    Bang    = char(24), // !
    Star    = char(25), // *
    Slash   = char(26), // /
    Percent = char(27), // %
    Semi    = char(28), // ;
    Colon   = char(29), // :
    Quest   = char(30), // ?
    Dollar  = char(31), // $
    At      = char(32), // @    unsure if this will be used
    BSlash  = char(33), // \
    // rare
    Eof     = char(34), // end of file
    Other   = char(35), // everything else
}

impl Char {
    /// Count of `Char` logical characters.
    const COUNT: usize = Self::Other as usize / State::COUNT + 1;
}

impl LogicalChar for Char {
    const NON_ASCII: Self = Self::Id;
    const EOF:       Self = Self::Eof;
}

/// Mapping of 7-bit ASCII to logical characters.
static CHARS: [Char; 128] = {
    use Char::*;
    const __: Char = Other;
[
//  xx0     xx1     xx2     xx3     xx4     xx5     xx6     xx7
    __,     __,     __,     __,     __,     __,     __,     __,     // 00x │········│
    __,     Space,  Lf,     __,     __,     Cr,     __,     __,     // 01x │·tn··r··│
    __,     __,     __,     __,     __,     __,     __,     __,     // 02x │········│
    __,     __,     __,     __,     __,     __,     __,     __,     // 03x │········│
    Space,  Bang,   DQuote, Hash,   Dollar, Percent,Amper,  SQuote, // 04x │ !"#$%&'│
    LParen, RParen, Star,   Plus,   Comma,  Minus,  Id,     Slash,  // 05x │()*+,-./│
    Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  Digit,  // 06x │01234567│
    Digit,  Digit,  Colon,  Semi,   Lt,     Equal,  Gt,     Quest,  // 07x │89:;<=>?│
    At,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 10x │@ABCDEFG│
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 11x │HIJKLMNO│
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 12x │PQRSTUVW│
    Id,     Id,     Id,     LSquare,BSlash, RSquare,Caret,  Id,     // 13x │XYZ[\]^_│
    __,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 14x │`abcdefg│
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 15x │hijklmno│
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 16x │pqrstuvw│
    Id,     Id,     Id,     LCurly, Pipe,   RCurly, Tilde,  __,     // 17x │xyz{|}~·│
]};

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

    /// Transition to `Bol` state and emit an `Eos` token.
    BolEos,

    /// Transition to `AfterCr` state and continue scanning.
    Cr,

    /// Transition to `AfterCr` state and emit an `Eos` token.
    CrEos,

    /// Transition to `Bol` state and continue scanning. [Do not increment line]
    CrLf,

    /// Transition to `Comment` state and continue scanning.
    Comment,

    /// Transition to `Comment` state and emit an `Eos` token.
    CommentEos,

    /// Transition to `Normal` state and emit a `ParenL` token.
    ParenL,

    /// Transition to `Normal` state and emit a `ParenR` token.
    ParenR,

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
//          Normal    Bol       AfterCr   Comment
//          ------------------------------------------------------------------------
/* Space */ Normal,   Bol,      Bol,      Comment,
/* Cr    */ CrEos,    Cr,       Cr,       Cr,
/* Lf    */ BolEos,   Bol,      CrLf,     Bol,

/* Id    */ Error,    Error,    Error,    Comment,
/* Digit */ Error,    Error,    Error,    Comment,

/*   (   */ ParenL,   ParenL,   ParenL,   Comment,
/*   )   */ ParenR,   ParenR,   ParenR,   Comment,
/*   [   */ Error,    Error,    Error,    Comment,
/*   ]   */ Error,    Error,    Error,    Comment,
/*   {   */ Error,    Error,    Error,    Comment,
/*   }   */ Error,    Error,    Error,    Comment,
/*   "   */ Error,    Error,    Error,    Comment,
/*   '   */ Error,    Error,    Error,    Comment,

/*   ,   */ Error,    Error,    Error,    Comment,
/*   #   */ CommentEos,Comment, Comment,  Comment,
/*   =   */ Error,    Error,    Error,    Comment,
/*   +   */ Error,    Error,    Error,    Comment,
/*   -   */ Error,    Error,    Error,    Comment,
/*   &   */ Error,    Error,    Error,    Comment,
/*   |   */ Error,    Error,    Error,    Comment,
/*   ^   */ Error,    Error,    Error,    Comment,
/*   <   */ Error,    Error,    Error,    Comment,
/*   >   */ Error,    Error,    Error,    Comment,
/*   ~   */ Error,    Error,    Error,    Comment,
/*   !   */ Error,    Error,    Error,    Comment,
/*   *   */ Error,    Error,    Error,    Comment,
/*   /   */ Error,    Error,    Error,    Comment,
/*   %   */ Error,    Error,    Error,    Comment,
/*   ;   */ Error,    Error,    Error,    Comment,
/*   :   */ Error,    Error,    Error,    Comment,
/*   ?   */ Error,    Error,    Error,    Comment,
/*   $   */ Error,    Error,    Error,    Comment,
/*   @   */ Error,    Error,    Error,    Comment,
/*   \   */ Error,    Error,    Error,    Comment,

/* Eof   */ End,      End,      End,      End,
/* Other */ Error,    Error,    Error,    Comment,
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
    use TransitionId as Id;
    use Action::*;
    use State::*;
    const fn t(_: TransitionId, state: State, action: Action, flags: u8) -> Transition {
        Transition { state, action, flags }
    }
[
//
//                                               Token┐
//                                           Newline┐ │
// Whitespace                                       │ │
    t(Id::Normal,     Normal,  Continue,         0b_0_0),
    t(Id::Bol,        Bol,     Continue,         0b_1_0),
    t(Id::BolEos,     Bol,     Yield(T::Eos),    0b_1_0),
    t(Id::Cr,         AfterCr, Continue,         0b_1_0),
    t(Id::CrEos,      AfterCr, Yield(T::Eos),    0b_1_0),
    t(Id::CrLf,       Bol,     Continue,         0b_0_0),
// Comments                                         │ │
    t(Id::Comment,    Comment, Continue,         0b_0_0),
    t(Id::CommentEos, Comment, Yield(T::Eos),    0b_0_0),
// Tokens                                           │ │
    t(Id::ParenL,     Normal,  Yield(T::ParenL), 0b_0_1),
    t(Id::ParenR,     Normal,  Yield(T::ParenR), 0b_0_1),
// Termination                                      │ │
    t(Id::Error,      Normal,  Fail,             0b_0_0),
    t(Id::End,        Normal,  Succeed,          0b_0_0),
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
    YieldParam,

    /// Yield a character literal.
    YieldChar,

    /// Yield a token.
    Yield(Token),

    // === Terminators ===

    /// Terminate unsuccessfully.
    Fail,

    /// Terminate successfully.
    Succeed,
}

// ---------------------------------------------------------------------------- 

/// Lexer states.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    /// Normal state.  Any token is possible.
    Normal,

    /// At the begining of a line.  Any token is possible.
    Bol,

    /// After a carriage return (0x0D).
    AfterCr,

    /// In a comment.
    Comment,
}

impl State {
    /// Count of lexer states.
    const COUNT: usize = Self::Comment as usize + 1;
}

// ---------------------------------------------------------------------------- 

/// Lexical analyzer.  Reads input and yields a stream of lexical tokens.
#[derive(Debug)]
pub struct Lexer<'a> {
    input: Reader<'a>,
    state: State,
    line:  usize,
    len:   usize,
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
                Continue     => continue,

                // Sublexers
                ScanBin      => { let (_, _) = scan_int(&mut self.input, Base::Bin); },
                ScanOct      => { let (_, _) = scan_int(&mut self.input, Base::Oct); },
                ScanDec      => { let (_, _) = scan_int(&mut self.input, Base::Dec); },
                ScanHex      => { let (_, _) = scan_int(&mut self.input, Base::Hex); },
                ScanStr      => panic!(), // self.scan_str(),

                // Identifiers & Literals
                YieldIdent   => break Token::Ident,
                YieldLabel   => break Token::Label,
                YieldParam   => break Token::Param,
                YieldChar    => break Token::Char,

                // Simple Tokens
                Yield(token) => break token,

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
}

