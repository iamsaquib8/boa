//! Boa parser implementation.

pub mod error;
mod expression;
mod function;
mod statement;
#[cfg(test)]
mod tests;

use self::error::{ParseError, ParseResult};
use crate::syntax::ast::node::StatementList;
use crate::syntax::lexer::Token;
use crate::syntax::lexer::Lexer;
use crate::syntax::ast::Node;

use ParseError as Error;

use std::io::Read;

/// Trait implemented by parsers.
///
/// This makes it possible to abstract over the underlying implementation of a parser.
trait TokenParser<R>: Sized 
where 
    R: Read
{
    /// Output type for the parser.
    type Output; // = Node; waiting for https://github.com/rust-lang/rust/issues/29661

    /// Parses the token stream using the current parser.
    ///
    /// This method needs to be provided by the implementor type.
    fn parse(self, parser: &mut Parser<R>) -> Result<Node, ParseError>;

    // /// Tries to parse the following tokens with this parser.
    // fn try_parse(self, parser: Parser<R>) -> Option<Self::Output> {
    //     let initial_pos = cursor.pos();
    //     if let Ok(node) = self.parse(cursor) {
    //         Some(node)
    //     } else {
    //         cursor.seek(initial_pos);
    //         None
    //     }
    // }
}

/// Boolean representing if the parser should allow a `yield` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowYield(bool);

impl From<bool> for AllowYield {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `await` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowAwait(bool);

impl From<bool> for AllowAwait {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `in` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowIn(bool);

impl From<bool> for AllowIn {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `return` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowReturn(bool);

impl From<bool> for AllowReturn {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

/// Boolean representing if the parser should allow a `default` keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllowDefault(bool);

impl From<bool> for AllowDefault {
    fn from(allow: bool) -> Self {
        Self(allow)
    }
}

#[derive(Debug)]
pub struct Parser <R> {
    /// Lexer used to get tokens for the parser.
    lexer: Lexer<R>,
}

impl<R> Parser<R> {
    pub fn new(reader: R) -> Self {
        Self {
            lexer: Lexer::new(reader)
        }
    }
}

impl<R> Iterator for Parser<R>
where
    R: Read,
{
    type Item = Result<Node, Error>;

    fn next(&mut self) -> Option<Self::Item> {

    }
}

/// Parses a full script.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-Script
#[derive(Debug, Clone, Copy)]
pub struct Script;

impl<R> TokenParser<R> for Script {
    type Output = StatementList;

    fn parse(self, parser: &mut Parser<R>) -> Result<Self::Output, ParseError> {
        if cursor.peek(0).is_some() {
            ScriptBody.parse(parser)
        } else {
            Ok(StatementList::from(Vec::new()))
        }
    }
}

/// Parses a script body.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-ScriptBody
#[derive(Debug, Clone, Copy)]
pub struct ScriptBody;

impl<R> TokenParser<R> for ScriptBody {
    type Output = StatementList;

    fn parse(self, parser: &mut Parser<R>) -> Result<Self::Output, ParseError> {
        self::statement::StatementList::new(false, false, false, false).parse(parser)
    }
}
