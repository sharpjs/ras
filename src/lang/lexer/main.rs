// This file is part of ras, an assembler.
// Copyright 2022 Jeffrey Sharp
//
// SPDX-License-Identifier: GPL-3.0-or-later
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

//! Main lexer.

use crate::lang::input::LogicalChar;
use crate::num::Base::*;
use super::*;

// ----------------------------------------------------------------------------

/// Logical characters recognized by the main lexer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
enum Char {
    // space, line endings
    Space   = char( 0), // \s \t \v \f
    Cr      = char( 1), // \r
    Lf      = char( 2), // \n
    // identifiers, numbers
    Ident   = char( 3), // A-Z a-z . and all code points above U+007F
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
    Caret   = char(19), // ^
    Pipe    = char(20), // |
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
    At      = char(32), // @
    BSlash  = char(33), // \
    // rare
    Eof     = char(34), // end of file
    Other   = char(35), // everything else // <- COUNT references this
}

// Helper to define Char variants.
const fn char(n: u16) -> u16 {
    n * State::COUNT as u16
}

impl Char {
    /// Count of logical characters.
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
//  x0       x1       x2       x3       x4       x5       x6       x7
//  x8       x9       xA       xB       xC       xD       xE       xF
    __,      __,      __,      __,      __,      __,      __,      __,      // 0x │········│
    __,      Space,   Lf,      Space,   Space,   Cr,      __,      __,      // 0x │·tnvfr··│
    __,      __,      __,      __,      __,      __,      __,      __,      // 1x │········│
    __,      __,      __,      __,      __,      __,      __,      __,      // 1x │········│
    Space,   Bang,    DQuote,  Hash,    Dollar,  Percent, Amp,     SQuote,  // 2x │ !"#$%&'│
    LParen,  RParen,  Star,    Plus,    Comma,   Minus,   Ident,   Slash,   // 2x │()*+,-./│
    Digit,   Digit,   Digit,   Digit,   Digit,   Digit,   Digit,   Digit,   // 3x │01234567│
    Digit,   Digit,   Colon,   Semi,    Lt,      Equal,   Gt,      Quest,   // 3x │89:;<=>?│
    At,      Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 4x │@ABCDEFG│
    Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 4x │HIJKLMNO│
    Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 5x │PQRSTUVW│
    Ident,   Ident,   Ident,   LSquare, BSlash,  RSquare, Caret,   Ident,   // 5x │XYZ[\]^_│
    __,      Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 6x │`abcdefg│
    Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 6x │hijklmno│
    Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   Ident,   // 7x │pqrstuvw│
    Ident,   Ident,   Ident,   LCurly,  Pipe,    RCurly,  Tilde,   __,      // 7x │xyz{|}~░│
]};

// ----------------------------------------------------------------------------

/// Main lexer states.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum State {
    /// Normal state.  Any token is possible.
    Normal,

    /// In a comment.
    Comment,

    /// After a `:`.
    Colon,

    /// After `=`.
    Equal,

    /// After `+`.
    Plus,

    /// After `-`.
    Minus,

    /// After `&`.
    Amp,

    /// After `&&`.
    AmpAmp,

    /// After `^`.
    Caret,

    /// After `^^`.
    CaretCaret,

    /// After `|`.
    Pipe,

    /// After `||`.
    PipePipe,

    /// After `<`.
    Lt,

    /// After `<<`.
    LtLt,

    /// After `>`.
    Gt,

    /// After `>>`.
    GtGt,

    /// After `!`
    Bang,

    /// After `*`
    Star,

    /// After `/`
    Slash,

    /// After `%`
    Percent,

    /// After `\`
    BSlash, // <- COUNT references this
}

impl State {
    /// Count of main lexer states.
    const COUNT: usize = Self::BSlash as usize + 1;
}

// ----------------------------------------------------------------------------

// Main lexer transitions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum Transition {
    /// Consume the current input byte and continue scanning in `Normal` state.
    Normal,

    /// Handle a CR or CR+LF newline.
    /// - Consume the current input byte.
    /// - Consume the next input byte if it is a line feed.
    /// - Yield an `Eos` token.
    /// - Increment the line number for subsequent tokens.
    CrEos,

    /// Handle a LF newline.
    /// - Consume the current input byte.
    /// - Yield an `Eos` token.
    /// - Increment the line number for subsequent tokens.
    LfEos,

    /// Consume the current input byte and continue scanning in `Comment` state.
    Comment,

    /// Scan an identifier or a label.
    Ident,

    /// Scan a macro parameter.
    Param,

    /// Scan a decimal numeric literal.
    IntDec,

    /// Scan a string literal.
    Str,

    /// Scan a character literal.
    Char,

    /// Consume the current input byte and yield a `BitNot` token.
    BitNot,

    /// Consume the current input byte and yield an `Unknown` token.
    Unknown,

    /// Consume the current input byte and yield a `LParen` token.
    LParen,

    /// Consume the current input byte and yield a `RParen` token.
    RParen,

    /// Consume the current input byte and yield a `LSquare` token.
    LSquare,

    /// Consume the current input byte and yield a `RSquare` token.
    RSquare,

    /// Consume the current input byte and yield a `LCurly` token.
    LCurly,

    /// Consume the current input byte and yield a `RCurly` token.
    RCurly,

    /// Consume the current input byte and yield an `Eos` token.
    Eos,

    /// Consume the current input byte and yield a `Comma` token.
    Comma,

    /// Consume the current input byte and yield a `Alias` token.
    Alias,

    // = ...

    /// Consume the current input byte and continue scanning in `Colon` state.
    Colon_,

    /// Yield a `Join` token.
    Join,

    /// Consume the current input byte and yield an `Weak` token.
    Weak,

    /// Consume the current input byte and yield an `Public` token.
    Public,

    // = ...

    /// Consume the current input byte and continue scanning in `Equal` state.
    Equal_,

    /// Yield an `Assign` token.
    Assign,

    /// Consume the current input byte and yield an `Eq` token.
    Eq,

    // + ...

    /// Consume the current input byte and continue scanning in `Plus` state.
    Plus_,

    /// Yield an `Add` token.
    Add,

    /// Consume the current input byte and yield an `AddAssign` token.
    AddAssign,

    /// Consume the current input byte and yield an `Inc` token.
    Inc,

    /// Consume the current input byte and yield a `Signed` token.
    Signed,

    // - ...

    /// Consume the current input byte and continue scanning in `Minus` state.
    Minus_,

    /// Yield a `Sub` token.
    Sub,

    /// Consume the current input byte and yield a `SubAssign` token.
    SubAssign,

    /// Consume the current input byte and yield a `Dec` token.
    Dec,

    // & ...

    /// Consume the current input byte and continue scanning in `Amp` state.
    Amp_,

    /// Yield a `BitAnd` token.
    BitAnd,

    /// Consume the current input byte and yield a `BitAndAssign` token.
    BitAndAssign,

    // && ...

    /// Consume the current input byte and continue scanning in `AmpAmp` state.
    AmpAmp_,

    /// Yield a `LogAnd` token.
    LogAnd,

    /// Consume the current input byte and yield a `LogAndAssign` token.
    LogAndAssign,

    // ^ ...

    /// Consume the current input byte and continue scanning in `Caret` state.
    Caret_,

    /// Yield a `BitXor` token.
    BitXor,

    /// Consume the current input byte and yield a `BitXorAssign` token.
    BitXorAssign,

    // ^^ ...

    /// Consume the current input byte and continue scanning in `CaretCaret` state.
    CaretCaret_,

    /// Yield a `LogXor` token.
    LogXor,

    /// Consume the current input byte and yield a `LogXorAssign` token.
    LogXorAssign,

    // | ...

    /// Consume the current input byte and continue scanning in `Pipe` state.
    Pipe_,

    /// Yield a `BitOr` token.
    BitOr,

    /// Consume the current input byte and yield a `BitOrAssign` token.
    BitOrAssign,

    // || ...

    /// Consume the current input byte and continue scanning in `PipePipe` state.
    PipePipe_,

    /// Yield a `LogOr` token.
    LogOr,

    /// Consume the current input byte and yield a `LogOrAssign` token.
    LogOrAssign,

    // < ...

    /// Consume the current input byte and continue scanning in `Lt` state.
    Lt_,

    /// Yield a `Less` token.
    Less,

    /// Consume the current input byte and yield a `LessEq` token.
    LessEq,

    // << ...

    /// Consume the current input byte and continue scanning in `LtLt` state.
    LtLt_,

    /// Yield a `Shl` token.
    Shl,

    /// Consume the current input byte and yield a `ShlAssign` token.
    ShlAssign,

    // > ...

    /// Consume the current input byte and continue scanning in `Gt` state.
    Gt_,

    /// Yield a `More` token.
    More,

    /// Consume the current input byte and yield a `MoreEq` token.
    MoreEq,

    // >> ...

    /// Consume the current input byte and continue scanning in `GtGt` state.
    GtGt_,

    /// Yield a `Shr` token.
    Shr,

    /// Consume the current input byte and yield a `ShrAssign` token.
    ShrAssign,

    // >> ...

    /// Consume the current input byte and continue scanning in `Bang` state.
    Bang_,

    /// Yield a `LogNot` token.
    LogNot,

    /// Consume the current input byte and yield a `NotEq` token.
    NotEq,

    // * ...

    /// Consume the current input byte and continue scanning in `Star` state.
    Star_,

    /// Yield a `Mul` token.
    Mul,

    /// Consume the current input byte and yield a `MulAssign` token.
    MulAssign,

    // / ...

    /// Consume the current input byte and continue scanning in `Slash` state.
    Slash_,

    /// Yield a `Div` token.
    Div,

    /// Consume the current input byte and yield a `DivAssign` token.
    DivAssign,

    // % ...

    /// Consume the current input byte and continue scanning in `Percent` state.
    Percent_,

    /// Yield a `Mod` token.
    Mod,

    /// Consume the current input byte and yield a `ModAssign` token.
    ModAssign,

    /// Consume the current input byte and yield an `Unsigned` token.
    Unsigned,

    // \ ...

    /// Consume the current input byte and continue scanning in `BSlash` state.
    BSlash_,

    /// Handle an escaped CR or CR+LF newline.
    /// - Consume the current input byte.
    /// - Consume the next input byte if it is a line feed.
    /// - Increment the line number for subsequent tokens.
    EscCr,

    /// Handle an escaped LF newline.
    /// - Consume the current input byte.
    /// - Increment the line number for subsequent tokens.
    EscLf,

    // Other

    /// Yield an `Eof` token.
    End,

    /// Handle an unexpected character.
    /// - Record a lexical error at the current input position.
    /// - Consume the current input byte.
    /// - Continue scanning in `Normal` state.
    Error,
}

impl Transition {
    /// Returns a tuple consisting of the action, token start flag, and token
    /// variant index for the transition.
    fn decode(self) -> (Action, u8, u8) {
        use Action::*;
        use State      as S;
        use Token      as T;
        use Transition as X;

        match self {
            //                                          Variant ─────╮
            //                                            Start ──╮  │
            //                     Action      Arguments          S  V
            // Whitespace        ---------------------------------------
            X::Normal         => ( Continue    (S::Normal),       0, 0 ),
            X::CrEos          => ( ScanCrLfEos ,                  1, 0 ),
            X::LfEos          => ( ScanLfEos   ,                  1, 0 ),
            X::Comment        => ( Continue    (S::Comment),      0, 0 ),
            // Numbers           ---------------------------------------
            X::Ident          => ( ScanIdent   ,                  1, 0 ),
            X::Param          => ( ScanParam   ,                  1, 1 ),
            X::IntDec         => ( ScanDec     ,                  1, 0 ),
            X::Str            => ( ScanStr     ,                  1, 0 ),
            X::Char           => ( ScanChar    ,                  1, 0 ),
            // Simple Tokens     ---------------------------------------
            X::BitNot         => ( Produce     (T::BitNot),       1, 0 ),
            X::Unknown        => ( Produce     (T::Unknown),      1, 0 ),
            X::LParen         => ( Produce     (T::LParen),       1, 0 ),
            X::RParen         => ( Produce     (T::RParen),       1, 0 ),
            X::LSquare        => ( Produce     (T::LSquare),      1, 0 ),
            X::RSquare        => ( Produce     (T::RSquare),      1, 0 ),
            X::LCurly         => ( Produce     (T::LCurly),       1, 0 ),
            X::RCurly         => ( Produce     (T::RCurly),       1, 0 ),
            X::Eos            => ( Produce     (T::Eos),          1, 0 ),
            X::Comma          => ( Produce     (T::Comma),        1, 0 ),
            X::Alias          => ( Produce     (T::Alias),        1, 0 ),
            // : ...             ---------------------------------------
            X::Colon_         => ( Continue    (S::Colon),        1, 0 ),
            X::Join           => ( Yield       (T::Colon),        0, 0 ),
            X::Weak           => ( Produce     (T::Weak),         0, 0 ),
            X::Public         => ( Produce     (T::Public),       0, 0 ),
            // = ...             ---------------------------------------
            X::Equal_         => ( Continue    (S::Equal),        1, 0 ),
            X::Assign         => ( Yield       (T::Assign),       0, 0 ),
            X::Eq             => ( Produce     (T::Eq),           0, 0 ),
            // + ...             ---------------------------------------
            X::Plus_          => ( Continue    (S::Plus),         1, 0 ),
            X::Add            => ( Yield       (T::Add),          0, 0 ),
            X::AddAssign      => ( Produce     (T::AddAssign),    0, 0 ),
            X::Inc            => ( Produce     (T::Inc),          0, 0 ),
            X::Signed         => ( Produce     (T::Signed),       0, 0 ),
            // - ...             ---------------------------------------
            X::Minus_         => ( Continue    (S::Minus),        1, 0 ),
            X::Sub            => ( Yield       (T::Sub),          0, 0 ),
            X::SubAssign      => ( Produce     (T::SubAssign),    0, 0 ),
            X::Dec            => ( Produce     (T::Dec),          0, 0 ),
            // & ...             ---------------------------------------
            X::Amp_           => ( Continue    (S::Amp),          1, 0 ),
            X::BitAnd         => ( Yield       (T::BitAnd),       0, 0 ),
            X::BitAndAssign   => ( Produce     (T::BitAndAssign), 0, 0 ),
            // && ...            ---------------------------------------
            X::AmpAmp_        => ( Continue    (S::AmpAmp),       0, 0 ),
            X::LogAnd         => ( Yield       (T::LogAnd),       0, 0 ),
            X::LogAndAssign   => ( Produce     (T::LogAndAssign), 0, 0 ),
            // ^ ...             ---------------------------------------
            X::Caret_         => ( Continue    (S::Caret),        1, 0 ),
            X::BitXor         => ( Yield       (T::BitXor),       0, 0 ),
            X::BitXorAssign   => ( Produce     (T::BitXorAssign), 0, 0 ),
            // ^^ ...            ---------------------------------------
            X::CaretCaret_    => ( Continue    (S::CaretCaret),   0, 0 ),
            X::LogXor         => ( Yield       (T::LogXor),       0, 0 ),
            X::LogXorAssign   => ( Produce     (T::LogXorAssign), 0, 0 ),
            // | ...             ---------------------------------------
            X::Pipe_          => ( Continue    (S::Pipe),         1, 0 ),
            X::BitOr          => ( Yield       (T::BitOr),        0, 0 ),
            X::BitOrAssign    => ( Produce     (T::BitOrAssign),  0, 0 ),
            // || ...            ---------------------------------------
            X::PipePipe_      => ( Continue    (S::PipePipe),     0, 0 ),
            X::LogOr          => ( Yield       (T::LogOr),        0, 0 ),
            X::LogOrAssign    => ( Produce     (T::LogOrAssign),  0, 0 ),
            // < ...             ---------------------------------------
            X::Lt_            => ( Continue    (S::Lt),           1, 0 ),
            X::Less           => ( Yield       (T::Less),         0, 0 ),
            X::LessEq         => ( Produce     (T::LessEq),       0, 0 ),
            // << ...            ---------------------------------------
            X::LtLt_          => ( Continue    (S::LtLt),         0, 0 ),
            X::Shl            => ( Yield       (T::Shl),          0, 0 ),
            X::ShlAssign      => ( Produce     (T::ShlAssign),    0, 0 ),
            // > ...             ---------------------------------------
            X::Gt_            => ( Continue    (S::Gt),           1, 0 ),
            X::More           => ( Yield       (T::More),         0, 0 ),
            X::MoreEq         => ( Produce     (T::MoreEq),       0, 0 ),
            // >> ...            ---------------------------------------
            X::GtGt_          => ( Continue    (S::GtGt),         0, 0 ),
            X::Shr            => ( Yield       (T::Shr),          0, 0 ),
            X::ShrAssign      => ( Produce     (T::ShrAssign),    0, 0 ),
            // ! ...             ---------------------------------------
            X::Bang_          => ( Continue    (S::Bang),         1, 0 ),
            X::LogNot         => ( Yield       (T::LogNot),       0, 0 ),
            X::NotEq          => ( Produce     (T::NotEq),        0, 0 ),
            // * ...             ---------------------------------------
            X::Star_          => ( Continue    (S::Star),         1, 0 ),
            X::Mul            => ( Yield       (T::Mul),          0, 0 ),
            X::MulAssign      => ( Produce     (T::MulAssign),    0, 0 ),
            // / ...             ---------------------------------------
            X::Slash_         => ( Continue    (S::Slash),        1, 0 ),
            X::Div            => ( Yield       (T::Div),          0, 0 ),
            X::DivAssign      => ( Produce     (T::DivAssign),    0, 0 ),
            // % ...             ---------------------------------------
            X::Percent_       => ( Continue    (S::Percent),      1, 0 ),
            X::Mod            => ( Yield       (T::Mod),          0, 0 ),
            X::ModAssign      => ( Produce     (T::ModAssign),    0, 0 ),
            X::Unsigned       => ( Produce     (T::Unsigned),     0, 0 ),
            // \ ...             ---------------------------------------
            X::BSlash_        => ( Continue    (S::BSlash),       0, 0 ),
            X::EscCr          => ( ScanCrLf    ,                  0, 0 ),
            X::EscLf          => ( ScanLf      ,                  0, 0 ),
            // Other             ---------------------------------------
            X::End            => ( Yield       (T::Eof),          1, 0 ),
            X::Error          => ( Error       ,                  0, 0 ),
        }
    }
}

// ----------------------------------------------------------------------------

/// Main lexer actions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    /// Consume the current input byte and continue scanning in a new state.
    Continue(State),

    // === Tokens ===

    /// Consume the current input byte and yield a token.
    Produce(Token),

    /// Yield a token.
    Yield(Token),

    // === Newlines ===

    /// Scan a CR or CR+LF end-of-statement.
    ScanCrLfEos,

    /// Scan an escaped CR or CR+LF.
    ScanCrLf,

    /// Scan a LF end-of-statement.
    ScanLfEos,

    /// Scan an escaped LF.
    ScanLf,

    // === Sublexers ===

    /// Scan a binary numeric literal.
    ScanBin,

    /// Scan an octal numeric literal.
    ScanOct,

    /// Scan a decimal numeric literal.
    ScanDec,

    /// Scan a hexadecimal numeric literal.
    ScanHex,

    /// Scan a string literal.
    ScanStr,

    /// Scan a character literal.
    ScanChar,

    /// Scan an identifier.
    ScanIdent,

    /// Scan a macro parameter.
    ScanParam,

    // === Miscellaneous ===

    /// Handle an unexpected character.
    /// - Record a lexical error at the current input position.
    /// - Consume the current input byte.
    /// - Continue scanning in `Normal` state.
    Error,
}

// ----------------------------------------------------------------------------

/// Main lexer state transition map.
static TRANSITION_MAP: [Transition; State::COUNT * Char::COUNT] = {
    use Transition::*;
    const __: Transition = Error;
[
//          Normal    Comment   Colon    Equal    Plus       Minus      Amp           AmpAmp        Caret         CaretCaret    Pipe         PipePipe     Lt      LtLt       Gt      GtGt       Bang    Star       Slash      Percent    BSlash
//          ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
/* Space */ Normal,   Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   Cr  */ CrEos,    CrEos,    Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       EscCr,
/*   Lf  */ LfEos,    LfEos,    Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       EscLf,

/* Ident */ Ident,    Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/* Digit */ IntDec,   Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,

/*   (   */ LParen,   Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   )   */ RParen,   Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   [   */ LSquare,  Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   ]   */ RSquare,  Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   {   */ LCurly,   Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   }   */ RCurly,   Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   "   */ Str,      Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   '   */ Char,     Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,

/*   ,   */ Comma,    Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   #   */ Comment,  Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   =   */ Equal_,   Comment,  Join,    Eq,      AddAssign, SubAssign, BitAndAssign, LogAndAssign, BitXorAssign, LogXorAssign, BitOrAssign, LogOrAssign, LessEq, ShlAssign, MoreEq, ShrAssign, NotEq,  MulAssign, DivAssign, ModAssign, Error,
/*   +   */ Plus_,    Comment,  Join,    Assign,  Inc,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   -   */ Minus_,   Comment,  Join,    Assign,  Add,       Dec,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   &   */ Amp_,     Comment,  Join,    Assign,  Add,       Sub,       AmpAmp_,      LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   ^   */ Caret_,   Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       CaretCaret_,  LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   |   */ Pipe_,    Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       PipePipe_,   LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   <   */ Lt_,      Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       LtLt_,  Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   >   */ Gt_,      Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       GtGt_,  Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   ~   */ BitNot,   Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   !   */ Bang_,    Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   *   */ Star_,    Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   /   */ Slash_,   Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   %   */ Percent_, Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   ;   */ Eos,      Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   :   */ Colon_,   Comment,  Public,  Assign,  Signed,    Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Unsigned,  Error,
/*   ?   */ Unknown,  Comment,  Weak,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   $   */ Param,    Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   @   */ Alias,    Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/*   \   */ BSlash_,  Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       LogXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,

/*  Eof  */ End,      End,      Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       BitXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
/* Other */ Error,    Comment,  Join,    Assign,  Add,       Sub,       BitAnd,       LogAnd,       BitXor,       BitXor,       BitOr,       LogOr,       Less,   Shl,       More,   Shr,       LogNot, Mul,       Div,       Mod,       Error,
]};

// ----------------------------------------------------------------------------

impl<I: Iterator<Item = u8>> Lexer<I> {
    /// Scans a token.
    pub(super) fn scan_main(&mut self) -> Token {
        use Action::*;

        // Apply deferred info from previous call
        self.line = self.line_next;

        // Every call begins in `Normal` state with no token found
        let mut state   = State::Normal;
        let mut start   = 0;

        // Scan until a token is found
        let token = loop {
            // Translate input into action
            // // flag = 0 or 1
            let (action, start_flag, _variant_flag) = {
                let input      = self.input.classify(&CHARS).0;
                let transition = TRANSITION_MAP[state as usize + input as usize];
                transition.decode()
            };

            // Record token start position
            start   |= (start_flag as usize).wrapping_neg() & self.input.position();

            // Perform action
            match action {
                Continue (s) => state = self.transition(s),
                Error        => state = self.add_error(),
                Yield    (t) => break              t,
                Produce  (t) => break self.produce(t),
                ScanCrLfEos  => break self.scan_crlf_eos(),
                ScanCrLf     =>       self.scan_crlf(),
                ScanLfEos    => break self.scan_lf_eos(),
                ScanLf       =>       self.scan_lf(),
                ScanBin      => if let Some(t) = self.scan_num(Bin)       { break t; },
                ScanOct      => if let Some(t) = self.scan_num(Oct)       { break t; },
                ScanDec      => if let Some(t) = self.scan_num(Dec)       { break t; },
                ScanHex      => if let Some(t) = self.scan_num(Hex)       { break t; },
                ScanStr      => if let Some(t) = self.scan_str()          { break t; },
                ScanChar     => if let Some(t) = self.scan_char()         { break t; },
                ScanIdent    => if let Some(t) = self.scan_ident_or_lit() { break t; },
                ScanParam    => break self.scan_param(),
            }
        };

        // Yield token
        self.range = start..self.input.position();
        token
    }

    #[inline]
    fn transition(&mut self, s: State) -> State {
        self.input.advance();
        s
    }

    #[inline]
    fn produce(&mut self, tok: Token) -> Token {
        self.input.advance();
        tok
    }

    #[inline]
    fn scan_crlf(&mut self) {
        self.input.advance();
        self.input.advance_if(b'\n');
        self.line_next += 1;
    }

    #[inline]
    fn scan_lf(&mut self) {
        self.input.advance();
        self.line_next += 1;
    }

    #[inline]
    fn scan_crlf_eos(&mut self) -> Token {
        self.scan_crlf();
        Token::Eos
    }

    #[inline]
    fn scan_lf_eos(&mut self) -> Token {
        self.scan_lf();
        Token::Eos
    }

    #[inline]
    fn scan_param(&mut self) -> Token {
        self.input.advance();
        self.scan_ident_or_lit();
        Token::Param
    }

    fn add_error(&mut self) -> State {
        // add error here
        self.input.advance();
        State::Normal
    }
}
