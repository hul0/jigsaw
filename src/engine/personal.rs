use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use anyhow::Result;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Profile {
    #[serde(default)]
    pub first_names: Vec<String>,
    #[serde(default)]
    pub last_names: Vec<String>,
    #[serde(default)]
    pub partners: Vec<String>, // Wife/Husband/Partner
    #[serde(default)]
    pub kids: Vec<String>,
    #[serde(default)]
    pub pets: Vec<String>,
    
    // Expanded Categories
    #[serde(default)]
    pub company: Vec<String>,
    #[serde(default)]
    pub school: Vec<String>,
    #[serde(default)]
    pub city: Vec<String>,
    #[serde(default)]
    pub sports: Vec<String>, // Teams, Players
    #[serde(default)]
    pub music: Vec<String>, // Bands, Artists
    #[serde(default)]
    pub usernames: Vec<String>, // Online handles

    #[serde(default)]
    pub dates: Vec<String>, // Years (1990), MMDD, etc.
    #[serde(default)]
    pub keywords: Vec<String>, // Colors, Brands, Cities, etc.
    #[serde(default)]
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
        self.iter_candidates(|s| {
            candidates.insert(s);
            false // Continue
        });
        candidates.into_iter().map(|s| s.into_bytes()).collect()
    }

    pub fn check_password(&self, target: &str) -> bool {
        let mut found = false;
        self.iter_candidates(|s| {
            if s == target {
                found = true;
                return true; // Stop
            }
            false // Continue
        });
        found
    }

    fn iter_candidates<F>(&self, mut callback: F) 
    where F: FnMut(String) -> bool 
    {
        // 1. Gather all text inputs
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
        
        let mut dates_expanded = Vec::new();

        // Pre-process dates: Combine separate components if found
        let mut combined_dates = self.dates.clone();
        
        let years: Vec<&String> = self.dates.iter().filter(|d| d.len() == 4 && (d.starts_with("19") || d.starts_with("20"))).collect();
        let mmdds: Vec<&String> = self.dates.iter().filter(|d| d.len() == 4 && !years.contains(d)).collect();
        
        for y in &years {
            for md in &mmdds {
                combined_dates.push(format!("{}{}", md, y)); // MMDDYYYY or DDMMYYYY
                combined_dates.push(format!("{}{}", y, md)); // YYYYMMDD
            }
        }

        for date in &combined_dates {
            suffixes.push(date.clone());
            dates_expanded.push(date.clone());

            // Deep Date Logic: 
            // 4-digit Year: 2007 -> 07, 7, 007
            if date.len() == 4 && (date.starts_with("19") || date.starts_with("20")) {
                let short_year: String = date.chars().skip(2).collect(); // "07"
                suffixes.push(short_year.clone());
                dates_expanded.push(short_year.clone());
                
                let last_3: String = date.chars().skip(1).collect(); // "007"
                suffixes.push(last_3.clone());

                if let Ok(num) = short_year.parse::<u32>() {
                     suffixes.push(num.to_string()); // "7"
                }
            } 
            // 8-digit Date: 01022007 (DDMMYYYY or MMDDYYYY)
            else if date.len() == 8 && date.chars().all(char::is_numeric) {
                 // Component extraction
                 let p1 = &date[0..2]; // 01
                 let p2 = &date[2..4]; // 02
                 let year = &date[4..8]; // 2007
                 
                 // Basic Components
                 suffixes.push(year.to_string());
                 suffixes.push(format!("{}{}", p1, p2)); // 0102
                 suffixes.push(format!("{}{}", p2, p1)); // 0201
                 
                 // Year components
                 let short_year: String = year.chars().skip(2).collect(); // "07"
                 suffixes.push(short_year.clone());
                 
                 // Stripped Zeros (1, 2)
                 let p1_n = p1.parse::<u32>().unwrap_or(0);
                 let p2_n = p2.parse::<u32>().unwrap_or(0);
                 let p1_s = p1_n.to_string();
                 let p2_s = p2_n.to_string();
                 
                 suffixes.push(p1_s.clone());
                 suffixes.push(p2_s.clone());
                 
                 // Permutations with Year
                 // 122007 (p1_s + p2_s + year)
                 suffixes.push(format!("{}{}{}", p1_s, p2_s, year));
                 suffixes.push(format!("{}{}{}", p2_s, p1_s, year));
                 
                 // 122007 (p1_s + p2_s + short_year) -> 1207
                 suffixes.push(format!("{}{}{}", p1_s, p2_s, short_year));
                 suffixes.push(format!("{}{}{}", p2_s, p1_s, short_year));

                 // 122007 (p1 + p2 + year) -> 01022007 (Base)
                 
                 // Just Year + Month or Day
                 suffixes.push(format!("{}{}", p1_s, year));
                 suffixes.push(format!("{}{}", p2_s, year));
                 
                 dates_expanded.push(year.to_string());

                 // Special case for Rupam#122007: p1=01, p2=02, year=2007 -> 122007 (matches p1_s matches 1, p2_s matches 2)
                 // But wait, "12" in Rupam's file is separate.
                 // "122007" is "1" + "2" + "2007". 
                 // So p1_s + p2_s + year covers it.
            }
        }

        // 3. Separators & Specials
        let separators = ["", "_", ".", "-", "@", "#", "!", "$", "&", "+", "="];
        let specials = ["!", "@", "#", "$", "*", "?", "123", "007", "!!", "1!", "123!"];
        let complex_suffixes = ["123", "1234", "12345", "007", "69", "420", "01", "00"];

        // Helper: Basic Case Variations + Reversal
        for word in &all_words {
            if word.is_empty() { continue; }
            
            let reversed: String = word.chars().rev().collect();
            let mut base_forms = vec![word.to_string(), reversed];
            
            let mut variants = Vec::new();
            for base in base_forms {
                 variants.push(base.to_lowercase());
                 variants.push(base.to_uppercase());
                 variants.push(to_title_case(&base.to_lowercase()));
            }

            let mut leet_versions = Vec::new();
            for v in &variants {
                leet_versions.extend(generate_leet(&v.to_lowercase()));
            }
            
            let mut word_forms = variants.clone();
            word_forms.extend(leet_versions);

            for form in word_forms {
                // Base
                if callback(form.clone()) { return; }
                
                // Base + Suffix/Special patterns
                // Pattern: Word + Sep + Suffix
                for suffix in &suffixes {
                    for sep in separators {
                        if callback(format!("{}{}{}", form, sep, suffix)) { return; }
                    }
                    // Pattern: Suffix + Sep + Word
                    for sep in separators {
                        if callback(format!("{}{}{}", suffix, sep, form)) { return; }
                    }
                    // Pattern: Word + Suffix + Special
                    for special in specials {
                         if callback(format!("{}{}{}", form, suffix, special)) { return; }
                    }
                    // Pattern: Sandwich
                    for special in specials {
                        if callback(format!("{}{}{}{}", special, form, suffix, special)) { return; }
                    }
                    // Pattern: Complex Sandwich
                    for sep in ["@", "#"] {
                         if callback(format!("{}{}{}{}", sep, form, sep, suffix)) { return; }
                    }
                    
                    // NEW PATTERNS
                    // Pattern: Name + Suffix + Suffix (Double suffix like Name1990_123)
                     for extra in &complex_suffixes {
                        for sep in separators {
                            if callback(format!("{}{}{}{}{}", form, sep, suffix, sep, extra)) { return; }
                        }
                    }
                }
                
                // Specials Only
                for special in specials {
                    if callback(format!("{}{}", form, special)) { return; }
                    if callback(format!("{}{}", special, form)) { return; }
                }
                
                // NEW: Complex Sandwiches
                if callback(format!("xX{}Xx", form)) { return; }
                if callback(format!("_{}_", form)) { return; }
                if callback(format!("- {} -", form)) { return; }
            }
        }

        // 4. Name/Word Combinations & Family
        let mut left_sides = Vec::new();
        left_sides.extend(self.first_names.iter());
        left_sides.extend(self.usernames.iter());
        
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
             // Explicit Family Combinations
             for p in &self.partners {
                 for n in &self.first_names {
                     // Name + Partner
                      for sep in ["&", "+", "and", "And"] {
                          if callback(format!("{}{}{}", n, sep, p)) { return; }
                          if callback(format!("{}{}{}", p, sep, n)) { return; }
                      }
                 }
             }
        }

        for left in &left_sides {
            for right in &right_sides {
                 let l_variants = vec![left.to_lowercase(), to_title_case(&left.to_lowercase())];
                 let r_variants = vec![right.to_lowercase(), to_title_case(&right.to_lowercase())];
                 
                 for l in &l_variants {
                     for r in &r_variants {
                         for sep in separators {
                             if callback(format!("{}{}{}", l, sep, r)) { return; }
                             
                             // Pattern: Name_Surname123
                             for suffix in &suffixes {
                                 if callback(format!("{}{}{}{}", l, sep, r, suffix)) { return; }
                                 // Pattern: Surname_Name123
                                 if callback(format!("{}{}{}{}", r, sep, l, suffix)) { return; }
                             }
                         }
                         // With suffix? (JohnDoe1990)
                         for suffix in &suffixes {
                             if callback(format!("{}{}{}", l, r, suffix)) { return; }
                             // JohnDoe_1990
                             if callback(format!("{}{}_{}", l, r, suffix)) { return; }
                             // John.Doe.1990
                             if callback(format!("{}.{}.{}", l, r, suffix)) { return; }
                             // John#Doe#1990
                             if callback(format!("{}#{}{}", l, r, suffix)) { return; }
                             if callback(format!("{}#{}#{}", l, r, suffix)) { return; }
                         }
                         
                         // Complex Pattern: Name@Partner#Year
                         for suffix in &suffixes {
                             if callback(format!("{}@{}#{}", l, r, suffix)) { return; }
                         }
                     }
                 }
            }
        }
        
        // 5. Date Permutations En Masse
        // If we have "1990", let's generate "90", "2000" -> "00" (already done in step 2 logic but let's be sure)
        // Also Year combinations? 19901991? Maybe too much.
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
