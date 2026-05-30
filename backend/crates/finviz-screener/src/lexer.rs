//! Tokenizer for the screener filter DSL.
//!
//! Grammar (informal):
//! ```text
//! price > 50 and pe < 25 and sector = "Technology" and (volume > 1e6 or beta < 1)
//! ```

use crate::error::ScreenerError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(f64),
    Str(String),
    Bool(bool),
    And,
    Or,
    Not,
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
    /// Case-insensitive substring match (`~`).
    Like,
    LParen,
    RParen,
    Eof,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ScreenerError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            ws if ws.is_whitespace() => {
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '~' => {
                tokens.push(Token::Like);
                i += 1;
            }
            '>' => {
                if peek(&chars, i + 1) == Some('=') {
                    tokens.push(Token::Ge);
                    i += 2;
                } else {
                    tokens.push(Token::Gt);
                    i += 1;
                }
            }
            '<' => match peek(&chars, i + 1) {
                Some('=') => {
                    tokens.push(Token::Le);
                    i += 2;
                }
                Some('>') => {
                    tokens.push(Token::Ne);
                    i += 2;
                }
                _ => {
                    tokens.push(Token::Lt);
                    i += 1;
                }
            },
            '=' => {
                // accept both `=` and `==`
                if peek(&chars, i + 1) == Some('=') {
                    i += 2;
                } else {
                    i += 1;
                }
                tokens.push(Token::Eq);
            }
            '!' => {
                if peek(&chars, i + 1) == Some('=') {
                    tokens.push(Token::Ne);
                    i += 2;
                } else {
                    return Err(ScreenerError::Lex(format!("unexpected '!' at position {i}")));
                }
            }
            '"' | '\'' => {
                let quote = c;
                let start = i + 1;
                i += 1;
                let mut s = String::new();
                let mut closed = false;
                while i < chars.len() {
                    let ch = chars[i];
                    if ch == quote {
                        closed = true;
                        i += 1;
                        break;
                    }
                    s.push(ch);
                    i += 1;
                }
                if !closed {
                    return Err(ScreenerError::Lex(format!(
                        "unterminated string starting at position {}",
                        start - 1
                    )));
                }
                tokens.push(Token::Str(s));
            }
            d if d.is_ascii_digit() || (d == '.' && peek(&chars, i + 1).is_some_and(|n| n.is_ascii_digit())) => {
                let start = i;
                while i < chars.len() && is_number_char(chars[i]) {
                    // handle exponent sign: e+6 / e-3
                    if (chars[i] == '+' || chars[i] == '-')
                        && !matches!(peek(&chars, i.wrapping_sub(1)), Some('e') | Some('E'))
                    {
                        break;
                    }
                    i += 1;
                }
                let raw: String = chars[start..i].iter().collect();
                let n: f64 = raw
                    .parse()
                    .map_err(|_| ScreenerError::Lex(format!("invalid number: {raw}")))?;
                tokens.push(Token::Number(n));
            }
            a if a.is_alphabetic() || a == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '.') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                match word.to_ascii_lowercase().as_str() {
                    "and" => tokens.push(Token::And),
                    "or" => tokens.push(Token::Or),
                    "not" => tokens.push(Token::Not),
                    "true" => tokens.push(Token::Bool(true)),
                    "false" => tokens.push(Token::Bool(false)),
                    _ => tokens.push(Token::Ident(word)),
                }
            }
            other => {
                return Err(ScreenerError::Lex(format!(
                    "unexpected character {other:?} at position {i}"
                )));
            }
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

fn peek(chars: &[char], i: usize) -> Option<char> {
    chars.get(i).copied()
}

fn is_number_char(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_a_full_expression() {
        let toks = tokenize("price > 1e6 and sector = \"Tech\"").unwrap();
        assert_eq!(
            toks,
            vec![
                Token::Ident("price".into()),
                Token::Gt,
                Token::Number(1_000_000.0),
                Token::And,
                Token::Ident("sector".into()),
                Token::Eq,
                Token::Str("Tech".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn handles_comparison_operators() {
        let toks = tokenize("a>=1 b<=2 c<>3 d!=4 e==5 f~6").unwrap();
        assert!(toks.contains(&Token::Ge));
        assert!(toks.contains(&Token::Le));
        assert_eq!(toks.iter().filter(|t| **t == Token::Ne).count(), 2);
        assert!(toks.contains(&Token::Eq));
        assert!(toks.contains(&Token::Like));
    }

    #[test]
    fn rejects_unterminated_string() {
        assert!(tokenize("name = \"oops").is_err());
    }
}
