use std::str::FromStr;
use anyhow::{anyhow, Result};
use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Charset {
    Lower,
    Upper,
    Digit,
    Special,
    Literal(u8),
    Custom(Vec<u8>),
}

impl Charset {
    pub fn chars(&self) -> &[u8] {
        match self {
            Charset::Lower => b"abcdefghijklmnopqrstuvwxyz",
            Charset::Upper => b"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            Charset::Digit => b"0123456789",
            Charset::Special => b"!@#$%^&*()-_=+[]{};:'\",.<>/?\\|`~",
            Charset::Literal(c) => std::slice::from_ref(c),
            Charset::Custom(chars) => chars,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mask {
    pub components: Vec<Charset>,
}


impl Mask {
    pub fn new(components: Vec<Charset>) -> Self {
        Self { components }
    }

    /// Calculate the total size of the search space for this mask
    pub fn search_space_size(&self) -> u128 {
        self.components.iter().map(|c| c.chars().len() as u128).product()
    }

    pub fn iter(&self) -> MaskIterator<'_> {
        MaskIterator::new(self)
    }

    pub fn nth_candidate(&self, mut index: u128) -> Option<Vec<u8>> {
        let total = self.search_space_size();
        if index >= total {
            return None;
        }

        let mut candidate = Vec::with_capacity(self.components.len());
        
        let mut divisors = Vec::with_capacity(self.components.len());
        let mut current_div = total;
        
        for component in &self.components {
            let len = component.chars().len() as u128;
            current_div /= len;
            divisors.push((current_div, len));
        }
        
        for (i, component) in self.components.iter().enumerate() {
            let (divisor, len) = divisors[i];
            let chars = component.chars();
            let char_idx = (index / divisor) % len;
            candidate.push(chars[char_idx as usize]);
        }
        
        Some(candidate)
    }

    pub fn par_iter(&self) -> rayon::iter::Map<rayon::range::Iter<u128>, impl Fn(u128) -> Vec<u8> + '_> {
        use rayon::prelude::*;
        let size = self.search_space_size();
        (0..size).into_par_iter().map(move |i| self.nth_candidate(i).expect("Index within bounds"))
    }
}

impl FromStr for Mask {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut components = Vec::new();
        let bytes = s.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] == b'?' {
                if i + 1 >= bytes.len() {
                    return Err(anyhow!("Invalid mask: ends with ?"));
                }
                match bytes[i + 1] {
                    b'l' => components.push(Charset::Lower),
                    b'u' => components.push(Charset::Upper),
                    b'd' => components.push(Charset::Digit),
                    b's' => components.push(Charset::Special),
                    b'?' => components.push(Charset::Literal(b'?')),
                    c => return Err(anyhow!("Unknown mask pattern: ?{}", c as char)),
                }
                i += 2;
            } else {
                components.push(Charset::Literal(bytes[i]));
                i += 1;
            }
        }

        Ok(Mask { components })
    }
}

pub struct MaskIterator<'a> {
    mask: &'a Mask,
    indices: Vec<usize>,
    done: bool,
}

impl<'a> MaskIterator<'a> {
    pub fn new(mask: &'a Mask) -> Self {
        let is_empty = mask.components.is_empty();
        Self {
            mask,
            indices: vec![0; mask.components.len()],
            done: is_empty && !mask.components.is_empty(),
        }
    }
}

impl<'a> Iterator for MaskIterator<'a> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let mut candidate = Vec::with_capacity(self.mask.components.len());
        for (i, component) in self.mask.components.iter().enumerate() {
            let chars = component.chars();
            if let Some(&byte) = chars.get(self.indices[i]) {
                candidate.push(byte);
            }
        }

        let mut i = self.indices.len();
        let mut incremented = false;

        while i > 0 {
            i -= 1;
            let max_len = self.mask.components[i].chars().len();
            if self.indices[i] + 1 < max_len {
                self.indices[i] += 1;
                incremented = true;
                break;
            } else {
                self.indices[i] = 0;
            }
        }

        if !incremented {
            self.done = true;
            if self.mask.components.is_empty() {
                return Some(candidate);
            }
        }
        
        Some(candidate)
    }
}

impl IntoIterator for &Mask {
    type Item = Vec<u8>;
    type IntoIter = MaskIterator<'static>; 
    fn into_iter(self) -> Self::IntoIter {
        panic!("Use Mask::iter(&self) instead");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nth_candidate() {
        let mask = Mask::from_str("?d?d").unwrap();
        assert_eq!(mask.nth_candidate(42).unwrap(), b"42");
        assert_eq!(mask.nth_candidate(0).unwrap(), b"00");
        assert_eq!(mask.nth_candidate(99).unwrap(), b"99");
        assert!(mask.nth_candidate(100).is_none());
    }

    #[test]
    fn test_mask_parsing() {
        let mask = Mask::from_str("?d").unwrap();
        assert_eq!(mask.components.len(), 1);
        match mask.components[0] {
            Charset::Digit => (),
            _ => panic!("Expected Digit"),
        }

        let mask = Mask::from_str("abc?l").unwrap();
        assert_eq!(mask.components.len(), 4);
    }

    #[test]
    fn test_generation_small() {
        let mask = Mask::from_str("?d").unwrap();
        let results: Vec<Vec<u8>> = mask.iter().collect();
        assert_eq!(results.len(), 10);
        assert_eq!(results[0], b"0");
        assert_eq!(results[9], b"9");
    }

    #[test]
    fn test_generation_odometer() {
        let mask = Mask::from_str("?d?l").unwrap();
        let count = mask.iter().count();
        assert_eq!(count, 260);
    }
    
    #[test]
    fn test_literal_handling() {
        let mask = Mask::from_str("a?d").unwrap();
        let results: Vec<Vec<u8>> = mask.iter().collect();
        assert_eq!(results.len(), 10);
        assert_eq!(results[0], b"a0");
        assert_eq!(results[9], b"a9");
    }
}
