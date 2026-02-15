use anyhow::{anyhow, Result};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    NoOp,               // :
    Append(u8),         // $x
    Prepend(u8),        // ^x
    Reverse,            // r
    Upper,              // u
    Lower,              // l
    ToggleCase,         // t (toggle all)
    Duplicate,          // d
    Reflect,            // f (duplicate reversed, e.g. abc -> abccba)
    RotateLeft,         // {
    RotateRight,        // }
}

impl Rule {
    pub fn apply(&self, candidate: &mut Vec<u8>) {
        match self {
            Rule::NoOp => {},
            Rule::Append(c) => candidate.push(*c),
            Rule::Prepend(c) => candidate.insert(0, *c),
            Rule::Reverse => candidate.reverse(),
            Rule::Upper => {
                for b in candidate.iter_mut() {
                    if b.is_ascii_lowercase() {
                        *b = b.to_ascii_uppercase();
                    }
                }
            },
            Rule::Lower => {
                for b in candidate.iter_mut() {
                    if b.is_ascii_uppercase() {
                        *b = b.to_ascii_lowercase();
                    }
                }
            },
            Rule::ToggleCase => {
                for b in candidate.iter_mut() {
                    if b.is_ascii_lowercase() {
                        *b = b.to_ascii_uppercase();
                    } else if b.is_ascii_uppercase() {
                        *b = b.to_ascii_lowercase();
                    }
                }
            },
            Rule::Duplicate => {
                let len = candidate.len();
                candidate.reserve(len);
                // Safety: we are copying valid bytes currently in the vector to the end of it.
                // We must avoid holding a reference to candidate while pushing to it.
                // Naive approach:
                let copy = candidate.clone();
                candidate.extend_from_slice(&copy);
            },
            Rule::Reflect => {
                let mut copy = candidate.clone();
                copy.reverse();
                candidate.extend_from_slice(&copy);
            },
            Rule::RotateLeft => {
                if !candidate.is_empty() {
                    candidate.rotate_left(1);
                }
            },
            Rule::RotateRight => {
                if !candidate.is_empty() {
                    candidate.rotate_right(1);
                }
            },
        }
    }
}

pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }

    pub fn apply(&self, candidate: &mut Vec<u8>) {
        for rule in &self.rules {
            rule.apply(candidate);
        }
    }
}

impl FromStr for RuleSet {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut rules = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                ' ' => continue, // Ignore spaces for robust parsing? Hashcat rules are usually stricter, but spaces allowed in checking is nice. 
                                 // Actually hashcat rules in a file are one per line, but ' ' is a specific rule (insert space)? 
                                 // No, in hashcat ' ' is just a character. Rules are typically compact.
                                 // However, I will allow spaces as separators for readability in this engine for now, 
                                 // unless it conflicts with a literal space arg.
                                 // Wait, `$` takes a char. If that char is space, we need to handle it.
                                 // Let's assume strict parsing for now: spaces are ignored unless they are arguments.
                ':' => rules.push(Rule::NoOp),
                'r' => rules.push(Rule::Reverse),
                'u' => rules.push(Rule::Upper),
                'l' => rules.push(Rule::Lower),
                't' => rules.push(Rule::ToggleCase),
                'd' => rules.push(Rule::Duplicate),
                'f' => rules.push(Rule::Reflect),
                '{' => rules.push(Rule::RotateLeft),
                '}' => rules.push(Rule::RotateRight),
                '$' => {
                    if let Some(arg) = chars.next() {
                        // Handle strict ASCII for now as u8
                        if arg.is_ascii() {
                            rules.push(Rule::Append(arg as u8));
                        } else {
                            return Err(anyhow!("Rule $ argument must be ASCII"));
                        }
                    } else {
                        return Err(anyhow!("Rule $ requires an argument"));
                    }
                },
                '^' => {
                    if let Some(arg) = chars.next() {
                        if arg.is_ascii() {
                            rules.push(Rule::Prepend(arg as u8));
                        } else {
                            return Err(anyhow!("Rule ^ argument must be ASCII"));
                        }
                    } else {
                        return Err(anyhow!("Rule ^ requires an argument"));
                    }
                },
                _ => return Err(anyhow!("Unknown rule: {}", c)),
            }
        }
        Ok(RuleSet { rules })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_rule(rule: Rule, input: &str) -> String {
        let mut buf = input.as_bytes().to_vec();
        rule.apply(&mut buf);
        String::from_utf8(buf).unwrap()
    }
    
    fn apply_ruleset(rules: &str, input: &str) -> String {
        let rs = RuleSet::from_str(rules).unwrap();
        let mut buf = input.as_bytes().to_vec();
        rs.apply(&mut buf);
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn test_noop() {
        assert_eq!(apply_rule(Rule::NoOp, "abc"), "abc");
    }

    #[test]
    fn test_reverse() {
        assert_eq!(apply_rule(Rule::Reverse, "abc"), "cba");
    }

    #[test]
    fn test_append() {
        assert_eq!(apply_rule(Rule::Append(b'!'), "abc"), "abc!");
    }

    #[test]
    fn test_prepend() {
        assert_eq!(apply_rule(Rule::Prepend(b'X'), "abc"), "Xabc");
    }

    #[test]
    fn test_casing() {
        assert_eq!(apply_rule(Rule::Upper, "Abc"), "ABC");
        assert_eq!(apply_rule(Rule::Lower, "Abc"), "abc");
        assert_eq!(apply_rule(Rule::ToggleCase, "aBc"), "AbC");
    }

    #[test]
    fn test_duplicate() {
        assert_eq!(apply_rule(Rule::Duplicate, "abc"), "abcabc");
    }
    
    #[test]
    fn test_reflect() {
        assert_eq!(apply_rule(Rule::Reflect, "abc"), "abccba");
    }
    
    #[test]
    fn test_rotate() {
        assert_eq!(apply_rule(Rule::RotateLeft, "abc"), "bca");
        assert_eq!(apply_rule(Rule::RotateRight, "abc"), "cab");
    }

    #[test]
    fn test_parsing() {
        let rs = RuleSet::from_str(":r$!").unwrap();
        assert_eq!(rs.rules.len(), 3);
        assert_eq!(rs.rules[0], Rule::NoOp);
        assert_eq!(rs.rules[1], Rule::Reverse);
        assert_eq!(rs.rules[2], Rule::Append(b'!'));
    }

    #[test]
    fn test_chain() {
        // Reverse "abc" -> "cba"
        // Upper "cba" -> "CBA"
        // Append ! -> "CBA!"
        assert_eq!(apply_ruleset("ru$!", "abc"), "CBA!");
    }
}
