// This file is part of ras, an assembler.
// Copyright (C) 2019 Jeffrey Sharp
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

// NOTES:
//
// - The lexer implementation is inspired by the article "Some Strategies For
//   Fast Lexical Analysis when Parsing Programming Languages" by Sean Barrett.
//   http://nothings.org/computer/lexing.html
//
// - The term "logical character" in this file is preferred over the probably
//   more-correct term "character equivalence class".

use std::borrow::Cow;
use std::fmt::{Debug, Formatter, Result};
use std::marker::PhantomData;
use std::slice;

use crate::lang::token::Token;
use crate::util::ConstDefault;

use Action::*;
use State::*;

// ---------------------------------------------------------------------------- 

// Just a helper to define Char variants
const fn char(n: u16) -> u16 {
    n * State::COUNT as u16
}

/// Logical characters recognized by the main lexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
enum Char {
    // Variants are in order roughly by descending frequency, except that
    // groups of related variants are kept contiguous.
    //
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
    const COUNT: usize = Char::Other as usize / State::COUNT + 1;
}

impl ConstDefault for Char {
    /// Default `Char` logical character.
    /// A [`Reader`] returns this value at the end of input.
    const DEFAULT: Self = Char::Eof;
}

/// Mapping of UTF-8 bytes to `Char` logical characters.
static CHARS: [Char; 256] = { use Char::*; [
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

//  UTF-8 multibyte sequences
//  0 (8)   1 (9)   2 (A)   3 (B)   4 (C)   5 (D)   6 (E)   7 (F)   RANGE
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 80-87
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 88-8F
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 90-97
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // 98-9F
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // A0-A7
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // A8-AF
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // B0-B7
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // B8-BF
    Other,  Other,  Id,     Id,     Id,     Id,     Id,     Id,     // C0-C7
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // C8-CF
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // D0-D7
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // D8-DF
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // E0-E7
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // E8-EF
    Id,     Id,     Id,     Id,     Id,     Id,     Id,     Id,     // F0-F7
    Id,     Id,     Id,     Id,     Id,     Id,     Other,  Other,  // F8-FF
]};

// ----------------------------------------------------------------------------

/// Logical characters recognized by the numeric literal sublexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
enum NumChar {
    // digits
    Bin   = char( 0), // 0-1  binary digit
    Oct   = char( 1), // 2-7  octal digit
    Dec   = char( 2), // 8-9  decimal digit
    HexU  = char( 3), // A-F  hex digit, uppercase
    HexL  = char( 4), // a-f  hex digit, lowercase
    // punctuation
    Sep   = char( 5), // _    separator
    Dot   = char( 6), // .    radix point
    Exp   = char( 7), // Pp   exponent prefix
    Pos   = char( 8), // +    positive sign
    Neg   = char( 9), // -    negative sign
    // rare
    Eof   = char(10), // end of file
    Other = char(11), // everything else
}

impl NumChar {
    /// Count of `NumChar` logical characters.
    const COUNT: usize = NumChar::Other as usize / State::COUNT + 1;
}

impl ConstDefault for NumChar {
    /// Default `NumChar` logical character.
    /// A [`Reader`] returns this value at the end of input.
    const DEFAULT: Self = NumChar::Eof;
}

/// Mapping of UTF-8 bytes to `NumChar` logical characters.
static NUM_CHARS: [NumChar; 256] = { use NumChar::*; [
//  7-bit ASCII characters
//  x0      x1      x2      x3      x4      x5      x6      x7      CHARS
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // ........
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // .tn..r..
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // ........
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // ........
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  //  !"#$%&'
    Other,  Other,  Other,  Pos,    Other,  Neg,    Dot,    Other,  // ()*+,-./
    Bin,    Bin,    Oct,    Oct,    Oct,    Oct,    Oct,    Oct,    // 01234567
    Dec,    Dec,    Other,  Other,  Other,  Other,  Other,  Other,  // 89:;<=>?
    Other,  HexU,   HexU,   HexU,   HexU,   HexU,   HexU,   Other,  // @ABCDEFG
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // HIJKLMNO
    Exp,    Other,  Other,  Other,  Other,  Other,  Other,  Other,  // PQRSTUVW
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Sep,    // XYZ[\]^_
    Other,  HexL,   HexL,   HexL,   HexL,   HexL,   HexL,   Other,  // `abcdefg
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // hijklmno
    Exp,    Other,  Other,  Other,  Other,  Other,  Other,  Other,  // pqrstuvw
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // xyz{|}~. <- DEL

//  UTF-8 multibyte sequences
//  0 (8)   1 (9)   2 (A)   3 (B)   4 (C)   5 (D)   6 (E)   7 (F)   RANGE
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // 80-87
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // 88-8F
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // 90-97
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // 98-9F
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // A0-A7
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // A8-AF
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // B0-B7
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // B8-BF
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // C0-C7
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // C8-CF
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // D0-D7
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // D8-DF
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // E0-E7
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // E8-EF
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // F0-F7
    Other,  Other,  Other,  Other,  Other,  Other,  Other,  Other,  // F8-FF
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

    /// After a CR.
    AfterCr,

    /// In a comment.
    Comment,
}

impl State {
    /// Count of lexer states.
    const COUNT: usize = Comment as usize + 1;
}

// ----------------------------------------------------------------------------

/// Lexer actions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
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

    // === Identifiers & Literals ===

    /// Yield an identifier.
    YieldIdent,

    /// Yield a label.
    YieldLabel,

    /// Yield a macro parameter.
    YieldParam,

    /// Yield a character literal.
    YieldChar,

    // === Operators ===
    
    /// Yield a `!` - logical NOT operator, side-effect indicator.
    YieldLogNot,
    
    /// Yield a `~` - bitwise NOT operator.
    YieldBitNot,

    /// Yield a `*` - signed multiplication operator.
    YieldMul,

    /// Yield a `/` - signed division operator.
    YieldDiv,

    /// Yield a `%` - signed modulo operator.
    YieldMod,

    /// Yield a `+*` - unsigned multiplication operator.
    YieldUMul,

    /// Yield a `+/` - unsigned division operator.
    YieldUDiv,

    /// Yield a `+%` - unsigned modulo operator.
    YieldUMod,

    /// Yield a `+` - addition operator, increment indicator.
    YieldAdd,

    /// Yield a `-` - subtraction operator, negation operator, decrement indicator.
    YieldSub,

    /// Yield a `<<` - left shift operator.
    YieldShl,

    /// Yield a `>>` - signed right shift operator.
    YieldShr,

    /// Yield a `+>>` - unsigned right shift operator.
    YieldUShr,

    /// Yield a `&` - bitwise AND operator.
    YieldBitAnd,

    /// Yield a `^` - bitwise XOR operator.
    YieldBitXor,

    /// Yield a `|` - bitwise OR operator.
    YieldBitOr,

    /// Yield a `==` - equal-to operator.
    YieldEq,

    /// Yield a `!=` - not-equal-to operator.
    YieldNotEq,

    /// Yield a `<` - signed less-than operator.
    YieldLess,

    /// Yield a `>` - signed greater-than operator.
    YieldMore,

    /// Yield a `<=` - signed less-than-or-equal-to operator.
    YieldLessEq,

    /// Yield a `>=` - signed greater-than-or-equal-to operator.
    YieldMoreEq,

    /// Yield a `+<` - unsigned less-than operator.
    YieldULess,

    /// Yield a `+>` - unsigned greater-than operator.
    YieldUMore,

    /// Yield a `+<=` - unsigned less-than-or-equal-to operator.
    YieldULessEq,

    /// Yield a `+>=` - unsigned greater-than-or-equal-to operator.
    YieldUMoreEq,

    /// Yield a `?` - not-known indicator.
    YieldUnknown,

    /// Yield a `&&` - logical AND operator.
    YieldLogAnd,

    /// Yield a `||` - logical OR operator.
    YieldLogOr,

    /// Yield a `=` - assignment operator.
    YieldAssign,

    /// Yield a `*=` - signed multiplication-assignment operator.
    YieldMulAssign,

    /// Yield a `/=` - signed division-assignment operator.
    YieldDivAssign,

    /// Yield a `%=` - signed modulo-assignment operator.
    YieldModAssign,

    /// Yield a `+*=` - unsigned multiplication-assignment operator.
    YieldUMulAssign,

    /// Yield a `+/=` - unsigned division-assignment operator.
    YieldUDivAssign,

    /// Yield a `+/=` - unsigned modulo-assignment operator.
    YieldUModAssign,

    /// Yield a `+=` - addition-assigment operator.
    YieldAddAssign,

    /// Yield a `-=` - subtraction-assignment operator.
    YieldSubAssign,

    /// Yield a `<<=` - left-shift-assignment operator.
    YieldShlAssign,

    /// Yield a `>>=` - signed right-shift-assignment operator.
    YieldShrAssign,

    /// Yield a `+>>=` - unsigned right-shift-assignment operator.
    YieldUShrAssign,

    /// Yield a `&=` - bitwise AND-assignment operator.
    YieldBitAndAssign,

    /// Yield a `^=` - bitwise XOR-assignment operator.
    YieldBitXorAssign,

    /// Yield a `|=` - bitwise OR-assignment operator.
    YieldBitOrAssign,

    /// Yield a `&&=` - logical AND-assignment operator.
    YieldLogAndAssign,

    /// Yield a `||=` - logical OR-assignment operator.
    YieldLogOrAssign,

    // === Punctuation ===

    /// Yield a `{` - left curly brace.
    YieldBraceL,

    /// Yield a `}` - right curly brace.
    YieldBraceR,

    /// Yield a `(` - left parenthesis.
    YieldParenL,

    /// Yield a `)` - right parenthesis.
    YieldParenR,

    /// Yield a `[` - left square bracket.
    YieldBracketL,

    /// Yield a `]` - right square bracket.
    YieldBracketR,

    /// Yield a `:` - item joiner.
    YieldColon,

    /// Yield a `,` - item separator.
    YieldComma,

    // === Terminators ===

    /// Yield an `Eos` token.
    YieldEos,

    /// Terminate unsuccessfully.
    Fail,

    /// Terminate successfully.
    Succeed,
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
    const COUNT: usize = TransitionId::End as usize + 1;
}

/// Lexer transition.
#[derive(Clone, Copy, Debug)]
struct Transition {
    state:  State,
    action: Action,
    flags:  u16,
    //      - line   increment
    //      - length increment
}

/// Lexer transitions in order by transition ID.
static TRANSITION_LUT: [Transition; TransitionId::COUNT] = [
/* Normal     */ Transition { state: Normal,  action: Continue,    flags: 0 },
/* Bol        */ Transition { state: Bol,     action: Continue,    flags: 1 },
/* BolEos     */ Transition { state: Bol,     action: YieldEos,    flags: 1 },
/* Cr         */ Transition { state: AfterCr, action: Continue,    flags: 1 },
/* CrEos      */ Transition { state: AfterCr, action: YieldEos,    flags: 0 },
/* CrLf       */ Transition { state: Bol,     action: Continue,    flags: 0 },
/* Comment    */ Transition { state: Comment, action: Continue,    flags: 0 },
/* CommentEos */ Transition { state: Comment, action: YieldEos,    flags: 0 },
/* ParenL     */ Transition { state: Normal,  action: YieldParenL, flags: 0 },
/* ParenR     */ Transition { state: Normal,  action: YieldParenR, flags: 0 },
/* Error      */ Transition { state: Normal,  action: Fail,        flags: 0 },
/* End        */ Transition { state: Normal,  action: Succeed,     flags: 0 },
];

/// Lexer state transition map.
static TRANSITION_MAP: [TransitionId; State::COUNT * Char::COUNT] = { use TransitionId::*; [
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

/// Input reader specialized for lexical analysis.  A `Reader` takes a slice of
/// bytes as input and provides a simple rewindable cursor over a sequence of
/// logical characters (effectively, character equivalence classes).
///
#[derive(Clone, Copy)]
struct Reader<'a> {
    ptr: *const u8,
    beg: *const u8,
    end: *const u8,
    _lt: PhantomData<&'a ()>,
}

impl<'a> Reader<'a> {
    // Safety: This is a specialization of std::slice::Iter.  The unsafe blocks
    // here are equivalent to those in std::slice::Iter and thus have the same
    // effective safety.

    /// Creates a new [`Reader`] over the given slice of bytes.
    #[inline(always)]
    pub fn new(bytes: &'a [u8]) -> Self {
        let beg = bytes.as_ptr();
        let end = unsafe { beg.add(bytes.len()) };

        Self { ptr: beg, beg, end, _lt: PhantomData }
    }

    /// Returns the position of the next byte to be read.
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.ptr as usize - self.beg as usize
    }

    /// Reads the next byte, advances the reader, and returns the corresponding
    /// logical character from the given character set `map`.
    ///
    /// If the reader is positioned at the end of input, this method returns
    /// [`C::DEFAULT`], and the reader's position remains unchanged.
    #[inline(always)]
    pub fn next<C>(&mut self, map: &[C; 256]) -> C where C: ConstDefault {
        let p = self.ptr;
        if p == self.end {
            C::DEFAULT
        } else {
            unsafe {
                self.ptr = p.offset(1);
                map[*p as usize]
            }
        }
    }

    /// Rewinds the reader by one byte.
    ///
    /// # Panics
    ///
    /// Panics if the reader is positioned at the beginning of input.
    ///
    #[inline(always)]
    pub fn rewind(&mut self) {
        let p = self.ptr;
        if p == self.beg {
            panic!("Attempt to rewind past beginning of input.")
        }
        self.ptr = unsafe { p.offset(-1) };
    }

    /// Returns a slice of the bytes remaining to be read.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let len = self.end as usize - self.ptr as usize;
            slice::from_raw_parts(self.ptr, len)
        }
    }
}

impl<'a> Debug for Reader<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Reader {:X?}", self.as_slice())
    }
}

// ----------------------------------------------------------------------------

/// A lexical analyzer.  Reads input and yields a stream of lexical tokens.
#[derive(Debug)]
pub struct Lexer<'a> {
    input: Reader<'a>,
    state: State,
}

impl<'a> Lexer<'a> {
    /// Creates a lexical analyzer that takes as input the contents of the
    /// given slice of bytes.
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input: Reader::new(input),
            state: Bol,
        }
    }

    /// Returns the source file path of the current token.
    #[inline]
    pub fn path(&self) -> &'a str {
        panic!()
    }

    /// Returns the source line number (1-indexed) of the current token.
    #[inline]
    pub fn line(&self) -> usize {
        panic!()
    }

    /// Returns the numeric value of the current token.
    #[inline]
    pub fn num(&mut self) -> u64 /*?*/ {
        panic!()
    }

    /// Returns the source text of the current token.
    #[inline]
    pub fn text(&self) -> &'a [u8] {
        panic!()
    }

    /// Takes the string value of the current token out of the lexer.
    /// Can be invoked only once per string token.
    #[inline]
    pub fn take_str(&mut self) -> Cow<'a, String> {
        panic!()
    }

    /// Advances to the next token and returns its type.
    pub fn next(&mut self) -> Token {
        // Restore saved state and prepare for loop
        let mut state = self.state;
        let mut action;
        //let mut length = 0;

        // Discover next token
        loop {
            let next = self.input.next(&CHARS);
            let next = TRANSITION_MAP[state as usize + next as usize];
            let next = TRANSITION_LUT[next  as usize];

            state   = next.state;
            action  = next.action;
            //length += next.flags & 1u16;

            if action != Continue { break }
        }

        // Save state for subsequent invocation
        self.state = state;

        // Return token
        match action {
            Continue          => unreachable!(),

            // Sublexers
            ScanBin           => self.scan_bin(),
            ScanOct           => self.scan_oct(),
            ScanDec           => self.scan_dec(),
            ScanHex           => self.scan_hex(),
            ScanStr           => self.scan_str(),

            // Identifiers & Literals
            YieldIdent        => Token::Ident,
            YieldLabel        => Token::Label,
            YieldParam        => Token::Param,
            YieldChar         => Token::Char,

            // Operators
            YieldLogNot       => Token::LogNot,
            YieldBitNot       => Token::BitNot,
            YieldMul          => Token::Mul,
            YieldDiv          => Token::Div,
            YieldMod          => Token::Mod,
            YieldUMul         => Token::UMul,
            YieldUDiv         => Token::UDiv,
            YieldUMod         => Token::UMod,
            YieldAdd          => Token::Add,
            YieldSub          => Token::Sub,
            YieldShl          => Token::Shl,
            YieldShr          => Token::Shr,
            YieldUShr         => Token::UShr,
            YieldBitAnd       => Token::BitAnd,
            YieldBitXor       => Token::BitXor,
            YieldBitOr        => Token::BitOr,
            YieldEq           => Token::Eq,
            YieldNotEq        => Token::NotEq,
            YieldLess         => Token::Less,
            YieldMore         => Token::More,
            YieldLessEq       => Token::LessEq,
            YieldMoreEq       => Token::MoreEq,
            YieldULess        => Token::ULess,
            YieldUMore        => Token::UMore,
            YieldULessEq      => Token::ULessEq,
            YieldUMoreEq      => Token::UMoreEq,
            YieldUnknown      => Token::Unknown,
            YieldLogAnd       => Token::LogAnd,
            YieldLogOr        => Token::LogOr,
            YieldAssign       => Token::Assign,
            YieldMulAssign    => Token::MulAssign,
            YieldDivAssign    => Token::DivAssign,
            YieldModAssign    => Token::ModAssign,
            YieldUMulAssign   => Token::UMulAssign,
            YieldUDivAssign   => Token::UDivAssign,
            YieldUModAssign   => Token::UModAssign,
            YieldAddAssign    => Token::AddAssign,
            YieldSubAssign    => Token::SubAssign,
            YieldShlAssign    => Token::ShlAssign,
            YieldShrAssign    => Token::ShrAssign,
            YieldUShrAssign   => Token::UShrAssign,
            YieldBitAndAssign => Token::BitAndAssign,
            YieldBitXorAssign => Token::BitXorAssign,
            YieldBitOrAssign  => Token::BitOrAssign,
            YieldLogAndAssign => Token::LogAndAssign,
            YieldLogOrAssign  => Token::LogOrAssign,

            // Punctuation
            YieldBraceL       => Token::BraceL,
            YieldBraceR       => Token::BraceR,
            YieldParenL       => Token::ParenL,
            YieldParenR       => Token::ParenR,
            YieldBracketL     => Token::BracketL,
            YieldBracketR     => Token::BracketR,
            YieldColon        => Token::Colon,
            YieldComma        => Token::Comma,

            // Terminators
            YieldEos          => Token::Eos,
            Succeed           => Token::Eof,
            Fail              => Token::Error,
        }
    }

    fn scan_bin(&mut self) -> Token {
        // TODO: Implement
        Token::Int // or Token::Float
    }

    fn scan_oct(&mut self) -> Token {
        // TODO: Implement
        Token::Int // or Token::Float
    }

    fn scan_dec(&mut self) -> Token {
        #[cfg(wip)]
        {
            let mut val     = 0u64; // significand
            let mut exp     = 0i16; // exponent
            let mut exp_inc = 0i16; // exponent increment

            loop {
                let next = self.input.next(&NUM_CHARS);

                // dig, sig_mask, dig_mask, frac_marker, 
                let mask = (next.flags & IS_DIGIT) as i64;
                let mask = (val_mask << 63 >> 63)  as u64;

                val = val * 10 + dig as u64 & mask;
                exp = exp + exp_inc         & mask;

                exp_inc |= -next.frac_marker;

                match action {
                    Continue => {},
                }
            }
        }
        Token::Int // or Token::Float
    }

    fn scan_hex(&mut self) -> Token {
        // TODO: Implement
        Token::Int // or Token::Float
    }

    fn scan_str(&mut self) -> Token {
        // TODO: Implement
        Token::Str
    }
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::Char::*;

    #[test]
    fn reader_empty() {
        let mut reader = Reader::new(b"");

        assert_eq!( reader.position(),   0   ); 

        assert_eq!( reader.next(&CHARS), Eof );
        assert_eq!( reader.position(),   0   ); 
    }

    #[test]
    fn reader_next() {
        let mut reader = Reader::new(b"X+1");

        assert_eq!( reader.position(),   0     ); 

        assert_eq!( reader.next(&CHARS), LetX  );
        assert_eq!( reader.position(),   1     ); 

        assert_eq!( reader.next(&CHARS), Plus  );
        assert_eq!( reader.position(),   2     ); 

        assert_eq!( reader.next(&CHARS), Digit );
        assert_eq!( reader.position(),   3     ); 

        reader.rewind();
        assert_eq!( reader.position(),   2     ); 

        assert_eq!( reader.next(&CHARS), Digit );
        assert_eq!( reader.position(),   3     ); 

        assert_eq!( reader.next(&CHARS), Eof   );
        assert_eq!( reader.position(),   3     ); 
    }

    #[test]
    fn reader_debug_empty() {
        let reader = Reader::new(b"");

        assert_eq!( format!("{:?}", reader), "Reader []" );
    }

    #[test]
    fn reader_debug_not_empty() {
        let reader = Reader::new(b"X+1");

        assert_eq!( format!("{:?}", reader), "Reader [58, 2B, 31]" );
    }

    #[test]
    fn lexer_empty() {
        let mut lexer = Lexer::new(b"");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_unrecognized() {
        let mut lexer = Lexer::new(b"`");

        assert_eq!( lexer.next(), Token::Error );
    }

    #[test]
    fn lexer_space() {
        let mut lexer = Lexer::new(b" \t \t");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_comment() {
        let mut lexer = Lexer::new(b"# this is a comment");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_cr() {
        let mut lexer = Lexer::new(b"\r\r # hello");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_lf() {
        let mut lexer = Lexer::new(b"\n\n # hello");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_crlf() {
        let mut lexer = Lexer::new(b"\r\n\r\n # hello");

        assert_eq!( lexer.next(), Token::Eof );
    }

    #[test]
    fn lexer_parens() {
        let mut lexer = Lexer::new(b"()#c\n\n");

        assert_eq!( lexer.next(), Token::ParenL );
        assert_eq!( lexer.next(), Token::ParenR );
        assert_eq!( lexer.next(), Token::Eos    );
        assert_eq!( lexer.next(), Token::Eof    );
    }
}

