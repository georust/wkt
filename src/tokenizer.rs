// Copyright 2014-2015 The GeoRust Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::error::ParseError;
use crate::WktNum;
use std::any::{type_name, Any, TypeId};
use std::iter::Peekable;
use std::marker::PhantomData;
use std::str;

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a, T>
where
    T: WktNum,
{
    Comma,
    Number(T),
    ParenClose,
    ParenOpen,
    Word(&'a str),
}

#[inline]
fn is_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\n' || c == b'\r' || c == b'\t'
}

// A byte that ends a word or number token.
#[inline]
fn is_break(c: u8) -> bool {
    c == b'\0' || c == b'(' || c == b')' || c == b',' || is_whitespace(c)
}

#[inline]
fn is_numberlike(c: u8) -> bool {
    c == b'.' || c == b'-' || c == b'+' || c.is_ascii_digit()
}

pub type PeekableTokens<'a, T> = Peekable<Tokens<'a, T>>;

#[derive(Debug)]
pub struct Tokens<'a, T> {
    input: &'a str,
    i: usize,
    phantom: PhantomData<T>,
}

impl<'a, T> Tokens<'a, T>
where
    T: WktNum,
{
    pub fn from_str(input: &'a str) -> Self {
        Tokens {
            input,
            i: 0,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Iterator for Tokens<'a, T>
where
    T: WktNum + str::FromStr + 'static,
{
    type Item = Result<Token<'a, T>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let input = self.input;
        let bytes = input.as_bytes();

        // Skip whitespace
        while self.i < bytes.len() && is_whitespace(bytes[self.i]) {
            self.i += 1;
        }
        if self.i >= bytes.len() {
            return None;
        }

        let c = bytes[self.i];
        let token = match c {
            b'\0' => return None,
            b'(' => {
                self.i += 1;
                Token::ParenOpen
            }
            b')' => {
                self.i += 1;
                Token::ParenClose
            }
            b',' => {
                self.i += 1;
                Token::Comma
            }
            c if is_numberlike(c) => {
                // A leading '+' is not part of the number token.
                if c == b'+' {
                    self.i += 1;
                }
                match self.read_number() {
                    Ok(parsed_num) => Token::Number(parsed_num),
                    Err(e) => return Some(Err(e)),
                }
            }
            _ => {
                let start = self.i;
                let end = self.read_until_break();
                // Tokens only end on ASCII bytes, which can never be part of
                // a multi-byte UTF-8 sequence, so token indices always fall
                // on char boundaries and slicing the input cannot panic.
                Token::Word(&input[start..end])
            }
        };
        Some(Ok(token))
    }
}

// Parses a float at the start of `bytes` if `T` is `f32` or `f64`. Returns the
// value and the number of bytes it uses, or `None` if `T` is a different type.
#[inline]
fn parse_float_prefix<T: 'static + Copy>(
    bytes: &[u8],
) -> Option<Result<(T, usize), fast_float2::Error>> {
    fn cast<F: 'static, T: 'static + Copy>(
        result: Result<(F, usize), fast_float2::Error>,
    ) -> Option<Result<(T, usize), fast_float2::Error>> {
        match result {
            Ok((value, len)) => (&value as &dyn Any)
                .downcast_ref::<T>()
                .map(|value| Ok((*value, len))),
            Err(e) => Some(Err(e)),
        }
    }
    let id = TypeId::of::<T>();
    if id == TypeId::of::<f64>() {
        cast(fast_float2::parse_partial::<f64, _>(bytes))
    } else if id == TypeId::of::<f32>() {
        cast(fast_float2::parse_partial::<f32, _>(bytes))
    } else {
        None
    }
}

impl<T> Tokens<'_, T>
where
    T: WktNum + str::FromStr + 'static,
{
    // Parses the number that begins at `self.i`.
    fn read_number(&mut self) -> Result<T, ParseError> {
        let bytes = self.input.as_bytes();
        if let Some(result) = parse_float_prefix::<T>(&bytes[self.i..]) {
            // For floats, the parser finds the end of the number, so the
            // number bytes are read one time only.
            let start = self.i;
            if let Ok((value, len)) = result {
                let end = start + len;
                if bytes.get(end).is_none_or(|&c| is_break(c)) {
                    self.i = end;
                    return Ok(value);
                }
            }
            // Find the full token for the log message.
            let end = self.read_until_break();
            return Err(self.number_error(start, end));
        }

        let start = self.i;
        let end = self.read_until_break();
        self.input[start..end]
            .parse::<T>()
            .map_err(|_| self.number_error(start, end))
    }

    fn number_error(&self, start: usize, end: usize) -> ParseError {
        log::warn!(
            "Failed to parse input: '{}' as {}",
            &self.input[start..end],
            type_name::<T>()
        );
        ParseError::InvalidNumber
    }

    // Returns the end index of the token beginning at `self.i`, consuming
    // one trailing whitespace byte; marker bytes end the token unconsumed.
    fn read_until_break(&mut self) -> usize {
        let bytes = self.input.as_bytes();
        let mut end = self.i;
        while self.i < bytes.len() {
            match bytes[self.i] {
                b'\0' | b'(' | b')' | b',' => break,
                c if is_whitespace(c) => {
                    self.i += 1;
                    break;
                }
                _ => {
                    self.i += 1;
                    end = self.i;
                }
            }
        }
        end
    }
}

#[test]
fn test_tokenizer_empty() {
    let test_str = "";
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap();
    assert_eq!(tokens, vec![]);
}

#[test]
fn test_tokenizer_1word() {
    let test_str = "hello";
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap();
    assert_eq!(tokens, vec![Token::Word("hello")]);
}

#[test]
fn test_tokenizer_2words() {
    let test_str = "hello world";
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap();
    assert_eq!(tokens, vec![Token::Word("hello"), Token::Word("world")]);
}

#[test]
fn test_tokenizer_1number() {
    let test_str = "4.2";
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap();
    assert_eq!(tokens, vec![Token::Number(4.2)]);
}

#[test]
fn test_tokenizer_1number_plus() {
    let test_str = "+4.2";
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap();
    assert_eq!(tokens, vec![Token::Number(4.2)]);
}

#[test]
fn test_tokenizer_invalid_number() {
    let test_str = "4.2p";
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap_err();
    assert_eq!(tokens, ParseError::InvalidNumber);
}

#[test]
fn test_tokenizer_not_a_number() {
    let test_str = "¾"; // A number according to char.is_numeric()
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap();
    assert_eq!(tokens, vec![Token::Word("¾")]);
}

#[test]
fn test_tokenizer_2numbers() {
    let test_str = ".4 -2";
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap();
    assert_eq!(tokens, vec![Token::Number(0.4), Token::Number(-2.0)]);
}

#[test]
fn test_no_stack_overflow() {
    fn check(c: &str, count: usize, expected: usize) {
        let test_str = c.repeat(count);
        assert_eq!(
            expected,
            Tokens::<f64>::from_str(&test_str)
                .filter(Result::is_ok)
                .count()
        );
    }

    let count = 100_000;
    check("+", count, 0);
    check(" ", count, 0);
    check("A", count, 1);
    check("1", count, 1);
    check("(", count, count);
    check(")", count, count);
    check(",", count, count);
}

#[test]
fn test_tokenizer_point() {
    let test_str = "POINT (10 -20)";
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Word("POINT"),
            Token::ParenOpen,
            Token::Number(10.0),
            Token::Number(-20.0),
            Token::ParenClose,
        ]
    );
}

#[test]
fn test_tokenizer_utf8_word() {
    // Multi-byte characters must not break byte scanning.
    let test_str = "POINT¾Z";
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap();
    assert_eq!(tokens, vec![Token::Word("POINT¾Z")]);
}

#[cfg(test)]
mod number_tests {
    use super::{Token, Tokens};
    use std::str::FromStr;

    // The number parsing of the tokenizer before the one-pass float path:
    // remove one leading '+', then parse the full token with `FromStr`.
    fn reference<T: FromStr>(token: &str) -> Option<T> {
        token.strip_prefix('+').unwrap_or(token).parse().ok()
    }

    fn first<T: crate::WktNum + FromStr + 'static>(input: &str) -> Option<T> {
        match Tokens::<T>::from_str(input).next() {
            Some(Ok(Token::Number(n))) => Some(n),
            _ => None,
        }
    }

    const CASES: &[&str] = &[
        "0",
        "-0",
        "+0",
        "1",
        "-1",
        "+1",
        "++1",
        "+-1",
        "-+1",
        "--1",
        "1.",
        ".5",
        "-.5",
        "+.5",
        "-",
        "+",
        ".",
        "-.",
        "1.5",
        "10.25",
        "6480000.001",
        "1e5",
        "1E5",
        "1e+5",
        "1e-5",
        "-1.5e-300",
        "1e",
        "1e+",
        "1.2.3",
        "1..2",
        "4.2p",
        "1x",
        "0x10",
        "1_000",
        "-inf",
        "+inf",
        "-infinity",
        "-Infinity",
        "+INF",
        "-nan",
        "+NaN",
        "-NAN",
        "1e400",
        "-1e400",
        "1e-400",
        "2.2250738585072014e-308",
        "4.9e-324",
        "123456789012345678901234567890",
        "0.1000000000000000055511151231257827",
        "3.4028235e38",
        "3.4028236e38",
        "1.17549435e-38",
        "1.0¾",
        "00012",
        "-000.000",
    ];

    fn check<T: crate::WktNum + FromStr + 'static>(eq: impl Fn(T, T) -> bool) {
        for case in CASES {
            for suffix in ["", " ", ",", ")", "(", "\n", "\0"] {
                let input = format!("{case}{suffix}");
                let expected = reference::<T>(case);
                let actual = first::<T>(&input);
                let same = match (expected, actual) {
                    (Some(a), Some(b)) => eq(a, b),
                    (None, None) => true,
                    _ => false,
                };
                assert!(same, "{input:?}: expected {expected:?}, got {actual:?}");
            }
        }
    }

    #[test]
    fn f64_matches_from_str() {
        check::<f64>(|a, b| a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()));
    }

    #[test]
    fn f32_matches_from_str() {
        check::<f32>(|a, b| a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()));
    }

    #[test]
    fn integer_path_unchanged() {
        check::<i64>(|a, b| a == b);
    }

    #[test]
    fn number_followed_by_tokens() {
        let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str("1.5,-2e1)3 4(").collect();
        assert_eq!(
            tokens.unwrap(),
            vec![
                Token::Number(1.5),
                Token::Comma,
                Token::Number(-20.0),
                Token::ParenClose,
                Token::Number(3.0),
                Token::Number(4.0),
                Token::ParenOpen,
            ]
        );
    }
}
