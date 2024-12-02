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
pub enum Token<T>
where
    T: WktNum,
{
    Comma,
    Number(T),
    ParenClose,
    ParenOpen,
    Word(String),
}

#[inline]
fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\n' || c == '\r' || c == '\t'
}

#[inline]
fn is_numberlike(c: char) -> bool {
    c == '.' || c == '-' || c == '+' || c.is_ascii_digit()
}

pub type PeekableTokens<'a, T> = Peekable<Tokens<'a, T>>;

#[derive(Debug)]
pub struct Tokens<'a, T> {
    chars: Peekable<str::Chars<'a>>,
    phantom: PhantomData<T>,
}

impl<'a, T> Tokens<'a, T>
where
    T: WktNum,
{
    pub fn from_str(input: &'a str) -> Self {
        Tokens {
            chars: input.chars().peekable(),
            phantom: PhantomData,
        }
    }
}

impl<T> Iterator for Tokens<'_, T>
where
    T: WktNum + str::FromStr,
{
    type Item = Result<Token<T>, &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: should this return Result?
        let mut next_char = self.chars.next()?;

        // Skip whitespace
        while is_whitespace(next_char) {
            next_char = self.chars.next()?
        }

        let token = match next_char {
            '\0' => return None,
            '(' => Token::ParenOpen,
            ')' => Token::ParenClose,
            ',' => Token::Comma,
            c if is_numberlike(c) => {
                let number = self.read_until_whitespace(if c == '+' { None } else { Some(c) });
                match number.parse::<T>() {
                    Ok(parsed_num) => Token::Number(parsed_num),
                    Err(_) => {
                        log::warn!(
                            "Failed to parse input: '{}' as {}",
                            &number,
                            type_name::<T>()
                        );
                        return Some(Err(
                            "Unable to parse input number as the desired output type",
                        ));
                    }
                }
            }
            c => Token::Word(self.read_until_whitespace(Some(c))),
        };
        Some(Ok(token))
    }
}

impl<T> Tokens<'_, T>
where
    T: str::FromStr,
{
    fn read_until_whitespace(&mut self, first_char: Option<char>) -> String {
        let mut result = String::with_capacity(12); // Big enough for most tokens
        if let Some(c) = first_char {
            result.push(c);
        }

        while let Some(&next_char) = self.chars.peek() {
            match next_char {
                '\0' | '(' | ')' | ',' => break, // Just stop on a marker
                c if is_whitespace(c) => {
                    let _ = self.chars.next();
                    break;
                }
                _ => {
                    result.push(next_char);
                    let _ = self.chars.next();
                }
            }
        }

        result
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
    assert_eq!(tokens, vec![Token::Word("hello".to_string())]);
}

#[test]
fn test_tokenizer_2words() {
    let test_str = "hello world";
    let tokens: Result<Vec<Token<f64>>, _> = Tokens::from_str(test_str).collect();
    let tokens = tokens.unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Word("hello".to_string()),
            Token::Word("world".to_string()),
        ]
    );
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
    assert_eq!(tokens, vec![Token::Word("¾".to_owned())]);
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
            Token::Word("POINT".to_string()),
            Token::ParenOpen,
            Token::Number(10.0),
            Token::Number(-20.0),
            Token::ParenClose,
        ]
    );
}
