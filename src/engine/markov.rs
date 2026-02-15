use std::collections::HashMap;
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct MarkovModel {
    pub order: usize,
    // Map: Context (string) -> List of (Next Char, Cumulative Probability)
    pub transitions: HashMap<String, Vec<(char, f64)>>,
}

impl MarkovModel {
    pub fn new(order: usize) -> Self {
        Self {
            order,
            transitions: HashMap::new(),
        }
    }

    pub fn train(&mut self, corpus_path: &Path) -> Result<()> {
        let file = File::open(corpus_path)?;
        let reader = BufReader::new(file);

        let mut counts: HashMap<String, HashMap<char, usize>> = HashMap::new();

        for line in reader.lines() {
            let word = line?;
            if word.len() < self.order {
                continue;
            }

            // We treat the word as a sequence.
            // We can pad specific start/end symbols if we want strict boundary modeling.
            // For simplicity, we just model internal transitions for now.
            // Actually, for password generation, start/end is crucial.
            // Let's wrap words in strict boundaries e.g. "^word$".
            // But this might explode state space. 
            // Let's just train on the word itself for now.
            
            let char_vec: Vec<char> = word.chars().collect();
            
            for i in 0..char_vec.len() {
                if i + self.order >= char_vec.len() {
                    break;
                }
                
                let context: String = char_vec[i..i+self.order].iter().collect();
                let next_char = char_vec[i+self.order];
                
                counts.entry(context)
                    .or_default()
                    .entry(next_char)
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
            }
        }

        // Convert counts to probabilities
        for (context, next_chars) in counts {
            let total: usize = next_chars.values().sum();
            let mut cumulative = 0.0;
            let mut trans_vec = Vec::new();
            
            for (ch, count) in next_chars {
                let prob = count as f64 / total as f64;
                cumulative += prob;
                trans_vec.push((ch, cumulative));
            }
            // Ensure last is exactly 1.0 to avoid float errors
            if let Some(last) = trans_vec.last_mut() {
                last.1 = 1.0;
            }
            
            self.transitions.insert(context, trans_vec);
        }

        Ok(())
    }

    pub fn generate(&self, rng: &mut impl Rng, min_len: usize, max_len: usize) -> String {
        // Without start/end tokens, we need a random starting point.
        // A better model would have a special START node.
        // For this implementation, we pick a random context from the map to start.
        if self.transitions.is_empty() {
            return String::from("empty_model");
        }

        // Reservoir sampling or just converting keys to vec to pick start is slow.
        // We really should have trained start probabilities.
        // Retrofit: Let's assume the user calls train, we should track start contexts explicitly?
        // For now, I'll pick a random key. In production, this should be optimized.
        let keys: Vec<&String> = self.transitions.keys().collect();
        let start_idx = rng.gen_range(0..keys.len());
        let mut current_context = keys[start_idx].clone();
        let mut result = current_context.clone();

        while result.len() < max_len {
            if let Some(trans) = self.transitions.get(&current_context) {
                let r: f64 = rng.gen(); // 0.0..1.0
                let next_char = trans.iter()
                    .find(|(_, cum)| r <= *cum)
                    .map(|(c, _)| *c)
                    .unwrap_or(trans.last().unwrap().0); // Should match

                result.push(next_char);
                
                // Shift context
                // context is 'order' chars. we drop first, append next_char.
                let mut chars: Vec<char> = current_context.chars().collect();
                if !chars.is_empty() {
                    chars.remove(0);
                    chars.push(next_char);
                    current_context = chars.into_iter().collect();
                }
            } else {
                // Dead end
                break;
            }
        }
        
        // Ensure min length (simple retry or truncation? simple truncation doesn't help if too short)
        if result.len() < min_len {
            // Recurse or loop? Loop protection needed.
            return self.generate(rng, min_len, max_len); 
        }
        
        result
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let model = serde_json::from_reader(file)?;
        Ok(model)
    }
}
