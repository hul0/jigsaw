use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Profile {
    pub first_names: Vec<String>,
    pub last_names: Vec<String>,
    pub partners: Vec<String>, // Wife/Husband/Partner
    pub kids: Vec<String>,
    pub pets: Vec<String>,
    
    // Expanded Categories
    pub company: Vec<String>,
    pub school: Vec<String>,
    pub city: Vec<String>,
    pub sports: Vec<String>, // Teams, Players
    pub music: Vec<String>, // Bands, Artists
    pub usernames: Vec<String>, // Online handles

    pub dates: Vec<String>, // Years (1990), MMDD, etc.
    pub keywords: Vec<String>, // Colors, Brands, Cities, etc.
    pub numbers: Vec<String>, // Phone, Zip, Fav
}

impl Profile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let profile = serde_json::from_reader(reader)?;
        Ok(profile)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    pub fn generate(&self) -> Vec<Vec<u8>> {
        let mut candidates = HashSet::new();

        // 1. Gather all text inputs for basic mutations
        let mut all_words = Vec::new();
        all_words.extend(self.first_names.iter());
        all_words.extend(self.last_names.iter());
        all_words.extend(self.partners.iter());
        all_words.extend(self.kids.iter());
        all_words.extend(self.pets.iter());
        all_words.extend(self.company.iter());
        all_words.extend(self.school.iter());
        all_words.extend(self.city.iter());
        all_words.extend(self.sports.iter());
        all_words.extend(self.music.iter());
        all_words.extend(self.usernames.iter());
        all_words.extend(self.keywords.iter());

        // 2. Gather suffixes (Dates, Numbers) & Deep Date Parsing
        let mut suffixes = Vec::new();
        suffixes.extend(self.numbers.iter().cloned());
        
        for date in &self.dates {
            suffixes.push(date.clone());
            // Deep Date Logic: 2007 -> 07, 7, 007
            if date.len() == 4 && (date.starts_with("19") || date.starts_with("20")) {
                let short_year = date[2..].to_string(); // "07"
                suffixes.push(short_year.clone());
                
                // "7" (Last digit, if not 0, or just simplified)
                // Actually, let's just take the last digit if it's significant, 
                // but user asked for "7", "007".
                // "007" = last 3 digits? 
                let last_3 = date[1..].to_string(); // "007"
                suffixes.push(last_3);

                // "7" -> remove leading zeros from short year?
                if let Ok(num) = short_year.parse::<u32>() {
                     suffixes.push(num.to_string());
                }
            }
        }

        // 3. Separators & Specials
        let separators = ["", "_", ".", "-", "@", "#", "!", "$", "&"];
        let specials = ["!", "@", "#", "$", "*", "?", "123", "007"]; // Suffix/Prefix specials

        // Helper: Basic Case Variations + Reversal
        for word in &all_words {
            if word.is_empty() { continue; }
            
            // Reversal: John -> nhoj
            let reversed: String = word.chars().rev().collect();
            
            // `word` is &&String because all_words is Vec<&String> and we iterate &all_words.
            // So to get String, we use to_string().
            let mut base_forms = vec![word.to_string(), reversed];
            
            let mut variants = Vec::new();
            for base in base_forms {
                 variants.push(base.to_lowercase());
                 variants.push(base.to_uppercase());
                 variants.push(to_title_case(&base.to_lowercase()));
            }

            // Add Leet Variations
            let mut leet_versions = Vec::new();
            for v in &variants {
                leet_versions.extend(generate_leet(&v.to_lowercase()));
            }
            
            let mut word_forms = variants.clone();
            word_forms.extend(leet_versions);

            for form in word_forms {
                // Base: just the word
                candidates.insert(form.clone());
                
                // Pattern: Word + Sep + Suffix (Name@1990, Name#90, Name_123)
                for suffix in &suffixes {
                    for sep in separators {
                        candidates.insert(format!("{}{}{}", form, sep, suffix));
                    }
                    
                    // Pattern: Suffix + Sep + Word (1990Name, 123_Name)
                    for sep in separators {
                        candidates.insert(format!("{}{}{}", suffix, sep, form));
                    }
                    
                     // Pattern: Word + Suffix + Special (Name1990!)
                    for special in specials {
                         candidates.insert(format!("{}{}{}", form, suffix, special));
                    }
                    
                    // Pattern: Sandwich (Special + Word + Suffix + Special) (!Name1990!)
                    for special in specials {
                        candidates.insert(format!("{}{}{}{}", special, form, suffix, special));
                    }

                    // Pattern: Complex Sandwich (Special + Word + Sep + Suffix) (#Name@2020)
                    for sep in ["@", "#"] {
                         candidates.insert(format!("{}{}{}{}", sep, form, sep, suffix));
                    }
                }
                
                // Specials Only (Name!, Name#)
                for special in specials {
                    candidates.insert(format!("{}{}", form, special));
                    candidates.insert(format!("{}{}", special, form));
                }
            }
        }

        // 4. Name/Word Combinations (First+Last, Name+Keyword)
        let mut left_sides = Vec::new();
        left_sides.extend(self.first_names.iter());
        left_sides.extend(self.usernames.iter()); // Usernames often prefix things
        
        let mut right_sides = Vec::new();
        right_sides.extend(self.last_names.iter());
        right_sides.extend(self.keywords.iter());
        right_sides.extend(self.company.iter());
        right_sides.extend(self.school.iter());
        right_sides.extend(self.city.iter());
        right_sides.extend(self.sports.iter());
        right_sides.extend(self.music.iter());
        
        if !self.partners.is_empty() {
             right_sides.extend(self.partners.iter());
        }

        for left in &left_sides {
            for right in &right_sides {
                 // Add Reversed logic to combinations too? 
                 // Maybe overkill, but consistent. Let's stick to standard forms for combos for now to avoid explosion.
                 let l_variants = vec![left.to_lowercase(), to_title_case(&left.to_lowercase())];
                 let r_variants = vec![right.to_lowercase(), to_title_case(&right.to_lowercase())];
                 
                 for l in &l_variants {
                     for r in &r_variants {
                         for sep in separators {
                             candidates.insert(format!("{}{}{}", l, sep, r));
                         }
                         // With suffix? (JohnDoe1990)
                         for suffix in &suffixes {
                             candidates.insert(format!("{}{}{}", l, r, suffix));
                             candidates.insert(format!("{}{}_{}", l, r, suffix));
                         }
                     }
                 }
            }
        }

        candidates.into_iter().map(|s| s.into_bytes()).collect()
    }
}

fn to_title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn generate_leet(s: &str) -> Vec<String> {
    // Simple recursive leet generator? Or just apply common subs.
    // For performance and simplicity, let's just do a few fixed variations.
    // full leet: a->@, e->3, i->1, o->0, s->$, t->7
    // partial leet are harder to generate exhaustively without exploding count.
    
    let mut results = Vec::new();
    
    // 1. Full Leet
    let full_leet: String = s.chars().map(|c| match c {
        'a' | 'A' => '@',
        'e' | 'E' => '3',
        'i' | 'I' => '1',
        'o' | 'O' => '0',
        's' | 'S' => '$',
        't' | 'T' => '7',
        x => x,
    }).collect();
    
    if full_leet != s {
        results.push(full_leet);
    }
    
    // 2. Simple 's' -> '$' only (common)
    let s_leet: String = s.replace('s', "$").replace('S', "$");
    if s_leet != s {
        results.push(s_leet);
    }

    // 3. Append '123', '!'? (Actually handled by suffix logic, but "!" is not in suffix list by default)
    // Let's rely on explicit suffixes for now.

    results
}
