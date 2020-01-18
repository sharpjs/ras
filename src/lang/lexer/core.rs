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
use super::super::token::Token::{self, self as T};

// ----------------------------------------------------------------------------

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
    const __: Char = Other;
[
//  xx0     xx1     xx2     xx3     xx4     xx5     xx6     xx7     CHARS
    __,     __,     __,     __,     __,     __,     __,     __,     // ........
    __,     Space,  Lf,     __,     __,     Cr,     __,     __,     // .tn..r..
    __,     __,     __,     __,     __,     __,     __,     __,     // ........
    __,     __,     __,     __,     __,     __,     __,     __,     // ........
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

/// Lexer transition.
#[derive(Clone, Copy, Debug)]
struct Transition {
    state:  State,
    action: Action,
    flags:  u8, // 0b000000LT
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
    use Action::*;
    use State::*;
[
//                                                                             Token┐
//                                                                         Newline┐ │
/* Normal     */ Transition { state: Normal,  action: Continue,         flags: 0b_0_0 },
/* Bol        */ Transition { state: Bol,     action: Continue,         flags: 0b_1_0 },
/* BolEos     */ Transition { state: Bol,     action: Yield(T::Eos),    flags: 0b_1_0 },
/* Cr         */ Transition { state: AfterCr, action: Continue,         flags: 0b_1_0 },
/* CrEos      */ Transition { state: AfterCr, action: Yield(T::Eos),    flags: 0b_1_0 },
/* CrLf       */ Transition { state: Bol,     action: Continue,         flags: 0b_0_0 },
/* Comment    */ Transition { state: Comment, action: Continue,         flags: 0b_0_0 },
/* CommentEos */ Transition { state: Comment, action: Yield(T::Eos),    flags: 0b_0_0 },
/* ParenL     */ Transition { state: Normal,  action: Yield(T::ParenL), flags: 0b_0_1 },
/* ParenR     */ Transition { state: Normal,  action: Yield(T::ParenR), flags: 0b_0_1 },
/* Error      */ Transition { state: Normal,  action: Fail,             flags: 0b_0_0 },
/* End        */ Transition { state: Normal,  action: Succeed,          flags: 0b_0_0 },
]};

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
/* LetB  */ Error,    Error,    Error,    Comment,
/* LetD  */ Error,    Error,    Error,    Comment,
/* LetO  */ Error,    Error,    Error,    Comment,
/* LetX  */ Error,    Error,    Error,    Comment,
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
