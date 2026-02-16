use crate::cli::args::{JigsawArgs, GenerationLevel, OutputFormat, MemStyle, MemCase, NumPosition};
use crate::engine::mask::Mask;
use crate::engine::personal::Profile;
use std::str::FromStr;
use dialoguer::{theme::ColorfulTheme, Input, Select, Confirm};
use std::path::PathBuf;

pub fn run_wizard() -> anyhow::Result<JigsawArgs> {
    println!();
    println!("  ╔═══════════════════════════════════════════╗");
    println!("  ║     JIGSAW — Interactive Wizard            ║");
    println!("  ╚═══════════════════════════════════════════╝");
    println!();

    let modes = vec![
        "🔑 Personal Attack — Generate wordlist from target profile",
        "🎲 Memorable Password — Generate strong memorable passwords",
        "🔍 Check Password — Test if a password is in the wordlist",
        "🎭 Mask Attack — Brute-force with mask patterns",
        "📖 Load Existing Profile — Load and re-run a saved profile",
        "❌ Quit",
    ];

    let mode_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Action")
        .default(0)
        .items(&modes)
        .interact()?;

    match mode_selection {
        0 => run_personal_wizard(),
        1 => run_memorable_wizard(),
        2 => run_check_wizard(),
        3 => run_mask_wizard(),
        4 => run_load_profile_wizard(),
        _ => std::process::exit(0),
    }
}

// ═══════════════════════════════════════════════════════════════
// PERSONAL ATTACK WIZARD
// ═══════════════════════════════════════════════════════════════

fn run_personal_wizard() -> anyhow::Result<JigsawArgs> {
    println!("\n  ── Personal Attack Profile Builder ──\n");

    let mut profile = Profile::new();

    fn ask_list(prompt: &str) -> anyhow::Result<Vec<String>> {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("{} (comma separated)", prompt))
            .allow_empty(true)
            .interact_text()?;

        Ok(input.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    fn ask_category(category_name: &str, items_prompt: &str) -> anyhow::Result<Vec<String>> {
         if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Add {}?", category_name))
            .default(false)
            .interact()?
        {
            ask_list(items_prompt)
        } else {
            Ok(Vec::new())
        }
    }

    // ── Identity ──
    println!("  [ Identity ]");
    profile.first_names = ask_list("Target's First Name(s)")?;
    profile.last_names = ask_list("Target's Last Name(s)")?;
    profile.usernames = ask_category("Usernames / Handles", "Usernames (e.g. jdoe99, xX_Slayer_Xx)")?;

    // ── Family ──
    println!("\n  [ Family & Relationships ]");
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add Family info?")
        .default(false)
        .interact()?
    {
        profile.partners = ask_list("Partner/Spouse Name(s)")?;
        profile.kids = ask_list("Children's Name(s)")?;
        profile.pets = ask_list("Pet's Name(s)")?;
        profile.parents = ask_list("Parent Name(s)")?;
        profile.maiden_name = ask_list("Maiden Name(s)")?;
    }

    // ── Work & Education ──
    println!("\n  [ Work & Education ]");
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add Work/School info?")
        .default(false)
        .interact()?
    {
        profile.company = ask_list("Company / Organization")?;
        profile.school = ask_list("School / University")?;
    }

    // ── Location ──
    println!("\n  [ Location ]");
    profile.city = ask_category("Location", "City / Town / Region")?;

    // ── Interests ──
    println!("\n  [ Interests & Favorites ]");
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add Interests?")
        .default(false)
        .interact()?
    {
        profile.sports = ask_list("Sports Teams / Athletes")?;
        profile.music = ask_list("Music Bands / Artists")?;
        profile.hobbies = ask_list("Hobbies (Gaming, Cooking, etc.)")?;
        profile.keywords = ask_list("Other Keywords (Car, Color, Movie, Brand)")?;
    }

    // ── Online ──
    println!("\n  [ Online Presence ]");
    profile.email = ask_category("Email Addresses", "Email(s)")?;

    // ── Numbers & Dates ──
    println!("\n  [ Numbers & Dates ]");
    profile.dates = ask_list("Important Dates (Years like 1990, MMDD like 0101)")?;
    profile.numbers = ask_list("Important Numbers (Phone, Zip, Room #)")?;

    // ── Generation Settings ──
    println!("\n  [ Generation Settings ]");

    let levels = vec!["Quick (~10K)", "Standard (~100K)", "Deep (~500K+)", "Insane (~1M+)"];
    let level_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Generation Level")
        .default(1)
        .items(&levels)
        .interact()?;

    let level = match level_idx {
        0 => GenerationLevel::Quick,
        1 => GenerationLevel::Standard,
        2 => GenerationLevel::Deep,
        _ => GenerationLevel::Insane,
    };

    // Length Filter
    let use_length_filter = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Set password length filter?")
        .default(false)
        .interact()?;

    if use_length_filter {
        let min_val: usize = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Minimum length (0 = none)")
            .default(0)
            .interact_text()?;
        let max_val: usize = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Maximum length (0 = none)")
            .default(0)
            .interact_text()?;
        if min_val > 0 { profile.min_length = Some(min_val); }
        if max_val > 0 { profile.max_length = Some(max_val); }
    }

    // Output Format
    let format_options = vec!["Plain text (one per line)", "JSON"];
    let format_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Output Format")
        .default(0)
        .items(&format_options)
        .interact()?;

    let format = match format_idx {
        1 => OutputFormat::Json,
        _ => OutputFormat::Plain,
    };

    // Save Profile
    let save_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Save profile to")
        .default("target.json".into())
        .interact_text()?;

    let path = PathBuf::from(&save_path);
    profile.save(&path)?;
    println!("  ✓ Profile saved to {:?}", path);

    // Output file
    let output_file: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Output wordlist file (empty = stdout)")
        .allow_empty(true)
        .interact_text()?;

    let output_path = if output_file.trim().is_empty() {
        None
    } else {
        Some(PathBuf::from(output_file))
    };

    Ok(JigsawArgs {
        mask: None, rules: None, threads: None,
        output: output_path,
        format,
        interactive: false,
        train: None, model: None, markov: false, count: 0,
        personal: true,
        profile: Some(path),
        level,
        min_length: profile.min_length,
        max_length: profile.max_length,
        memorable: false,
        words: 3, mem_sep: String::new(), mem_style: MemStyle::Classic,
        mem_case: MemCase::Title, mem_number: true, no_number: false,
        num_pos: NumPosition::End, num_max: 99,
        mem_special: true, no_special: false, special_pos: NumPosition::End,
        mem_count: 1, mem_min_len: 12, mem_max_len: 32,
        check: None, command: None,
    })
}

// ═══════════════════════════════════════════════════════════════
// MEMORABLE PASSWORD WIZARD
// ═══════════════════════════════════════════════════════════════

fn run_memorable_wizard() -> anyhow::Result<JigsawArgs> {
    println!("\n  ── Memorable Password Generator ──\n");

    // Style
    let style_options = vec![
        "Classic (Adjective-Noun-Verb)",
        "Passphrase (random words)",
        "Story (Subject-Verb-Object)",
        "Alliterative (same letter)",
    ];
    let style_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Password Style")
        .default(0)
        .items(&style_options)
        .interact()?;

    let mem_style = match style_idx {
        0 => MemStyle::Classic,
        1 => MemStyle::Passphrase,
        2 => MemStyle::Story,
        _ => MemStyle::Alliterative,
    };

    // Word Count
    let words: usize = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Number of words")
        .default(3)
        .validate_with(|v: &usize| {
            if *v >= 2 && *v <= 8 { Ok(()) } else { Err("Must be 2-8") }
        })
        .interact_text()?;

    // Separator
    let sep_options = vec!["None (CamelCase)", "Dash (-)", "Underscore (_)", "Dot (.)", "Space ( )", "Custom"];
    let sep_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Word Separator")
        .default(0)
        .items(&sep_options)
        .interact()?;

    let mem_sep = match sep_idx {
        0 => String::new(),
        1 => "-".to_string(),
        2 => "_".to_string(),
        3 => ".".to_string(),
        4 => " ".to_string(),
        _ => {
            let custom: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Custom separator")
                .interact_text()?;
            custom
        }
    };

    // Case Style
    let case_options = vec!["Title Case", "lowercase", "UPPERCASE", "rAnDoM", "AlTeRnAtInG"];
    let case_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Case Style")
        .default(0)
        .items(&case_options)
        .interact()?;

    let mem_case = match case_idx {
        0 => MemCase::Title,
        1 => MemCase::Lower,
        2 => MemCase::Upper,
        3 => MemCase::Random,
        _ => MemCase::Alternating,
    };

    // Number
    let mem_number = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Include a number?")
        .default(true)
        .interact()?;

    let mut num_pos = NumPosition::End;
    let mut num_max: u32 = 99;
    if mem_number {
        let pos_options = vec!["End", "Start", "Between words"];
        let pos_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Number position")
            .default(0)
            .items(&pos_options)
            .interact()?;
        num_pos = match pos_idx {
            1 => NumPosition::Start,
            2 => NumPosition::Between,
            _ => NumPosition::End,
        };

        let max_options = vec!["0-9", "00-99", "000-999", "0000-9999"];
        let max_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Number range")
            .default(1)
            .items(&max_options)
            .interact()?;
        num_max = match max_idx {
            0 => 9,
            1 => 99,
            2 => 999,
            _ => 9999,
        };
    }

    // Special
    let mem_special = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Include a special character?")
        .default(true)
        .interact()?;

    let mut special_pos = NumPosition::End;
    if mem_special {
        let pos_options = vec!["End", "Start", "Between words"];
        let pos_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Special char position")
            .default(0)
            .items(&pos_options)
            .interact()?;
        special_pos = match pos_idx {
            1 => NumPosition::Start,
            2 => NumPosition::Between,
            _ => NumPosition::End,
        };
    }

    // Count
    let mem_count: usize = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("How many passwords to generate?")
        .default(5)
        .interact_text()?;

    // Length
    let mem_min_len: usize = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Minimum password length")
        .default(12)
        .interact_text()?;

    let mem_max_len: usize = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Maximum password length")
        .default(32)
        .interact_text()?;

    Ok(JigsawArgs {
        mask: None, rules: None, threads: None,
        output: None,
        format: OutputFormat::Plain,
        interactive: false,
        train: None, model: None, markov: false, count: 0,
        personal: false, profile: None,
        level: GenerationLevel::Standard,
        min_length: None, max_length: None,
        memorable: true,
        words, mem_sep, mem_style, mem_case,
        mem_number, no_number: !mem_number,
        num_pos, num_max,
        mem_special, no_special: !mem_special,
        special_pos, mem_count, mem_min_len, mem_max_len,
        check: None, command: None,
    })
}

// ═══════════════════════════════════════════════════════════════
// CHECK PASSWORD WIZARD
// ═══════════════════════════════════════════════════════════════

fn run_check_wizard() -> anyhow::Result<JigsawArgs> {
    println!("\n  ── Password Checker ──\n");

    let profile_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Path to Profile JSON")
        .default("target.json".into())
        .interact_text()?;

    let password: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Password to Check")
        .interact_text()?;

    Ok(JigsawArgs {
        mask: None, rules: None, threads: None,
        output: None, format: OutputFormat::Plain,
        interactive: false,
        train: None, model: None, markov: false, count: 0,
        personal: true,
        profile: Some(PathBuf::from(profile_path)),
        level: GenerationLevel::Standard,
        min_length: None, max_length: None,
        memorable: false,
        words: 3, mem_sep: String::new(), mem_style: MemStyle::Classic,
        mem_case: MemCase::Title, mem_number: true, no_number: false,
        num_pos: NumPosition::End, num_max: 99,
        mem_special: true, no_special: false, special_pos: NumPosition::End,
        mem_count: 1, mem_min_len: 12, mem_max_len: 32,
        check: Some(password), command: None,
    })
}

// ═══════════════════════════════════════════════════════════════
// MASK ATTACK WIZARD
// ═══════════════════════════════════════════════════════════════

fn run_mask_wizard() -> anyhow::Result<JigsawArgs> {
    println!("\n  ── Mask Attack ──\n");
    println!("  Patterns: ?l=lower ?u=upper ?d=digit ?s=special\n");

    let mask_input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter Mask Pattern (e.g. ?u?l?l?d)")
        .validate_with(|input: &String| -> Result<(), &str> {
            if Mask::from_str(input).is_ok() { Ok(()) } else { Err("Invalid pattern") }
        })
        .interact_text()?;

    let output_file: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Output file (empty = stdout)")
        .allow_empty(true)
        .interact_text()?;

    let output_path = if output_file.trim().is_empty() {
        None
    } else {
        Some(PathBuf::from(output_file))
    };

    let threads = if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Configure threads?")
        .default(false)
        .interact()?
    {
        Some(Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Number of threads")
            .default(4)
            .interact_text()?)
    } else {
        None
    };

    Ok(JigsawArgs {
        mask: Some(mask_input), rules: None, threads,
        output: output_path, format: OutputFormat::Plain,
        interactive: false,
        train: None, model: None, markov: false, count: 10000,
        personal: false, profile: None,
        level: GenerationLevel::Standard,
        min_length: None, max_length: None,
        memorable: false,
        words: 3, mem_sep: String::new(), mem_style: MemStyle::Classic,
        mem_case: MemCase::Title, mem_number: true, no_number: false,
        num_pos: NumPosition::End, num_max: 99,
        mem_special: true, no_special: false, special_pos: NumPosition::End,
        mem_count: 1, mem_min_len: 12, mem_max_len: 32,
        check: None, command: None,
    })
}

// ═══════════════════════════════════════════════════════════════
// LOAD EXISTING PROFILE
// ═══════════════════════════════════════════════════════════════

fn run_load_profile_wizard() -> anyhow::Result<JigsawArgs> {
    println!("\n  ── Load Existing Profile ──\n");

    let profile_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Path to Profile JSON")
        .default("target.json".into())
        .interact_text()?;

    let path = PathBuf::from(&profile_path);
    let profile = Profile::load(&path)?;

    println!("\n  Profile loaded successfully:");
    println!("    Names:    {:?}", profile.first_names);
    println!("    Surnames: {:?}", profile.last_names);
    println!("    Partners: {:?}", profile.partners);
    println!("    Kids:     {:?}", profile.kids);
    println!("    Pets:     {:?}", profile.pets);
    println!("    Dates:    {:?}", profile.dates);
    println!("    Numbers:  {:?}", profile.numbers);
    println!();

    let actions = vec!["Generate wordlist", "Check a password", "Back to menu"];
    let action_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What to do?")
        .default(0)
        .items(&actions)
        .interact()?;

    match action_idx {
        0 => {
            // Generation level
            let levels = vec!["Quick", "Standard", "Deep", "Insane"];
            let level_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Generation Level")
                .default(1)
                .items(&levels)
                .interact()?;

            let level = match level_idx {
                0 => GenerationLevel::Quick,
                1 => GenerationLevel::Standard,
                2 => GenerationLevel::Deep,
                _ => GenerationLevel::Insane,
            };

            let format_options = vec!["Plain text", "JSON"];
            let format_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Output Format")
                .default(0)
                .items(&format_options)
                .interact()?;

            let output_file: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Output file (empty = stdout)")
                .allow_empty(true)
                .interact_text()?;

            Ok(JigsawArgs {
                mask: None, rules: None, threads: None,
                output: if output_file.trim().is_empty() { None } else { Some(PathBuf::from(output_file)) },
                format: if format_idx == 1 { OutputFormat::Json } else { OutputFormat::Plain },
                interactive: false,
                train: None, model: None, markov: false, count: 0,
                personal: true, profile: Some(path),
                level,
                min_length: profile.min_length, max_length: profile.max_length,
                memorable: false,
                words: 3, mem_sep: String::new(), mem_style: MemStyle::Classic,
                mem_case: MemCase::Title, mem_number: true, no_number: false,
                num_pos: NumPosition::End, num_max: 99,
                mem_special: true, no_special: false, special_pos: NumPosition::End,
                mem_count: 1, mem_min_len: 12, mem_max_len: 32,
                check: None, command: None,
            })
        }
        1 => {
            let password: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Password to check")
                .interact_text()?;

            Ok(JigsawArgs {
                mask: None, rules: None, threads: None,
                output: None, format: OutputFormat::Plain,
                interactive: false,
                train: None, model: None, markov: false, count: 0,
                personal: true, profile: Some(path),
                level: GenerationLevel::Standard,
                min_length: None, max_length: None,
                memorable: false,
                words: 3, mem_sep: String::new(), mem_style: MemStyle::Classic,
                mem_case: MemCase::Title, mem_number: true, no_number: false,
                num_pos: NumPosition::End, num_max: 99,
                mem_special: true, no_special: false, special_pos: NumPosition::End,
                mem_count: 1, mem_min_len: 12, mem_max_len: 32,
                check: Some(password), command: None,
            })
        }
        _ => std::process::exit(0),
    }
}
