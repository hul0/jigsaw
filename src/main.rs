mod engine;
mod io;
mod cli;
mod interactive;
mod api;

use clap::Parser;
use cli::args::{JigsawArgs, Commands, OutputFormat, GenerationLevel, MemStyle, MemCase, NumPosition};
use engine::mask::Mask;
use engine::memorable::{MemorableConfig, MemorableStyle, CaseStyle, Position};
use io::writer::{Writer, Output as WriterOutput};
use std::str::FromStr;
use std::path::PathBuf;
use crossbeam_channel::bounded;
use rayon::prelude::*;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let args = JigsawArgs::parse();

    // Check for subcommands first
    if let Some(Commands::Server { port }) = args.command {
        return api::server::run_server(port).await.map_err(|e| anyhow::anyhow!(e));
    }

    let final_args = if args.interactive {
        interactive::run_wizard()?
    } else {
        args
    };

    // --- Markov Training Mode ---
    if let Some(train_path) = final_args.train {
        let start_time = std::time::Instant::now();
        println!("Training Markov model from {:?}...", train_path);
        let mut model = engine::markov::MarkovModel::new(3);
        model.train(&train_path)?;
        
        let valid_model_path = final_args.model.clone().unwrap_or_else(|| PathBuf::from("jigsaw.model"));
        println!("Saving model to {:?}...", valid_model_path);
        model.save(&valid_model_path)?;
        println!("Training complete. Time taken: {}ms", start_time.elapsed().as_millis());
        return Ok(());
    }

    // --- Markov Generation Mode ---
    if final_args.markov {
        let start_time = std::time::Instant::now();
        println!("JIGSAW Running in Markov Mode...");
        let model_path = final_args.model.clone().unwrap_or_else(|| PathBuf::from("jigsaw.model"));
        println!("Loading model from {:?}...", model_path);
        
        let model = engine::markov::MarkovModel::load(&model_path)?;
        let model = std::sync::Arc::new(model);
        
        let count = final_args.count;
        println!("Generating {} candidates...", count);

        if let Some(threads) = final_args.threads {
            rayon::ThreadPoolBuilder::new().num_threads(threads).build_global()?;
        }

        let (sender, receiver) = bounded::<Vec<Vec<u8>>>(100);
        let writer_output = match final_args.output {
            Some(path) => WriterOutput::File(path),
            None => WriterOutput::Stdout,
        };
        let writer_thread = Writer::new(receiver, writer_output).start();

        struct MarkovBatcher {
            buffer: Vec<Vec<u8>>,
            sender: crossbeam_channel::Sender<Vec<Vec<u8>>>,
            rng: rand::rngs::ThreadRng,
        }

        impl Drop for MarkovBatcher {
            fn drop(&mut self) {
                if !self.buffer.is_empty() {
                    let _ = self.sender.send(self.buffer.clone());
                }
            }
        }

        (0..count).into_par_iter()
            .for_each_init(
                || MarkovBatcher {
                    buffer: Vec::with_capacity(1000),
                    sender: sender.clone(),
                    rng: rand::rng(),
                },
                |batcher, _| {
                    let candidate = model.generate(&mut batcher.rng, 6, 12);
                    batcher.buffer.push(candidate.into_bytes());
                    
                    if batcher.buffer.len() >= 1000 {
                        batcher.sender.send(batcher.buffer.clone()).expect("Channel closed");
                        batcher.buffer.clear();
                    }
                }
            );
            
         drop(sender);
         writer_thread.join().expect("Writer panic")?;
         println!("Done. Time taken: {}ms", start_time.elapsed().as_millis());
         return Ok(());
    }

    // --- Memorable Password Mode ---
    if final_args.memorable {
        let start_time = std::time::Instant::now();
        
        let config = build_memorable_config(&final_args);
        let passwords = engine::memorable::generate_batch(&config);
        
        match final_args.format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "passwords": passwords,
                    "count": passwords.len(),
                    "style": format!("{:?}", config.style),
                    "time_taken_ms": start_time.elapsed().as_millis(),
                }))?);
            }
            OutputFormat::Plain => {
                println!("\n  ╔═══════════════════════════════════════════╗");
                println!("  ║     JIGSAW Memorable Passwords            ║");
                println!("  ╚═══════════════════════════════════════════╝\n");
                for (i, pw) in passwords.iter().enumerate() {
                    println!("  {}. {} (len: {})", i + 1, pw, pw.len());
                }
                println!("\n  Generated {} password(s) in {}ms\n",
                    passwords.len(), start_time.elapsed().as_millis());
            }
        }
        return Ok(());
    }

    // --- Personal Attack Mode ---
    if final_args.personal || final_args.profile.is_some() {
        let start_time = std::time::Instant::now();
        println!("\n  ╔═══════════════════════════════════════════╗");
        println!("  ║     JIGSAW Personal Attack Engine          ║");
        println!("  ╚═══════════════════════════════════════════╝\n");
        
        let profile_path = final_args.profile
            .ok_or_else(|| anyhow::anyhow!("Profile path required (use --profile <PATH>)"))?;
            
        println!("  Profile:  {:?}", profile_path);
        println!("  Level:    {:?}", final_args.level);
        
        let mut profile = engine::personal::Profile::load(&profile_path)?;
        
        // Apply CLI length overrides
        if let Some(min) = final_args.min_length {
            profile.min_length = Some(min);
        }
        if let Some(max) = final_args.max_length {
            profile.max_length = Some(max);
        }
        
        if let Some(min) = profile.min_length {
            println!("  Min Len:  {}", min);
        }
        if let Some(max) = profile.max_length {
            println!("  Max Len:  {}", max);
        }
        println!();
        
        // Check Mode
        if let Some(target) = &final_args.check {
            println!("  Checking for password: '{}'...", target);
            if profile.check_password(target) {
                println!("\n  [+] FOUND: Password exists in generated candidates!");
            } else {
                println!("\n  [-] NOT FOUND: Password not in generated list.");
            }
            println!("  Time taken: {}ms", start_time.elapsed().as_millis());
            return Ok(());
        }

        // Generate
        println!("  Generating candidates...");
        let candidates = profile.generate();
        println!("  Generated {} unique candidates.", candidates.len());

        match final_args.format {
            OutputFormat::Json => {
                let strings: Vec<String> = candidates.iter()
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .collect();
                let output_path = final_args.output;
                let json = serde_json::to_string_pretty(&serde_json::json!({
                    "candidates": strings,
                    "total": strings.len(),
                    "time_taken_ms": start_time.elapsed().as_millis(),
                }))?;
                if let Some(path) = output_path {
                    std::fs::write(&path, &json)?;
                    println!("  Written to {:?}", path);
                } else {
                    println!("{}", json);
                }
            }
            OutputFormat::Plain => {
                // Setup Output via writer
                let (sender, receiver) = bounded::<Vec<Vec<u8>>>(100);
                let writer_output = match final_args.output {
                    Some(path) => WriterOutput::File(path),
                    None => WriterOutput::Stdout,
                };
                let writer_thread = Writer::new(receiver, writer_output).start();
                
                // Send in parallel batches
                let chunk_size = 1000;
                for chunk in candidates.chunks(chunk_size) {
                    sender.send(chunk.to_vec()).expect("Channel closed");
                }
                
                drop(sender);
                writer_thread.join().expect("Writer panic")?;
            }
        }
        
        println!("  Done. Time taken: {}ms\n", start_time.elapsed().as_millis());
        return Ok(());
    }

    // --- Mask Mode ---
    if final_args.mask.is_none() {
        println!("Error: No mode specified. Use --interactive, --personal, --memorable, --mask, or --markov.");
        println!("Try: jigsaw --help");
        return Ok(());
    }

    let mask_str = final_args.mask.unwrap();
    let start_time = std::time::Instant::now();
    println!("JIGSAW Running...");
    println!("Mask: {}", mask_str);

    let mask = Mask::from_str(&mask_str)?;
    println!("Search space: {}", mask.search_space_size());

    if let Some(threads) = final_args.threads {
        rayon::ThreadPoolBuilder::new().num_threads(threads).build_global()?;
    }

    let (sender, receiver) = bounded::<Vec<Vec<u8>>>(100);
    
    let writer_output = match final_args.output {
        Some(path) => WriterOutput::File(path),
        None => WriterOutput::Stdout,
    };

    let writer_thread = Writer::new(receiver, writer_output).start();
    
    struct BatchSender {
        buffer: Vec<Vec<u8>>,
        sender: crossbeam_channel::Sender<Vec<Vec<u8>>>,
    }
    
    impl Drop for BatchSender {
        fn drop(&mut self) {
            if !self.buffer.is_empty() {
                let _ = self.sender.send(self.buffer.clone());
            }
        }
    }
    
    mask.par_iter().for_each_init(
        || BatchSender {
            buffer: Vec::with_capacity(1000),
            sender: sender.clone(),
        },
        |batcher, candidate| {
            batcher.buffer.push(candidate);
            if batcher.buffer.len() >= 1000 {
                batcher.sender.send(batcher.buffer.clone()).expect("Writer channel closed");
                batcher.buffer.clear();
            }
        }
    );
    
    drop(sender);
    writer_thread.join().expect("Writer thread panicked")?;
    
    println!("Done. Time taken: {}ms", start_time.elapsed().as_millis());
    Ok(())
}

/// Build MemorableConfig from CLI args
fn build_memorable_config(args: &JigsawArgs) -> MemorableConfig {
    MemorableConfig {
        word_count: args.words,
        separator: args.mem_sep.clone(),
        case_style: match args.mem_case {
            MemCase::Title => CaseStyle::Title,
            MemCase::Lower => CaseStyle::Lower,
            MemCase::Upper => CaseStyle::Upper,
            MemCase::Random => CaseStyle::Random,
            MemCase::Alternating => CaseStyle::Alternating,
        },
        include_number: args.mem_number && !args.no_number,
        number_position: match args.num_pos {
            NumPosition::Start => Position::Start,
            NumPosition::End => Position::End,
            NumPosition::Between => Position::Between,
        },
        number_max: args.num_max,
        include_special: args.mem_special && !args.no_special,
        special_position: match args.special_pos {
            NumPosition::Start => Position::Start,
            NumPosition::End => Position::End,
            NumPosition::Between => Position::Between,
        },
        style: match args.mem_style {
            MemStyle::Classic => MemorableStyle::Classic,
            MemStyle::Passphrase => MemorableStyle::Passphrase,
            MemStyle::Story => MemorableStyle::Story,
            MemStyle::Alliterative => MemorableStyle::Alliterative,
        },
        count: args.mem_count,
        min_length: args.mem_min_len,
        max_length: args.mem_max_len,
    }
}
