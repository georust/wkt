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

use crate::WktNum;
use std::any::type_name;
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
    T: WktNum + str::FromStr,
{
    type Item = Result<Token<'a, T>, &'static str>;

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
                let start = self.i;
                let end = self.read_until_break();
                let number = &input[start..end];
                match number.parse::<T>() {
                    Ok(parsed_num) => Token::Number(parsed_num),
                    Err(_) => {
                        log::warn!(
                            "Failed to parse input: '{}' as {}",
                            number,
                            type_name::<T>()
                        );
                        return Some(Err(
                            "Unable to parse input number as the desired output type",
                        ));
                    }
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

impl<T> Tokens<'_, T>
where
    T: str::FromStr,
{
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
    assert_eq!(
        tokens,
        "Unable to parse input number as the desired output type"
    );
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
