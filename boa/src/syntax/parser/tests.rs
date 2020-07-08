//! Tests for the parser.

use super::Parser;
use crate::syntax::ast::{
    node::{
        field::GetConstField, Assign, BinOp, Call, FunctionDecl, Identifier, LetDecl, LetDeclList,
        New, Node, Return, StatementList, UnaryOp, VarDecl, VarDeclList,
    },
    op::{self, CompOp, LogOp, NumOp},
    Const,
};

/// Checks that the given JavaScript string gives the expected expression.
#[allow(clippy::result_unwrap_used)]
// TODO: #[track_caller]: https://github.com/rust-lang/rust/issues/47809
pub(super) fn check_parser<L>(js: &str, expr: L)
where
    L: Into<Box<[Node]>>,
{
    assert_eq!(
        Parser::new(js.as_bytes())
            .parse_all()
            .expect("failed to parse"),
        StatementList::from(expr)
    );
}

/// Checks that the given javascript string creates a parse error.
// TODO: #[track_caller]: https://github.com/rust-lang/rust/issues/47809
pub(super) fn check_invalid(js: &str) {
    assert!(Parser::new(js.as_bytes()).parse_all().is_err());
}

/// Should be parsed as `new Class().method()` instead of `new (Class().method())`
#[test]
fn check_construct_call_precedence() {
    check_parser(
        "new Date().getTime()",
        vec![Node::from(Call::new(
            GetConstField::new(
                New::from(Call::new(Identifier::from("Date"), vec![])),
                "getTime",
            ),
            vec![],
        ))],
    );
}

#[test]
fn assign_operator_precedence() {
    check_parser(
        "a = a + 1",
        vec![Assign::new(
            Identifier::from("a"),
            BinOp::new(NumOp::Add, Identifier::from("a"), Const::from(1)),
        )
        .into()],
    );
}

#[test]
fn hoisting() {
    check_parser(
        r"
            var a = hello();
            a++;

            function hello() { return 10 }",
        vec![
            FunctionDecl::new(
                Box::from("hello"),
                vec![],
                vec![Return::new(Const::from(10), None).into()],
            )
            .into(),
            VarDeclList::from(vec![VarDecl::new(
                "a",
                Node::from(Call::new(Identifier::from("hello"), vec![])),
            )])
            .into(),
            UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::from("a")).into(),
        ],
    );

    check_parser(
        r"
            a = 10;
            a++;

            var a;",
        vec![
            Assign::new(Identifier::from("a"), Const::from(10)).into(),
            UnaryOp::new(op::UnaryOp::IncrementPost, Identifier::from("a")).into(),
            VarDeclList::from(vec![VarDecl::new("a", None)]).into(),
        ],
    );
}

#[test]
fn ambigous_regex_divide_expression() {
    let s = "1 / a === 1 / b";

    check_parser(
        s,
        vec![BinOp::new(
            CompOp::StrictEqual,
            BinOp::new(NumOp::Div, Const::Int(1), Identifier::from("a")),
            BinOp::new(NumOp::Div, Const::Int(1), Identifier::from("b")),
        )
        .into()],
    );
}

#[test]
fn two_divisions_in_expression() {
    let s = "a !== 0 || 1 / a === 1 / b;";

    check_parser(
        s,
        vec![BinOp::new(
            LogOp::Or,
            BinOp::new(CompOp::StrictNotEqual, Identifier::from("a"), Const::Int(0)),
            BinOp::new(
                CompOp::StrictEqual,
                BinOp::new(NumOp::Div, Const::Int(1), Identifier::from("a")),
                BinOp::new(NumOp::Div, Const::Int(1), Identifier::from("b")),
            ),
        )
        .into()],
    );
}

#[test]
fn comment_semi_colon_insertion() {
    let s = r#"
    let a = 10 // Comment
    let b = 20;
    "#;

    check_parser(
        s,
        vec![
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "a",
                Some(Const::Int(10).into()),
            )
            .into()])
            .into(),
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "b",
                Some(Const::Int(20).into()),
            )
            .into()])
            .into(),
        ],
    );
}

#[test]
fn multiline_comment_semi_colon_insertion() {
    let s = r#"
    let a = 10 /* Test
    Multiline
    Comment
    */ let b = 20;
    "#;

    check_parser(
        s,
        vec![
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "a",
                Some(Const::Int(10).into()),
            )
            .into()])
            .into(),
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "b",
                Some(Const::Int(20).into()),
            )
            .into()])
            .into(),
        ],
    );
}

#[test]
fn multiline_comment_no_lineterminator() {
    let s = r#"
    let a = 10; /* Test comment */ let b = 20;
    "#;

    check_parser(
        s,
        vec![
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "a",
                Some(Const::Int(10).into()),
            )
            .into()])
            .into(),
            LetDeclList::from(vec![LetDecl::new::<&str, Option<Node>>(
                "b",
                Some(Const::Int(20).into()),
            )
            .into()])
            .into(),
        ],
    );
}
