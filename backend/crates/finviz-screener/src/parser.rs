//! Recursive-descent parser: tokens -> [`Expr`] AST.
//!
//! Precedence (lowest to highest): `or` < `and` < `not` < comparison.

use crate::ast::{CmpOp, Expr, Literal};
use crate::error::ScreenerError;
use crate::lexer::{tokenize, Token};

pub fn parse(input: &str) -> Result<Expr, ScreenerError> {
    let tokens = tokenize(input)?;
    let mut p = Parser { tokens, pos: 0 };
    let expr = p.parse_or()?;
    p.expect(Token::Eof)?;
    Ok(expr)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, want: Token) -> Result<(), ScreenerError> {
        if *self.peek() == want {
            self.advance();
            Ok(())
        } else {
            Err(ScreenerError::Parse(format!(
                "expected {want:?}, found {:?}",
                self.peek()
            )))
        }
    }

    fn parse_or(&mut self) -> Result<Expr, ScreenerError> {
        let mut left = self.parse_and()?;
        while *self.peek() == Token::Or {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ScreenerError> {
        let mut left = self.parse_not()?;
        while *self.peek() == Token::And {
            self.advance();
            let right = self.parse_not()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, ScreenerError> {
        if *self.peek() == Token::Not {
            self.advance();
            let inner = self.parse_not()?;
            return Ok(Expr::Not(Box::new(inner)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ScreenerError> {
        if *self.peek() == Token::LParen {
            self.advance();
            let inner = self.parse_or()?;
            self.expect(Token::RParen)?;
            return Ok(inner);
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, ScreenerError> {
        let field = match self.advance() {
            Token::Ident(name) => name,
            other => {
                return Err(ScreenerError::Parse(format!(
                    "expected a field name, found {other:?}"
                )))
            }
        };

        let op = match self.advance() {
            Token::Gt => CmpOp::Gt,
            Token::Lt => CmpOp::Lt,
            Token::Ge => CmpOp::Ge,
            Token::Le => CmpOp::Le,
            Token::Eq => CmpOp::Eq,
            Token::Ne => CmpOp::Ne,
            Token::Like => CmpOp::Like,
            other => {
                return Err(ScreenerError::Parse(format!(
                    "expected a comparison operator after `{field}`, found {other:?}"
                )))
            }
        };

        let value = match self.advance() {
            Token::Number(n) => Literal::Num(n),
            Token::Str(s) => Literal::Str(s),
            Token::Bool(b) => Literal::Bool(b),
            other => {
                return Err(ScreenerError::Parse(format!(
                    "expected a literal value after operator, found {other:?}"
                )))
            }
        };

        Ok(Expr::Compare { field, op, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_precedence_correctly() {
        // a or b and c  ==  a or (b and c)
        let expr = parse("price > 1 or pe < 2 and beta > 3").unwrap();
        match expr {
            Expr::Or(_, right) => assert!(matches!(*right, Expr::And(_, _))),
            other => panic!("expected Or at root, got {other:?}"),
        }
    }

    #[test]
    fn parentheses_override_precedence() {
        let expr = parse("(price > 1 or pe < 2) and beta > 3").unwrap();
        assert!(matches!(expr, Expr::And(_, _)));
    }

    #[test]
    fn rejects_trailing_garbage() {
        assert!(parse("price > 1 pe < 2").is_err());
    }
}
