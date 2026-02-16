use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use anyhow::Result;

const CURRENT_YEAR: u32 = 2026;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Profile {
    #[serde(default)]
    pub first_names: Vec<String>,
    #[serde(default)]
    pub last_names: Vec<String>,
    #[serde(default)]
    pub partners: Vec<String>,
    #[serde(default)]
    pub kids: Vec<String>,
    #[serde(default)]
    pub pets: Vec<String>,

    #[serde(default)]
    pub company: Vec<String>,
    #[serde(default)]
    pub school: Vec<String>,
    #[serde(default)]
    pub city: Vec<String>,
    #[serde(default)]
    pub sports: Vec<String>,
    #[serde(default)]
    pub music: Vec<String>,
    #[serde(default)]
    pub usernames: Vec<String>,

    #[serde(default)]
    pub dates: Vec<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub numbers: Vec<String>,

    // NEW fields
    #[serde(default)]
    pub email: Vec<String>,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub maiden_name: Vec<String>,
    #[serde(default)]
    pub hobbies: Vec<String>,

    // Optional length filtering
    #[serde(default)]
    pub min_length: Option<usize>,
    #[serde(default)]
    pub max_length: Option<usize>,
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
            false
        });
        candidates.into_iter().map(|s| s.into_bytes()).collect()
    }

    pub fn check_password(&self, target: &str) -> bool {
        let mut found = false;
        self.iter_candidates(|s| {
            if s == target {
                found = true;
                return true;
            }
            false
        });
        found
    }

    fn iter_candidates<F>(&self, mut callback: F)
    where F: FnMut(String) -> bool
    {
        let min_len = self.min_length.unwrap_or(0);
        let max_len = self.max_length.unwrap_or(usize::MAX);

        macro_rules! emit {
            ($s:expr) => {{
                let s: String = $s;
                if s.len() >= min_len && s.len() <= max_len {
                    if callback(s) { return; }
                }
            }};
        }

        // ═══════════════════════════════════════════════════════
        // 1. GATHER ALL TEXT INPUTS
        // ═══════════════════════════════════════════════════════
        let mut all_words: Vec<String> = Vec::new();

        for field in [
            &self.first_names, &self.last_names, &self.partners,
            &self.kids, &self.pets, &self.company, &self.school,
            &self.city, &self.sports, &self.music, &self.keywords,
            &self.parents, &self.maiden_name, &self.hobbies,
        ] {
            all_words.extend(field.iter().cloned());
        }

        // Usernames: whole + decomposed parts
        for username in &self.usernames {
            all_words.push(username.clone());
            all_words.extend(decompose_username(username));
        }

        // Emails: decompose and add parts
        for email in &self.email {
            all_words.extend(decompose_email(email));
        }

        // Generate nicknames/truncations
        let base_words = all_words.clone();
        for word in &base_words {
            all_words.extend(generate_nicknames(word));
        }

        // Deduplicate
        all_words.sort();
        all_words.dedup();
        all_words.retain(|w| !w.is_empty());

        // ═══════════════════════════════════════════════════════
        // 2. SUFFIX EXPANSION (Dates, Numbers, Pins, Keyboard)
        // ═══════════════════════════════════════════════════════
        let mut suffixes: Vec<String> = Vec::new();

        // --- Numbers (raw + phone decomposition + reversed) ---
        for num in &self.numbers {
            suffixes.push(num.clone());
            let reversed: String = num.chars().rev().collect();
            if reversed != *num { suffixes.push(reversed); }
            suffixes.extend(decompose_phone(num));
        }

        // --- Date Expansion ---
        let mut dates_expanded: Vec<String> = Vec::new();
        let mut combined_dates = self.dates.clone();

        let years: Vec<&String> = self.dates.iter()
            .filter(|d| d.len() == 4 && (d.starts_with("19") || d.starts_with("20")))
            .collect();
        let mmdds: Vec<&String> = self.dates.iter()
            .filter(|d| d.len() == 4 && !years.contains(d))
            .collect();

        // Combine years with MMDDs
        for y in &years {
            for md in &mmdds {
                combined_dates.push(format!("{}{}", md, y));
                combined_dates.push(format!("{}{}", y, md));
            }
        }

        // Year range ±3
        for y in &years {
            if let Ok(year_num) = y.parse::<i32>() {
                for delta in -3..=3i32 {
                    if delta == 0 { continue; }
                    let nearby = (year_num + delta).to_string();
                    if !combined_dates.contains(&nearby) {
                        combined_dates.push(nearby);
                    }
                }
            }
        }

        for date in &combined_dates {
            suffixes.push(date.clone());
            dates_expanded.push(date.clone());

            // 4-digit Year
            if date.len() == 4 && (date.starts_with("19") || date.starts_with("20")) {
                let short_year: String = date.chars().skip(2).collect();
                suffixes.push(short_year.clone());
                dates_expanded.push(short_year.clone());

                let last_3: String = date.chars().skip(1).collect();
                suffixes.push(last_3);

                if let Ok(num) = short_year.parse::<u32>() {
                    suffixes.push(num.to_string());
                }

                // Informal: 2k7, '07
                if let Ok(year_num) = date.parse::<u32>() {
                    if year_num >= 2000 {
                        suffixes.push(format!("{}k{}", year_num / 1000, year_num % 1000));
                    }
                    suffixes.push(format!("'{}", &short_year));

                    // Age derivation
                    if year_num < CURRENT_YEAR && year_num > 1920 {
                        let age = CURRENT_YEAR - year_num;
                        suffixes.push(age.to_string());
                    }
                }
            }
            // 4-digit non-year (MMDD) — also generate DDMM flip
            else if date.len() == 4 && date.chars().all(char::is_numeric)
                && !years.iter().any(|y| *y == date)
            {
                let p1 = &date[0..2];
                let p2 = &date[2..4];
                let flipped = format!("{}{}", p2, p1);
                if flipped != *date {
                    suffixes.push(flipped.clone());
                    dates_expanded.push(flipped);
                }
                // Date with separators
                for sep in ["/", "-", "."] {
                    suffixes.push(format!("{}{}{}", p1, sep, p2));
                    suffixes.push(format!("{}{}{}", p2, sep, p1));
                }
                // Month name variants
                if let Ok(m) = p1.parse::<u32>() {
                    if let Some((short, long)) = month_name(m) {
                        suffixes.push(format!("{}{}", short, p2));
                        suffixes.push(format!("{}{}", long, p2));
                    }
                }
                if let Ok(m) = p2.parse::<u32>() {
                    if let Some((short, long)) = month_name(m) {
                        suffixes.push(format!("{}{}", short, p1));
                        suffixes.push(format!("{}{}", long, p1));
                    }
                }
            }
            // 8-digit date
            else if date.len() == 8 && date.chars().all(char::is_numeric) {
                let p1 = &date[0..2];
                let p2 = &date[2..4];
                let year = &date[4..8];

                suffixes.push(year.to_string());
                suffixes.push(format!("{}{}", p1, p2));
                suffixes.push(format!("{}{}", p2, p1));

                let short_year: String = year.chars().skip(2).collect();
                suffixes.push(short_year.clone());

                let p1_n = p1.parse::<u32>().unwrap_or(0);
                let p2_n = p2.parse::<u32>().unwrap_or(0);
                let p1_s = p1_n.to_string();
                let p2_s = p2_n.to_string();

                suffixes.push(p1_s.clone());
                suffixes.push(p2_s.clone());

                suffixes.push(format!("{}{}{}", p1_s, p2_s, year));
                suffixes.push(format!("{}{}{}", p2_s, p1_s, year));
                suffixes.push(format!("{}{}{}", p1_s, p2_s, short_year));
                suffixes.push(format!("{}{}{}", p2_s, p1_s, short_year));
                suffixes.push(format!("{}{}", p1_s, year));
                suffixes.push(format!("{}{}", p2_s, year));

                // Date with separators
                for sep in ["/", "-", "."] {
                    suffixes.push(format!("{}{}{}{}{}", p1, sep, p2, sep, year));
                    suffixes.push(format!("{}{}{}{}{}", p2, sep, p1, sep, year));
                    suffixes.push(format!("{}{}{}{}{}", p1, sep, p2, sep, &short_year));
                }

                // Month names
                if let Ok(m) = p1.parse::<u32>() {
                    if let Some((short, long)) = month_name(m) {
                        suffixes.push(format!("{}{}{}", short, p2, year));
                        suffixes.push(format!("{}{}{}", long, p2, year));
                        suffixes.push(format!("{}{}{}", short, p2, &short_year));
                    }
                }

                dates_expanded.push(year.to_string());
            }
        }

        // --- Keyboard Walk Suffixes ---
        for kw in ["qwerty", "asdf", "zxcvbn", "qazwsx", "1qaz", "2wsx", "qwer", "asdfgh"] {
            suffixes.push(kw.to_string());
        }

        // --- Pin / Common Number Suffixes ---
        for pin in [
            "0000", "1111", "2222", "3333", "4444", "5555", "6666", "7777", "8888", "9999",
            "321", "4321", "54321", "123", "1234", "12345", "123456",
            "007", "69", "420", "01", "00", "666", "777", "888", "999", "13", "7",
        ] {
            suffixes.push(pin.to_string());
        }

        // Deduplicate suffixes
        suffixes.sort();
        suffixes.dedup();

        // ═══════════════════════════════════════════════════════
        // 3. SEPARATORS & SPECIALS
        // ═══════════════════════════════════════════════════════
        let separators = ["", "_", ".", "-", "@", "#", "!", "$", "&", "+", "="];
        let specials = [
            "!", "@", "#", "$", "*", "?", "1!", "123!",
            "!!", "!!!", "...", "___", "###", "***", "!@#", "!@#$",
            "123", "007",
        ];

        // ═══════════════════════════════════════════════════════
        // 4. WORD VARIANT GENERATION
        // ═══════════════════════════════════════════════════════
        for word in &all_words {
            if word.is_empty() { continue; }

            let base_variants = case_variants(word);

            // Only reverse short words (≤ 6 chars)
            let mut all_bases = base_variants.clone();
            if word.len() <= 6 {
                let reversed: String = word.chars().rev().collect();
                all_bases.extend(case_variants(&reversed));
            }

            // Generate leet for all case variants
            let mut word_forms: Vec<String> = Vec::new();
            for v in &all_bases {
                word_forms.push(v.clone());
                word_forms.extend(generate_leet(v));
            }
            word_forms.sort();
            word_forms.dedup();

            for form in &word_forms {
                emit!(form.clone());

                // Word + Sep + Suffix
                for suffix in &suffixes {
                    for sep in &separators {
                        emit!(format!("{}{}{}", form, sep, suffix));
                    }
                    // Suffix + Sep + Word
                    for sep in &separators {
                        emit!(format!("{}{}{}", suffix, sep, form));
                    }
                    // Word + Suffix + Special
                    for special in &specials {
                        emit!(format!("{}{}{}", form, suffix, special));
                    }
                    // Sandwich: Special + Word + Suffix + Special
                    for special in &specials {
                        emit!(format!("{}{}{}{}", special, form, suffix, special));
                    }
                    // Complex Sandwich with separators
                    for sep in &separators {
                        if !sep.is_empty() {
                            emit!(format!("{}{}{}{}", sep, form, sep, suffix));
                        }
                    }
                    // Double suffix
                    for extra in ["123", "!", "@", "#", "00", "007"] {
                        emit!(format!("{}{}{}", form, suffix, extra));
                    }
                }

                // Specials only (no suffix)
                for special in &specials {
                    emit!(format!("{}{}", form, special));
                    emit!(format!("{}{}", special, form));
                }

                // Decorative wraps
                emit!(format!("xX{}Xx", form));
                emit!(format!("_{}_", form));
                emit!(format!("x{}x", form));
                emit!(format!("xx{}xx", form));
            }
        }

        // ═══════════════════════════════════════════════════════
        // 5. IDIOMATIC PHRASES
        // ═══════════════════════════════════════════════════════
        let idiom_words: Vec<&String> = self.first_names.iter()
            .chain(self.partners.iter())
            .chain(self.kids.iter())
            .chain(self.pets.iter())
            .chain(self.sports.iter())
            .chain(self.music.iter())
            .chain(self.keywords.iter())
            .chain(self.hobbies.iter())
            .collect();

        let idiom_prefixes = ["ilove", "iluv", "i_love_", "my", "miss", "go", "team", "the"];
        let idiom_postfixes = ["4ever", "4life", "fan", "#1", "rules", "sucks",
            "lover", "rocks", "ftw", "islife"];

        for word in &idiom_words {
            let lower = word.to_lowercase();
            let title = to_title_case(&lower);
            for w in [&lower, &title] {
                for prefix in &idiom_prefixes {
                    emit!(format!("{}{}", prefix, w));
                    for suffix in &suffixes {
                        emit!(format!("{}{}{}", prefix, w, suffix));
                    }
                }
                for postfix in &idiom_postfixes {
                    emit!(format!("{}{}", w, postfix));
                    for suffix in &suffixes {
                        emit!(format!("{}{}{}", w, postfix, suffix));
                    }
                }
            }
        }

        // Family-specific idioms
        for kid in &self.kids {
            let lower = kid.to_lowercase();
            for tmpl in [
                format!("{}smom", lower), format!("{}sdad", lower),
                format!("{}s_mom", lower), format!("{}s_dad", lower),
                format!("mama{}", lower), format!("papa{}", lower),
            ] {
                emit!(tmpl.clone());
                for suffix in &suffixes {
                    emit!(format!("{}{}", tmpl, suffix));
                }
            }
        }

        for pet in &self.pets {
            let lower = pet.to_lowercase();
            for tmpl in [
                format!("my{}", lower), format!("my_{}", lower),
            ] {
                emit!(tmpl.clone());
                for suffix in &suffixes {
                    emit!(format!("{}{}", tmpl, suffix));
                }
            }
        }

        // ═══════════════════════════════════════════════════════
        // 6. INITIALS-BASED PASSWORDS
        // ═══════════════════════════════════════════════════════
        let initials = generate_initials(
            &self.first_names, &self.last_names, &self.partners, &self.kids,
        );

        for init in &initials {
            emit!(init.clone());
            for suffix in &suffixes {
                emit!(format!("{}{}", init, suffix));
                for sep in ["", "_", ".", "#"] {
                    emit!(format!("{}{}{}", init, sep, suffix));
                }
            }
            for special in &specials {
                emit!(format!("{}{}", init, special));
            }
        }

        // ═══════════════════════════════════════════════════════
        // 7. TWO-WORD COMBINATIONS (Fixed: all categories)
        // ═══════════════════════════════════════════════════════
        let mut left_sides: Vec<&String> = Vec::new();
        left_sides.extend(self.first_names.iter());
        left_sides.extend(self.usernames.iter());
        left_sides.extend(self.kids.iter());
        left_sides.extend(self.pets.iter());
        left_sides.extend(self.sports.iter());
        left_sides.extend(self.music.iter());
        left_sides.extend(self.hobbies.iter());
        left_sides.extend(self.parents.iter());

        let mut right_sides: Vec<&String> = Vec::new();
        right_sides.extend(self.first_names.iter());
        right_sides.extend(self.last_names.iter());
        right_sides.extend(self.usernames.iter());
        right_sides.extend(self.keywords.iter());
        right_sides.extend(self.company.iter());
        right_sides.extend(self.school.iter());
        right_sides.extend(self.city.iter());
        right_sides.extend(self.sports.iter());
        right_sides.extend(self.music.iter());
        right_sides.extend(self.kids.iter());
        right_sides.extend(self.pets.iter());
        right_sides.extend(self.partners.iter());
        right_sides.extend(self.parents.iter());
        right_sides.extend(self.maiden_name.iter());
        right_sides.extend(self.hobbies.iter());

        // Explicit Family Combinations
        for p in &self.partners {
            for n in &self.first_names {
                for sep in ["&", "+", "and", "And", "_", "x", "X", "<3", "loves"] {
                    emit!(format!("{}{}{}", n, sep, p));
                    emit!(format!("{}{}{}", p, sep, n));
                }
                for suffix in &suffixes {
                    emit!(format!("{}{}{}", n, p, suffix));
                    emit!(format!("{}{}{}", p, n, suffix));
                }
            }
        }

        for left in &left_sides {
            for right in &right_sides {
                if *left == *right { continue; }

                let l_variants = vec![left.to_lowercase(), to_title_case(&left.to_lowercase())];
                let r_variants = vec![right.to_lowercase(), to_title_case(&right.to_lowercase())];

                for l in &l_variants {
                    for r in &r_variants {
                        for sep in &separators {
                            emit!(format!("{}{}{}", l, sep, r));

                            for suffix in &suffixes {
                                emit!(format!("{}{}{}{}", l, sep, r, suffix));
                                emit!(format!("{}{}{}{}", r, sep, l, suffix));
                            }
                        }
                        for suffix in &suffixes {
                            emit!(format!("{}{}{}", l, r, suffix));
                            emit!(format!("{}{}_{}", l, r, suffix));
                            emit!(format!("{}.{}.{}", l, r, suffix));
                            emit!(format!("{}#{}{}", l, r, suffix));
                            emit!(format!("{}#{}#{}", l, r, suffix));
                            emit!(format!("{}@{}#{}", l, r, suffix));
                        }
                    }
                }

                // camelCase combo
                let camel = format!("{}{}", left.to_lowercase(), to_title_case(&right.to_lowercase()));
                emit!(camel.clone());
                for suffix in &suffixes {
                    emit!(format!("{}{}", camel, suffix));
                }
            }
        }

        // ═══════════════════════════════════════════════════════
        // 8. TRIPLE-TOKEN COMBINATIONS
        // ═══════════════════════════════════════════════════════
        let triple_tokens: Vec<&String> = self.first_names.iter()
            .chain(self.last_names.iter())
            .chain(self.partners.iter())
            .chain(self.kids.iter())
            .chain(self.pets.iter())
            .chain(self.city.iter())
            .collect();

        let max_t = triple_tokens.len().min(8);
        if max_t >= 3 {
            for i in 0..max_t {
                for j in 0..max_t {
                    if j == i { continue; }
                    for k in 0..max_t {
                        if k == i || k == j { continue; }
                        let a = triple_tokens[i].to_lowercase();
                        let b = triple_tokens[j].to_lowercase();
                        let c = triple_tokens[k].to_lowercase();

                        for sep in ["", "_", "."] {
                            emit!(format!("{}{}{}{}{}", a, sep, b, sep, c));
                        }
                        for suffix in &suffixes {
                            emit!(format!("{}{}{}{}", a, b, c, suffix));
                        }
                    }
                }
            }
        }

        // ═══════════════════════════════════════════════════════
        // 9. SUFFIXES & DATES AS STANDALONE
        // ═══════════════════════════════════════════════════════
        for date in &dates_expanded {
            emit!(date.clone());
        }
        for suffix in &suffixes {
            emit!(suffix.clone());
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════

fn to_title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// jOHN from john
fn to_inverted_title(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => {
            let rest: String = c.as_str().to_uppercase();
            format!("{}{}", f.to_lowercase().collect::<String>(), rest)
        }
    }
}

/// johN from john
fn to_last_upper(s: &str) -> String {
    if s.is_empty() { return String::new(); }
    let len = s.len();
    let (head, tail) = s.split_at(len - 1);
    format!("{}{}", head, tail.to_uppercase())
}

/// Generate all case variants for a word
fn case_variants(word: &str) -> Vec<String> {
    let lower = word.to_lowercase();
    let mut variants = vec![
        lower.clone(),
        word.to_uppercase(),
        to_title_case(&lower),
        to_inverted_title(&lower),
        to_last_upper(&lower),
    ];
    variants.sort();
    variants.dedup();
    variants
}

/// Expanded leet generator with partial single-substitution variants
fn generate_leet(s: &str) -> Vec<String> {
    let leet_map: &[(char, &[char])] = &[
        ('a', &['@', '4']),
        ('e', &['3']),
        ('i', &['1']),
        ('l', &['1']),
        ('o', &['0']),
        ('s', &['$', '5']),
        ('t', &['7']),
        ('b', &['8']),
        ('g', &['9']),
        ('z', &['2']),
    ];

    let mut results = Vec::new();
    let chars: Vec<char> = s.chars().collect();

    // 1. Full leet (primary substitution for each, preserving case of non-replaced)
    let full_leet: String = chars.iter().map(|&c| {
        let lower_c = c.to_lowercase().next().unwrap_or(c);
        for (from, to) in leet_map {
            if lower_c == *from {
                return to[0];
            }
        }
        c
    }).collect();

    if full_leet != *s {
        results.push(full_leet);
    }

    // 2. Partial leet: single-position substitutions (capped at 8 variants)
    let mut partial_count = 0;
    for (i, &ch) in chars.iter().enumerate() {
        if partial_count >= 8 { break; }
        let lower_ch = ch.to_lowercase().next().unwrap_or(ch);
        for (from, to_list) in leet_map {
            if lower_ch == *from {
                for &to_char in *to_list {
                    if partial_count >= 8 { break; }
                    let mut variant = chars.clone();
                    variant[i] = to_char;
                    let v: String = variant.into_iter().collect();
                    if v != *s {
                        results.push(v);
                        partial_count += 1;
                    }
                }
                break;
            }
        }
    }

    // 3. Common s → $ only pattern
    let s_leet: String = s.replace('s', "$").replace('S', "$");
    if s_leet != *s {
        results.push(s_leet);
    }

    results.sort();
    results.dedup();
    results
}

/// Generate nickname/truncation variants for words ≥ 5 chars
fn generate_nicknames(word: &str) -> Vec<String> {
    let mut nicknames = Vec::new();
    let lower = word.to_lowercase();
    let len = lower.chars().count();

    if len >= 5 {
        let chars: Vec<char> = lower.chars().collect();
        nicknames.push(chars[..3].iter().collect());
        nicknames.push(chars[..4].iter().collect());
    }
    if len >= 7 {
        let chars: Vec<char> = lower.chars().collect();
        nicknames.push(chars[..5].iter().collect());
    }

    nicknames.sort();
    nicknames.dedup();
    nicknames
}

/// Generate initials from name lists (e.g., JD, jd, J.D.)
fn generate_initials(
    first_names: &[String],
    last_names: &[String],
    partners: &[String],
    kids: &[String],
) -> Vec<String> {
    let mut initials = Vec::new();

    for first in first_names {
        let f = match first.chars().next() {
            Some(c) => c,
            None => continue,
        };

        for last in last_names {
            let l = match last.chars().next() {
                Some(c) => c,
                None => continue,
            };
            // FL, fl, F.L.
            initials.push(format!("{}{}", f.to_uppercase(), l.to_uppercase()));
            initials.push(format!("{}{}", f.to_lowercase(), l.to_lowercase()));
            initials.push(format!("{}.{}.", f.to_uppercase(), l.to_uppercase()));
            initials.push(format!("{}_{}", f.to_uppercase(), l.to_uppercase()));
        }

        for partner in partners {
            let p = match partner.chars().next() {
                Some(c) => c,
                None => continue,
            };
            initials.push(format!("{}{}", f.to_uppercase(), p.to_uppercase()));
            initials.push(format!("{}{}", f.to_lowercase(), p.to_lowercase()));
        }

        for kid in kids {
            let k = match kid.chars().next() {
                Some(c) => c,
                None => continue,
            };
            initials.push(format!("{}{}", f.to_uppercase(), k.to_uppercase()));
            initials.push(format!("{}{}", f.to_lowercase(), k.to_lowercase()));
        }

        // Triple initials: F + L + P or F + L + K
        for last in last_names {
            let l = match last.chars().next() {
                Some(c) => c,
                None => continue,
            };
            for extra in partners.iter().chain(kids.iter()) {
                let e = match extra.chars().next() {
                    Some(c) => c,
                    None => continue,
                };
                initials.push(format!("{}{}{}", f.to_uppercase(), l.to_uppercase(), e.to_uppercase()));
                initials.push(format!("{}{}{}", f.to_lowercase(), l.to_lowercase(), e.to_lowercase()));
            }
        }
    }

    initials.sort();
    initials.dedup();
    initials
}

/// Month name lookup (1-indexed)
fn month_name(month: u32) -> Option<(&'static str, &'static str)> {
    match month {
        1 => Some(("Jan", "January")),
        2 => Some(("Feb", "February")),
        3 => Some(("Mar", "March")),
        4 => Some(("Apr", "April")),
        5 => Some(("May", "May")),
        6 => Some(("Jun", "June")),
        7 => Some(("Jul", "July")),
        8 => Some(("Aug", "August")),
        9 => Some(("Sep", "September")),
        10 => Some(("Oct", "October")),
        11 => Some(("Nov", "November")),
        12 => Some(("Dec", "December")),
        _ => None,
    }
}

/// Decompose a phone number into suffix fragments
fn decompose_phone(number: &str) -> Vec<String> {
    let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
    let mut parts = Vec::new();

    if digits.len() >= 7 {
        parts.push(digits[digits.len()-4..].to_string());
        if digits.len() >= 6 {
            parts.push(digits[digits.len()-6..].to_string());
        }
        parts.push(digits[..3].to_string());
    }

    let reversed: String = digits.chars().rev().collect();
    if reversed != digits && !reversed.is_empty() {
        parts.push(reversed);
    }

    parts.sort();
    parts.dedup();
    parts
}

/// Decompose an email into reusable word parts
fn decompose_email(email: &str) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some(at_pos) = email.find('@') {
        let local = &email[..at_pos];
        parts.push(local.to_string());

        for sep in ['.', '_', '-'] {
            for part in local.split(sep) {
                if !part.is_empty() && part != local {
                    parts.push(part.to_string());
                }
            }
        }
    }
    parts.sort();
    parts.dedup();
    parts
}

/// Decompose a username into component parts
fn decompose_username(username: &str) -> Vec<String> {
    let mut parts = Vec::new();

    for sep in ['_', '-', '.'] {
        for part in username.split(sep) {
            if !part.is_empty() && part != username {
                parts.push(part.to_string());
            }
        }
    }

    // Extract trailing numbers
    let alpha: String = username.chars().take_while(|c| !c.is_ascii_digit()).collect();
    let numeric: String = username.chars().skip_while(|c| !c.is_ascii_digit()).collect();
    if !alpha.is_empty() && alpha != *username {
        parts.push(alpha);
    }
    if !numeric.is_empty() {
        parts.push(numeric);
    }

    parts.sort();
    parts.dedup();
    parts
}

// ═══════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn profile_generates(profile: &Profile, target: &str) -> bool {
        profile.check_password(target)
    }

    fn make_basic_profile() -> Profile {
        Profile {
            first_names: vec!["John".to_string()],
            last_names: vec!["Doe".to_string()],
            dates: vec!["1990".to_string()],
            numbers: vec!["5551234567".to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn test_basic_generation() {
        let p = make_basic_profile();
        assert!(profile_generates(&p, "john"));
        assert!(profile_generates(&p, "John"));
        assert!(profile_generates(&p, "JOHN"));
        assert!(profile_generates(&p, "doe"));
    }

    #[test]
    fn test_nicknames() {
        let p = Profile {
            first_names: vec!["Christopher".to_string()],
            ..Default::default()
        };
        assert!(profile_generates(&p, "chris"));
        assert!(profile_generates(&p, "chri"));
        assert!(profile_generates(&p, "chris"));
    }

    #[test]
    fn test_initials() {
        let p = make_basic_profile();
        assert!(profile_generates(&p, "JD"));
        assert!(profile_generates(&p, "jd"));
        assert!(profile_generates(&p, "J.D."));
    }

    #[test]
    fn test_idioms() {
        let p = make_basic_profile();
        assert!(profile_generates(&p, "ilovejohn"));
        assert!(profile_generates(&p, "iluvjohn"));
        assert!(profile_generates(&p, "myjohn"));
        assert!(profile_generates(&p, "john4ever"));
    }

    #[test]
    fn test_date_formats() {
        let p = Profile {
            dates: vec!["1990".to_string(), "0102".to_string()],
            ..Default::default()
        };
        let candidates = p.generate();
        let strs: Vec<String> = candidates.iter()
            .map(|b| String::from_utf8_lossy(b).to_string())
            .collect();
        // Year range: 1987..1993
        assert!(strs.contains(&"1987".to_string()));
        assert!(strs.contains(&"1993".to_string()));
        // Short year
        assert!(strs.contains(&"90".to_string()));
        // DDMM flip
        assert!(strs.contains(&"0201".to_string()));
        // Date with separator
        assert!(strs.contains(&"01/02".to_string()));
    }

    #[test]
    fn test_leet_partial() {
        let p = Profile {
            first_names: vec!["pass".to_string()],
            ..Default::default()
        };
        // Full leet: p@$$ (a→@, s→$)
        assert!(profile_generates(&p, "p@$$"));
        // Partial leet: only a→@
        assert!(profile_generates(&p, "p@ss"));
    }

    #[test]
    fn test_kids_in_combos() {
        let p = Profile {
            first_names: vec!["John".to_string()],
            kids: vec!["Max".to_string()],
            ..Default::default()
        };
        assert!(profile_generates(&p, "johnmax"));
        assert!(profile_generates(&p, "maxjohn"));
        // Kid idiom
        assert!(profile_generates(&p, "maxsmom"));
    }

    #[test]
    fn test_phone_decomposition() {
        let p = Profile {
            first_names: vec!["John".to_string()],
            numbers: vec!["5551234567".to_string()],
            ..Default::default()
        };
        // Last 4 digits
        assert!(profile_generates(&p, "john4567"));
        // Area code
        assert!(profile_generates(&p, "john555"));
    }

    #[test]
    fn test_keyboard_walks() {
        let p = make_basic_profile();
        assert!(profile_generates(&p, "johnqwerty"));
        assert!(profile_generates(&p, "johnasdf"));
    }

    #[test]
    fn test_length_filter() {
        let p = Profile {
            first_names: vec!["Jo".to_string()],
            min_length: Some(6),
            max_length: Some(12),
            ..Default::default()
        };
        let candidates = p.generate();
        for c in &candidates {
            assert!(c.len() >= 6, "Too short: {:?}", String::from_utf8_lossy(c));
            assert!(c.len() <= 12, "Too long: {:?}", String::from_utf8_lossy(c));
        }
    }

    #[test]
    fn test_case_variants() {
        let p = Profile {
            first_names: vec!["john".to_string()],
            ..Default::default()
        };
        assert!(profile_generates(&p, "jOHN"));
        assert!(profile_generates(&p, "johN"));
    }

    #[test]
    fn test_camel_case_combo() {
        let p = Profile {
            first_names: vec!["John".to_string()],
            last_names: vec!["Doe".to_string()],
            ..Default::default()
        };
        assert!(profile_generates(&p, "johnDoe"));
    }

    #[test]
    fn test_email_decomposition() {
        let p = Profile {
            email: vec!["john.doe@example.com".to_string()],
            ..Default::default()
        };
        assert!(profile_generates(&p, "john"));
        assert!(profile_generates(&p, "doe"));
        assert!(profile_generates(&p, "john.doe"));
    }

    #[test]
    fn test_reversed_short_only() {
        let p = Profile {
            first_names: vec!["Max".to_string()],
            ..Default::default()
        };
        // Short word reversed
        assert!(profile_generates(&p, "xam"));
        assert!(profile_generates(&p, "XAM"));
    }

    #[test]
    fn test_triple_combos() {
        let p = Profile {
            first_names: vec!["John".to_string()],
            last_names: vec!["Doe".to_string()],
            kids: vec!["Max".to_string()],
            ..Default::default()
        };
        assert!(profile_generates(&p, "johndoemax"));
        assert!(profile_generates(&p, "john_doe_max"));
    }

    #[test]
    fn test_age_derivation() {
        let p = Profile {
            first_names: vec!["John".to_string()],
            dates: vec!["1990".to_string()],
            ..Default::default()
        };
        // Age = 2026 - 1990 = 36
        assert!(profile_generates(&p, "john36"));
    }
}
