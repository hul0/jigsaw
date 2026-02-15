use std::io::{self, Write, BufWriter};
use std::fs::File;
use std::path::PathBuf;
use std::thread;
use crossbeam_channel::Receiver;
use anyhow::Result;

pub enum Output {
    Stdout,
    File(PathBuf),
}

pub struct Writer {
    receiver: Receiver<Vec<Vec<u8>>>,
    output: Output,
}

impl Writer {
    pub fn new(receiver: Receiver<Vec<Vec<u8>>>, output: Output) -> Self {
        Self { receiver, output }
    }

    pub fn start(self) -> thread::JoinHandle<Result<()>> {
        thread::spawn(move || {
            let writer: Box<dyn Write> = match self.output {
                Output::Stdout => Box::new(BufWriter::new(io::stdout().lock())),
                Output::File(path) => Box::new(BufWriter::new(File::create(path)?)),
            };

            let mut writer = BufWriter::new(writer);

            // Iterate over received batches
            for batch in self.receiver {
                for candidate in batch {
                    writer.write_all(&candidate)?;
                    writer.write_all(b"\n")?;
                }
            }

            writer.flush()?;
            Ok(())
        })
    }
}
