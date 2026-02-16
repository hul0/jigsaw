mod engine;
mod io;
mod cli;
mod interactive;
mod api;

use clap::Parser;
use cli::args::{JigsawArgs, Commands};
use engine::mask::Mask;
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
        println!("Training Markov model from {:?}...", train_path);
        let mut model = engine::markov::MarkovModel::new(3); // Default order 3
        model.train(&train_path)?;
        
        let valid_model_path = final_args.model.clone().unwrap_or_else(|| PathBuf::from("jigsaw.model"));
        println!("Saving model to {:?}...", valid_model_path);
        model.save(&valid_model_path)?;
        println!("Training complete.");
        return Ok(());
    }

    // --- Markov Generation Mode ---
    if final_args.markov {
        println!("JIGSAW Running in Markov Mode...");
        let model_path = final_args.model.clone().unwrap_or_else(|| PathBuf::from("jigsaw.model"));
        println!("Loading model from {:?}...", model_path);
        
        // Load model (Arc it for threads)
        let model = engine::markov::MarkovModel::load(&model_path)?;
        let model = std::sync::Arc::new(model);
        
        let count = final_args.count;
        println!("Generating {} candidates...", count);

        if let Some(threads) = final_args.threads {
            rayon::ThreadPoolBuilder::new().num_threads(threads).build_global()?;
        }

        // Setup output
        let (sender, receiver) = bounded::<Vec<Vec<u8>>>(100);
        let writer_output = match final_args.output {
            Some(path) => WriterOutput::File(path),
            None => WriterOutput::Stdout,
        };
        let writer_thread = Writer::new(receiver, writer_output).start();


        // Define a batcher that includes RNG and flushes on drop
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

        // Parallel Generation
        (0..count).into_par_iter()
            .for_each_init(
                || MarkovBatcher {
                    buffer: Vec::with_capacity(1000),
                    sender: sender.clone(),
                    rng: rand::thread_rng(),
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
            
         // Drop sender to signal 'done' (Main thread sender needs to be handled? No, bounded creates one pair. 
         // Wait, `for_each_init` clones the sender. 
         // But the original `sender` variable in main scope is still alive. We must drop it.
         drop(sender);
         writer_thread.join().expect("Writer panic")?;
         println!("Done.");
         return Ok(());
    }

    // --- Memorable Password Mode ---
    if final_args.memorable {
        let password = engine::memorable::generate_memorable_password();
        println!("\nGenerated Memorable Password:\n{}", password);
        return Ok(());
    }

    // --- Personal Attack Mode ---
    if final_args.personal || final_args.profile.is_some() {
        println!("JIGSAW Running in Personal Attack Mode...");
        let profile_path = final_args.profile
            .ok_or_else(|| anyhow::anyhow!("Profile path required for personal mode via CLI (use --profile)"))?;
            
        println!("Loading profile from {:?}...", profile_path);
        let profile = engine::personal::Profile::load(&profile_path)?;
        
        println!("Generating candidates...");
        let candidates = profile.generate();
        println!("Generated {} candidates.", candidates.len());
        
        // Check Mode
        if let Some(target) = &final_args.check {
            println!("Checking for password: '{}'...", target);
            if profile.check_password(target) {
                println!("\n[+] SUCCESS: Password found!");
            } else {
                println!("\n[-] FAILURE: Password NOT found in generated list.");
            }
            return Ok(());
        }

        // Setup Output
        let (sender, receiver) = bounded::<Vec<Vec<u8>>>(100);
        let writer_output = match final_args.output {
            Some(path) => WriterOutput::File(path),
            None => WriterOutput::Stdout,
        };
        let writer_thread = Writer::new(receiver, writer_output).start();
        
        // Send in batches (Single threaded is fast enough for pre-generated vec)
        let mut buffer = Vec::with_capacity(1000);
        for candidate in candidates {
            buffer.push(candidate);
            if buffer.len() >= 1000 {
                sender.send(buffer.clone()).expect("Channel closed");
                buffer.clear();
            }
        }
        if !buffer.is_empty() {
            sender.send(buffer).expect("Channel closed");
        }
        
        drop(sender);
        writer_thread.join().expect("Writer panic")?;
        println!("Done.");
        return Ok(());
    }

    // --- Mask Mode (Original) ---
    // If no mask provided (and not interactive), show help or error
    if final_args.mask.is_none() {
        println!("Error: No mask specified. Use --mask or --interactive.");
        return Ok(());
    }

    let mask_str = final_args.mask.unwrap();
    println!("JIGSAW Running...");
    println!("Mask: {}", mask_str);

    let mask = Mask::from_str(&mask_str)?;
    println!("Search space: {}", mask.search_space_size());

    if let Some(threads) = final_args.threads {
        rayon::ThreadPoolBuilder::new().num_threads(threads).build_global()?;
    }

    // Create channel
    let (sender, receiver) = bounded::<Vec<Vec<u8>>>(100);
    
    // Output config
    let writer_output = match final_args.output {
        Some(path) => WriterOutput::File(path),
        None => WriterOutput::Stdout,
    };

    // Spawn writer thread
    let writer_thread = Writer::new(receiver, writer_output).start();
    
    // Parallel generation
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
    
    // Drop sender to close channel
    drop(sender);
    
    // Wait for writer
    writer_thread.join().expect("Writer thread panicked")?;
    
    println!("Done.");
    Ok(())
}

