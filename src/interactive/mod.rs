use crate::cli::args::JigsawArgs;
use crate::engine::mask::Mask;
use crate::engine::personal::Profile;
use std::str::FromStr;
use dialoguer::{theme::ColorfulTheme, Input, Select, Confirm, MultiSelect};
use std::path::PathBuf;

pub fn run_wizard() -> anyhow::Result<JigsawArgs> {
    println!("Welcome to JIGSAW Interactive Mode");
    println!("----------------------------------");

    let modes = vec!["Mask Attack", "Personal Attack", "Generate Memorable Password", "Check Password", "Quit"];
    let mode_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Action")
        .default(0)
        .items(&modes)
        .interact()?;

    match mode_selection {
        0 => run_mask_wizard(),
        1 => run_personal_wizard(),
        2 => run_memorable_wizard(),
        3 => run_check_wizard(),
        _ => {
            std::process::exit(0);
        }
    }
}

fn run_memorable_wizard() -> anyhow::Result<JigsawArgs> {
    Ok(JigsawArgs {
        mask: None, rules: None, threads: None, output: None, interactive: false,
        train: None, model: None, markov: false, count: 0, 
        personal: false, profile: None, 
        memorable: true, check: None, command: None
    })
}

fn run_check_wizard() -> anyhow::Result<JigsawArgs> {
    println!("\n--- Password Checker ---");
    let profile_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Path to Profile JSON")
        .default("target.json".into())
        .interact_text()?;
    
    let password: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Password to Check")
        .interact_text()?;
        
    Ok(JigsawArgs {
        mask: None, rules: None, threads: None, output: None, interactive: false,
        train: None, model: None, markov: false, count: 0, 
        personal: true, 
        profile: Some(PathBuf::from(profile_path)), 
        memorable: false, 
        check: Some(password), command: None
    })
}

fn run_mask_wizard() -> anyhow::Result<JigsawArgs> {
    let mask_input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter Mask Pattern (e.g. ?u?l?l?d)")
        .validate_with(|input: &String| -> Result<(), &str> {
            if Mask::from_str(input).is_ok() {
                Ok(())
            } else {
                Err("Invalid mask pattern format")
            }
        })
        .interact_text()?;

    let output_file: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Output File Path (leave empty for stdout)")
        .allow_empty(true)
        .interact_text()?;

    let output_path = if output_file.trim().is_empty() {
        None
    } else {
        Some(PathBuf::from(output_file))
    };

    let use_custom_threads = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Configure threads manually?")
        .default(false)
        .interact()?;

    let threads = if use_custom_threads {
        let count: usize = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Number of threads")
            .default(4)
            .interact_text()?;
        Some(count)
    } else {
        None
    };

    Ok(JigsawArgs {
        mask: Some(mask_input),
        rules: None, 
        threads,
        output: output_path,
        interactive: false,
        train: None,
        model: None,
        markov: false,
        count: 10000,
        personal: false,
        profile: None,
        command: None,
        memorable: false,
        check: None,
    })
}

fn run_personal_wizard() -> anyhow::Result<JigsawArgs> {
    println!("\n--- Personal Attack Profile Builder ---");
    println!("We will build a targeted profile. Answer 'Yes' to add categories.");
    
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

    // 1. Identity (Always ask Name)
    println!("\n[ Identity ]");
    profile.first_names = ask_list("Target's First Name(s)")?;
    profile.last_names = ask_list("Target's Last Name(s)")?;
    profile.usernames = ask_category("Usernames / Handles", "Usernames (e.g. jdoe99, xX_Slayer_Xx)")?;

    // 2. Family
    println!("\n[ Family & Relationships ]");
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add Family/Relationship info?")
        .default(false)
        .interact()? 
    {
        profile.partners = ask_list("Partner/Spouse Name(s)")?;
        profile.kids = ask_list("Children's Name(s)")?;
        profile.pets = ask_list("Pet's Name(s)")?;
    }

    // 3. Work & Education
    println!("\n[ Work & Education ]");
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add Work/School info?")
        .default(false)
        .interact()? 
    {
        profile.company = ask_list("Company / Organization Name(s)")?;
        profile.school = ask_list("School / University Name(s)")?;
    }

    // 4. Location
    println!("\n[ Location ]");
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add Location info?")
        .default(false)
        .interact()? 
    {
        profile.city = ask_list("City / Town / Region")?;
    }

    // 5. Interests
    println!("\n[ Interests & Favorites ]");
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add Interests (Sports, Music, etc.)?")
        .default(false)
        .interact()? 
    {
        profile.sports = ask_list("Sports Teams / Athletes")?;
        profile.music = ask_list("Music Bands / Artists")?;
        profile.keywords = ask_list("Other Keywords (Car brand, Color, Hobby)")?;
    }

    // 6. Numbers & Dates
    println!("\n[ Numbers & Dates ]");
    profile.dates = ask_list("Important Dates (Years like 1990, MMDD like 0101)")?;
    profile.numbers = ask_list("Important Numbers (Phone segments, Zip, Room #)")?;

    // Save
    let save_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Save Profile to JSON file?")
        .default("target.json".into())
        .interact_text()?;
    
    let path = PathBuf::from(&save_path);
    profile.save(&path)?;
    println!("Profile saved to {:?}", path);

    // Output
    let output_file: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Output Wordlist File (leave empty for stdout)")
        .allow_empty(true)
        .interact_text()?;

    let output_path = if output_file.trim().is_empty() {
        None
    } else {
        Some(PathBuf::from(output_file))
    };

    Ok(JigsawArgs {
        mask: None,
        rules: None,
        threads: None,
        output: output_path,
        interactive: false,
        train: None,
        model: None,
        markov: false,
        count: 0,
        personal: true,
        profile: Some(path),
        command: None,
        memorable: false,
        check: None,
    })
}
