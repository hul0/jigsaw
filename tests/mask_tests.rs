use jigsaw::engine::mask::{Mask, Charset};
use std::str::FromStr;

#[test]
fn test_integration_numeric_mask() {
    let mask = Mask::from_str("?d").unwrap();
    let results: Vec<Vec<u8>> = mask.iter().collect();
    assert_eq!(results.len(), 10);
    assert_eq!(results[0], b"0");
    assert_eq!(results[9], b"9");
}

#[test]
fn test_integration_mixed_mask() {
    // ?d?l = 10 * 26 = 260
    let mask = Mask::from_str("?d?l").unwrap();
    assert_eq!(mask.search_space_size(), 260);
    
    let results: Vec<Vec<u8>> = mask.iter().collect();
    assert_eq!(results.len(), 260);
    assert_eq!(results[0], b"0a");
    assert_eq!(results[25], b"0z");
    assert_eq!(results[26], b"1a");
}

#[test]
fn test_integration_literals() {
    let mask = Mask::from_str("pass?d").unwrap();
    let results: Vec<Vec<u8>> = mask.iter().collect();
    assert_eq!(results.len(), 10);
    assert_eq!(results[0], b"pass0");
    assert_eq!(results[9], b"pass9");
}

#[test]
fn test_integration_special_chars() {
    let mask = Mask::from_str("?s").unwrap();
    let results: Vec<Vec<u8>> = mask.iter().collect();
    // Special chars: !@#$%^&*()-_=+[]{};:'",.<>/?\|`~
    // Length is 32 (based on standard US keyboard implementation in mask.rs)
    assert_eq!(results.len(), 32);
}

#[test]
fn test_empty_mask() {
    // Empty mask should produce one empty result or nothing?
    // My implementation: 
    // FromStr "" -> components empty.
    // MaskIterator new -> done = false (if is_empty logic is tricky).
    // Let's check implementation behavior:
    // MaskIterator::new: if mask.components.is_empty(), done = false (wait, is_empty && !is_empty is false). 
    // Logic: let is_empty = mask.components.is_empty();
    // done: is_empty && !mask.components.is_empty() -> false.
    // So done is false.
    // next(): 
    // construct candidate (empty).
    // increment logic: i starts at 0. while i > 0 -> loop doesn't run.
    // !incremented -> true.
    // done = true.
    // return Some(candidate) (empty vec).
    // So it yields 1 empty string. This is chemically correct (identity element).
    
    let mask = Mask::from_str("").unwrap();
    let results: Vec<Vec<u8>> = mask.iter().collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], Vec::<u8>::new());
}
