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

const NUMBER_ERROR: &str = "Unable to parse input number as the desired output type";

impl<T> Tokens<'_, T>
where
    T: WktNum + str::FromStr,
{
    // Parses the number that begins at `self.i`.
    fn read_number(&mut self) -> Result<T, &'static str> {
        let bytes = self.input.as_bytes();
        if let Some((value, len)) = T::parse_prefix(&bytes[self.i..]) {
            let end = self.i + len;
            if bytes.get(end).is_none_or(|&c| is_break(c)) {
                self.i = end;
                return Ok(value);
            }
        }

        let start = self.i;
        let end = self.read_until_break();
        self.input[start..end]
            .parse::<T>()
            .map_err(|_| self.number_error(start, end))
    }

    fn number_error(&self, start: usize, end: usize) -> &'static str {
        log::warn!(
            "Failed to parse input: '{}' as {}",
            &self.input[start..end],
            type_name::<T>()
        );
        NUMBER_ERROR
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

#[cfg(test)]
mod number_tests {
    use super::{Token, Tokens};
    use std::str::FromStr;

    // The number parsing of the tokenizer before the one-pass float path:
    // remove one leading '+', then parse the full token with `FromStr`.
    fn reference<T: FromStr>(token: &str) -> Option<T> {
        token.strip_prefix('+').unwrap_or(token).parse().ok()
    }

    fn first<T: crate::WktNum + FromStr>(input: &str) -> Option<T> {
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

    fn check<T: crate::WktNum + FromStr>(eq: impl Fn(T, T) -> bool) {
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
        check::<f64>(|a, b| a.to_bits() == b.to_bits());
    }

    #[test]
    fn f32_matches_from_str() {
        check::<f32>(|a, b| a.to_bits() == b.to_bits());
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

// Differential properties: the tokenizer must give the same tokens as the
// tokenizer before the one-pass float path, which `reference_tokens` copies.
#[cfg(test)]
mod property_tests {
    use super::{Token, Tokens};
    use crate::WktNum;
    use hegel::generators as gs;
    use hegel::TestCase;
    use std::str::FromStr;

    // A token in a form that can be compared. `Number` holds a key for the
    // value, because NaN is not equal to NaN.
    #[derive(Debug, PartialEq)]
    enum Tok<K> {
        Comma,
        ParenOpen,
        ParenClose,
        Word(String),
        Number(K),
        Error,
    }

    // Bytes that can start or form numbers, the delimiters, letters that
    // occur in exponents and special values, and bytes that make a number
    // token invalid.
    const ALPHABET: &str = "0123456789.+-eEinftyaNI xp_\u{be}\u{a0}\u{feff}\0\n\r\t,()";

    // Number-shaped text: optional signs, digits, a fraction, an exponent
    // and the special values, then one optional trailing byte that is not
    // part of a number and one optional delimiter.
    const NUMBER_PATTERN: &str = concat!(
        r"[+-]{0,2}",
        r"([0-9]{0,22}(\.[0-9]{0,22})?([eE][+-]{0,2}[0-9]{0,4})?",
        r"|[iI][nN][fF]([iI][nN][iI][tT][yY])?|[nN][aA][nN])",
        r"[a-zA-Z_.+\-]?",
        r"[ ,()\x00\n]?",
    );

    fn is_whitespace(c: u8) -> bool {
        c == b' ' || c == b'\n' || c == b'\r' || c == b'\t'
    }

    // The tokenizer before the one-pass float path. A number token is the
    // full run of bytes up to a delimiter, with one leading '+' removed, and
    // `FromStr` parses it.
    fn reference_tokens<T: FromStr>(input: &str) -> Vec<Tok<T>> {
        let bytes = input.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        // Returns the token that begins at `i`. Moves `i` past the token and
        // past one trailing whitespace byte.
        let read_until_break = |i: &mut usize| -> &str {
            let start = *i;
            let mut end = *i;
            while *i < bytes.len() {
                match bytes[*i] {
                    b'\0' | b'(' | b')' | b',' => break,
                    c if is_whitespace(c) => {
                        *i += 1;
                        break;
                    }
                    _ => {
                        *i += 1;
                        end = *i;
                    }
                }
            }
            &input[start..end]
        };
        loop {
            while i < bytes.len() && is_whitespace(bytes[i]) {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }
            let token = match bytes[i] {
                b'\0' => break,
                b'(' => {
                    i += 1;
                    Tok::ParenOpen
                }
                b')' => {
                    i += 1;
                    Tok::ParenClose
                }
                b',' => {
                    i += 1;
                    Tok::Comma
                }
                c if c == b'.' || c == b'-' || c == b'+' || c.is_ascii_digit() => {
                    if c == b'+' {
                        i += 1;
                    }
                    match read_until_break(&mut i).parse::<T>() {
                        Ok(n) => Tok::Number(n),
                        Err(_) => Tok::Error,
                    }
                }
                _ => Tok::Word(read_until_break(&mut i).to_string()),
            };
            tokens.push(token);
        }
        tokens
    }

    fn map_number<T, K>(token: Tok<T>, key: &impl Fn(T) -> K) -> Tok<K> {
        match token {
            Tok::Comma => Tok::Comma,
            Tok::ParenOpen => Tok::ParenOpen,
            Tok::ParenClose => Tok::ParenClose,
            Tok::Word(w) => Tok::Word(w),
            Tok::Number(n) => Tok::Number(key(n)),
            Tok::Error => Tok::Error,
        }
    }

    // All the tokens of `input`. Iteration continues after an error, so that
    // the tokens after an invalid number are compared too.
    fn actual_tokens<T: WktNum + FromStr, K>(input: &str, key: &impl Fn(T) -> K) -> Vec<Tok<K>> {
        Tokens::<T>::from_str(input)
            .map(|result| match result {
                Ok(Token::Comma) => Tok::Comma,
                Ok(Token::ParenOpen) => Tok::ParenOpen,
                Ok(Token::ParenClose) => Tok::ParenClose,
                Ok(Token::Word(w)) => Tok::Word(w.to_string()),
                Ok(Token::Number(n)) => Tok::Number(key(n)),
                Err(_) => Tok::Error,
            })
            .collect()
    }

    fn assert_same_tokens<T: WktNum + FromStr, K: PartialEq + std::fmt::Debug>(
        input: &str,
        key: impl Fn(T) -> K,
    ) {
        let expected: Vec<Tok<K>> = reference_tokens::<T>(input)
            .into_iter()
            .map(|t| map_number(t, &key))
            .collect();
        assert_eq!(
            actual_tokens::<T, K>(input, &key),
            expected,
            "input: {input:?}"
        );
    }

    // The bits are compared, so NaN sign and payload must also agree.
    fn f64_key(n: f64) -> u64 {
        n.to_bits()
    }

    fn f32_key(n: f32) -> u32 {
        n.to_bits()
    }

    fn arbitrary_text(tc: &TestCase) -> String {
        tc.draw(gs::text().alphabet(ALPHABET).max_size(40))
    }

    // `count` copies of `digit`, or `count` random digits if `digit` is
    // `None`. The count is drawn uniformly from `0..=max`.
    fn digits(tc: &TestCase, digit: Option<char>, min: usize, max: usize) -> String {
        let count = tc.draw(gs::integers::<usize>().min_value(min).max_value(max));
        match digit {
            Some(d) => d.to_string().repeat(count),
            None => tc.draw(
                gs::text()
                    .alphabet("0123456789")
                    .min_size(count)
                    .max_size(count),
            ),
        }
    }

    // Long number-shaped text: up to 300 leading zeros and 400 digits in
    // the integer and fraction parts, and an exponent of up to 25 digits
    // after up to 30 zeros. Regex repeat counts are mostly small, so each
    // length is drawn explicitly.
    fn long_number_text(tc: &TestCase) -> String {
        let mut text: String = tc.draw(gs::sampled_from(&["", "+", "-"][..])).to_string();
        text += &digits(tc, Some('0'), 0, 300);
        text += &digits(tc, None, 0, 400);
        if tc.draw(gs::booleans()) {
            text.push('.');
            text += &digits(tc, Some('0'), 0, 300);
            text += &digits(tc, None, 0, 400);
        }
        if tc.draw(gs::booleans()) {
            text += tc.draw(gs::sampled_from(&["e", "E", "e+", "e-", "E+", "E-"][..]));
            text += &digits(tc, Some('0'), 0, 30);
            text += &digits(tc, None, 1, 25);
        }
        text += tc.draw(gs::sampled_from(&["", " ", ",", ")"][..]));
        text
    }

    fn number_shaped_text(tc: &TestCase) -> String {
        tc.draw(
            gs::vecs(gs::from_regex(NUMBER_PATTERN))
                .min_size(1)
                .max_size(4),
        )
        .concat()
    }

    #[hegel::test(test_cases = 5000)]
    fn f64_tokens_match_reference_on_any_text(tc: TestCase) {
        assert_same_tokens::<f64, _>(&tc.draw(gs::text()), f64_key);
    }

    #[hegel::test(test_cases = 5000)]
    fn f32_tokens_match_reference_on_any_text(tc: TestCase) {
        assert_same_tokens::<f32, _>(&tc.draw(gs::text()), f32_key);
    }

    #[hegel::test(test_cases = 5000)]
    fn f64_tokens_match_reference_on_arbitrary_text(tc: TestCase) {
        assert_same_tokens::<f64, _>(&arbitrary_text(&tc), f64_key);
    }

    #[hegel::test(test_cases = 5000)]
    fn f32_tokens_match_reference_on_arbitrary_text(tc: TestCase) {
        assert_same_tokens::<f32, _>(&arbitrary_text(&tc), f32_key);
    }

    #[hegel::test(test_cases = 2000)]
    fn i64_tokens_match_reference_on_arbitrary_text(tc: TestCase) {
        assert_same_tokens::<i64, _>(&arbitrary_text(&tc), |n| n);
    }

    #[hegel::test(test_cases = 5000)]
    fn f64_tokens_match_reference_on_number_shaped_text(tc: TestCase) {
        assert_same_tokens::<f64, _>(&number_shaped_text(&tc), f64_key);
    }

    #[hegel::test(test_cases = 5000)]
    fn f32_tokens_match_reference_on_number_shaped_text(tc: TestCase) {
        assert_same_tokens::<f32, _>(&number_shaped_text(&tc), f32_key);
    }

    #[hegel::test(test_cases = 2000)]
    fn i64_tokens_match_reference_on_number_shaped_text(tc: TestCase) {
        assert_same_tokens::<i64, _>(&number_shaped_text(&tc), |n| n);
    }

    #[hegel::test(test_cases = 2000)]
    fn i64_tokens_match_reference_on_long_numbers(tc: TestCase) {
        assert_same_tokens::<i64, _>(&long_number_text(&tc), |n| n);
    }

    #[hegel::test(test_cases = 2000)]
    fn f64_tokens_match_reference_on_long_numbers(tc: TestCase) {
        assert_same_tokens::<f64, _>(&long_number_text(&tc), f64_key);
    }

    #[hegel::test(test_cases = 2000)]
    fn f32_tokens_match_reference_on_long_numbers(tc: TestCase) {
        assert_same_tokens::<f32, _>(&long_number_text(&tc), f32_key);
    }

    // The exact decimal value halfway between two adjacent f32 values, and
    // the same value with a nonzero digit far after it. The halfway value
    // rounds to the value with an even mantissa, and the second value
    // rounds away from zero. Each decimal must give the same f32 as
    // `FromStr`. An f32 halfway value is exact in f64, so the decimal is
    // exact.
    #[hegel::test(test_cases = 5000)]
    fn f32_halfway_decimals_match_reference(tc: TestCase) {
        let x: f32 = tc.draw(gs::floats().allow_nan(false).allow_infinity(false));
        let next = f32::from_bits(x.to_bits() + 1);
        tc.assume(next.is_finite());
        let halfway = (f64::from(x) + f64::from(next)) / 2.0;
        let exact = format!("{halfway:.200e}");
        let (mantissa, exponent) = exact.split_once('e').unwrap();
        let sticky = format!("{mantissa}{}1e{exponent}", "0".repeat(100));
        for text in [exact.as_str(), sticky.as_str()] {
            assert_same_tokens::<f32, _>(text, f32_key);
        }
    }

    // Returns the decimal digits of `x` with 1100 fraction digits, without
    // the sign. The digits are exact: an f64 has at most 1074 fraction
    // digits.
    fn exact_fixed_digits(x: f64) -> String {
        format!("{:.1100}", x.abs())
    }

    // Returns (a + b) / 2 for two non-negative decimals that have the same
    // number of fraction digits. The result has one more fraction digit.
    fn decimal_mean(a: &str, b: &str) -> String {
        let (a_int, a_frac) = a.split_once('.').unwrap();
        let (b_int, b_frac) = b.split_once('.').unwrap();
        let width = a_int.len().max(b_int.len());
        let a_digits = format!("{a_int:0>width$}{a_frac}0");
        let b_digits = format!("{b_int:0>width$}{b_frac}0");
        // Add from the last digit, then halve from the first digit.
        let mut sum = vec![0u8; a_digits.len() + 1];
        let mut carry = 0;
        for (i, (x, y)) in a_digits.bytes().zip(b_digits.bytes()).enumerate().rev() {
            let digit = (x - b'0') + (y - b'0') + carry;
            sum[i + 1] = digit % 10;
            carry = digit / 10;
        }
        sum[0] = carry;
        let mut remainder = 0;
        let mut mean = String::with_capacity(sum.len() + 1);
        for (i, digit) in sum.iter().enumerate() {
            if i == width + 1 {
                mean.push('.');
            }
            let value = remainder * 10 + digit;
            mean.push(char::from(b'0' + value / 2));
            remainder = value % 2;
        }
        mean
    }

    // The exact decimal value halfway between two adjacent f64 values, and
    // the same value with a nonzero digit far after it. f64 cannot hold the
    // halfway value, so the decimal is calculated from the exact digits of
    // the two values. Each decimal must give the same f64 as `FromStr`.
    #[hegel::test(test_cases = 2000)]
    fn f64_halfway_decimals_match_reference(tc: TestCase) {
        let x: f64 = tc.draw(gs::floats().allow_nan(false).allow_infinity(false));
        let next = f64::from_bits(x.to_bits() + 1);
        tc.assume(next.is_finite());
        let sign = if x.is_sign_negative() { "-" } else { "" };
        let halfway = decimal_mean(&exact_fixed_digits(x), &exact_fixed_digits(next));
        let exact = format!("{sign}{halfway}");
        let sticky = format!("{exact}{}1", "0".repeat(100));
        for text in [exact.as_str(), sticky.as_str()] {
            assert_same_tokens::<f64, _>(text, f64_key);
        }
    }

    // Round-half-to-even: the exact halfway decimal gives the neighbour with
    // the even mantissa, and a decimal slightly above it in magnitude gives
    // the neighbour further from zero.
    #[hegel::test(test_cases = 2000)]
    fn f64_halfway_decimals_round_half_to_even(tc: TestCase) {
        let x: f64 = tc.draw(gs::floats().allow_nan(false).allow_infinity(false));
        let next = f64::from_bits(x.to_bits() + 1);
        tc.assume(next.is_finite());
        let sign = if x.is_sign_negative() { "-" } else { "" };
        let halfway = decimal_mean(&exact_fixed_digits(x), &exact_fixed_digits(next));
        let exact = format!("{sign}{halfway}");
        let sticky = format!("{exact}{}1", "0".repeat(100));
        let even = if x.to_bits().is_multiple_of(2) {
            x
        } else {
            next
        };
        assert_eq!(
            single_number::<f64>(&exact).map(f64::to_bits),
            Some(even.to_bits()),
            "text: {exact:?}"
        );
        assert_eq!(
            single_number::<f64>(&sticky).map(f64::to_bits),
            Some(next.to_bits()),
            "text: {sticky:?}"
        );
    }

    #[hegel::test(test_cases = 2000)]
    fn f32_halfway_decimals_round_half_to_even(tc: TestCase) {
        let x: f32 = tc.draw(gs::floats().allow_nan(false).allow_infinity(false));
        let next = f32::from_bits(x.to_bits() + 1);
        tc.assume(next.is_finite());
        let halfway = (f64::from(x) + f64::from(next)) / 2.0;
        let exact = format!("{halfway:.200e}");
        let (mantissa, exponent) = exact.split_once('e').unwrap();
        let sticky = format!("{mantissa}{}1e{exponent}", "0".repeat(100));
        let even = if x.to_bits().is_multiple_of(2) {
            x
        } else {
            next
        };
        assert_eq!(
            single_number::<f32>(&exact).map(f32::to_bits),
            Some(even.to_bits()),
            "text: {exact:?}"
        );
        assert_eq!(
            single_number::<f32>(&sticky).map(f32::to_bits),
            Some(next.to_bits()),
            "text: {sticky:?}"
        );
    }

    // The decimal mean is correct for values that it can check exactly.
    #[test]
    fn decimal_mean_examples() {
        assert_eq!(decimal_mean("1.0", "2.0"), "01.50");
        assert_eq!(decimal_mean("9.5", "10.5"), "010.00");
        assert_eq!(decimal_mean("0.1", "0.2"), "00.15");
    }

    fn single_number<T: WktNum + FromStr>(input: &str) -> Option<T> {
        let mut tokens = Tokens::<T>::from_str(input);
        match (tokens.next(), tokens.next()) {
            (Some(Ok(Token::Number(n))), None) => Some(n),
            _ => None,
        }
    }

    // The `Display` and `LowerExp` forms of a finite float hold enough
    // digits to give the same value when parsed.
    #[hegel::test(test_cases = 5000)]
    fn finite_f64_text_parses_to_same_value(tc: TestCase) {
        let x: f64 = tc.draw(gs::floats().allow_nan(false).allow_infinity(false));
        for text in [format!("{x}"), format!("{x:e}"), format!("{x:E}")] {
            let parsed = single_number::<f64>(&text);
            assert_eq!(
                parsed.map(f64::to_bits),
                Some(x.to_bits()),
                "text: {text:?}"
            );
        }
    }

    #[hegel::test(test_cases = 5000)]
    fn finite_f32_text_parses_to_same_value(tc: TestCase) {
        let x: f32 = tc.draw(gs::floats().allow_nan(false).allow_infinity(false));
        for text in [format!("{x}"), format!("{x:e}"), format!("{x:E}")] {
            let parsed = single_number::<f32>(&text);
            assert_eq!(
                parsed.map(f32::to_bits),
                Some(x.to_bits()),
                "text: {text:?}"
            );
        }
    }

    // The crate writes a point and parses the text back to the same
    // coordinate values.
    #[hegel::test(test_cases = 2000)]
    fn f64_point_written_by_crate_parses_to_same_values(tc: TestCase) {
        let finite = || gs::floats::<f64>().allow_nan(false).allow_infinity(false);
        let coord = crate::types::Coord {
            x: tc.draw(finite()),
            y: tc.draw(finite()),
            z: Some(tc.draw(finite())),
            m: Some(tc.draw(finite())),
        };
        let text = crate::Wkt::Point(crate::types::Point::from_coord(coord)).to_string();
        let crate::Wkt::Point(point) = crate::Wkt::<f64>::from_str(&text).unwrap() else {
            panic!("not a point: {text:?}");
        };
        let parsed = point.coord.unwrap();
        let bits = |c: crate::types::Coord<f64>| {
            (
                c.x.to_bits(),
                c.y.to_bits(),
                c.z.map(f64::to_bits),
                c.m.map(f64::to_bits),
            )
        };
        assert_eq!(bits(parsed), bits(coord), "text: {text:?}");
    }

    #[hegel::test(test_cases = 2000)]
    fn f32_point_written_by_crate_parses_to_same_values(tc: TestCase) {
        let finite = || gs::floats::<f32>().allow_nan(false).allow_infinity(false);
        let coord = crate::types::Coord {
            x: tc.draw(finite()),
            y: tc.draw(finite()),
            z: Some(tc.draw(finite())),
            m: Some(tc.draw(finite())),
        };
        let text = crate::Wkt::Point(crate::types::Point::from_coord(coord)).to_string();
        let crate::Wkt::Point(point) = crate::Wkt::<f32>::from_str(&text).unwrap() else {
            panic!("not a point: {text:?}");
        };
        let parsed = point.coord.unwrap();
        let bits = |c: crate::types::Coord<f32>| {
            (
                c.x.to_bits(),
                c.y.to_bits(),
                c.z.map(f32::to_bits),
                c.m.map(f32::to_bits),
            )
        };
        assert_eq!(bits(parsed), bits(coord), "text: {text:?}");
    }
}
