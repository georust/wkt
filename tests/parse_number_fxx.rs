//! Parses each number in the parse-number-fxx-test-data corpus as the X
//! coordinate of a point, and compares the f64 and f32 values with the bits
//! in the corpus.
//!
//! The corpus is approximately 260 MB, so it is not in this repository. To
//! run the test:
//!
//! ```sh
//! git clone --depth 1 https://github.com/nigeltao/parse-number-fxx-test-data
//! PARSE_NUMBER_FXX_DATA=parse-number-fxx-test-data/data \
//!     cargo test --release --test parse_number_fxx -- --ignored
//! ```
//!
//! Each line of a corpus file has four fields: the f16, f32 and f64 bits in
//! hexadecimal, then the decimal text.

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use wkt::Wkt;

const DATA_VAR: &str = "PARSE_NUMBER_FXX_DATA";

// The number of mismatches to show before the test stops collecting them.
const MAX_REPORTED: usize = 20;

fn x_coordinate<T>(text: &str) -> Option<T>
where
    T: wkt::WktNum + FromStr + Default + 'static,
{
    match Wkt::<T>::from_str(&format!("POINT({text} 0)")) {
        Ok(Wkt::Point(point)) => point.coord().map(|coord| coord.x),
        _ => None,
    }
}

#[test]
#[ignore = "needs the parse-number-fxx-test-data corpus; see the module documentation"]
fn corpus_numbers_parse_to_expected_bits() {
    let dir = PathBuf::from(std::env::var_os(DATA_VAR).unwrap_or_else(|| {
        panic!("set {DATA_VAR} to the data directory of parse-number-fxx-test-data")
    }));
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no .txt files in {}", dir.display());

    let mut checked = 0usize;
    let mut mismatches = Vec::new();
    let mut mismatch_count = 0usize;
    for file in &files {
        let contents = fs::read_to_string(file).unwrap();
        for (index, line) in contents.lines().enumerate() {
            let fields: Vec<&str> = line.split(' ').collect();
            let [_, f32_hex, f64_hex, text] = fields[..] else {
                panic!("{}:{}: expected 4 fields", file.display(), index + 1);
            };
            let expected_f64 = u64::from_str_radix(f64_hex, 16).unwrap();
            let expected_f32 = u32::from_str_radix(f32_hex, 16).unwrap();
            let actual_f64 = x_coordinate::<f64>(text).map(f64::to_bits);
            let actual_f32 = x_coordinate::<f32>(text).map(f32::to_bits);
            checked += 1;
            if actual_f64 != Some(expected_f64) || actual_f32 != Some(expected_f32) {
                mismatch_count += 1;
                if mismatches.len() < MAX_REPORTED {
                    mismatches.push(format!(
                        "{}:{}: {text:?}: f64 expected {expected_f64:016X}, got {actual_f64:016X?}; \
                         f32 expected {expected_f32:08X}, got {actual_f32:08X?}",
                        file.display(),
                        index + 1,
                    ));
                }
            }
        }
    }
    assert!(
        mismatch_count == 0,
        "{mismatch_count} of {checked} numbers did not match:\n{}",
        mismatches.join("\n")
    );
    eprintln!("{checked} numbers matched");
}
