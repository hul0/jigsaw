# JIGSAW

```text
  888888 8888888  .d8888b.   .d8888b.         d8888 888       888 
    "88b   888   d88P  Y88b d88P  Y88b       d88888 888   o   888 
     888   888   888    888 Y88b.           d88P888 888  d8b  888 
     888   888   888         "Y888b.       d88P 888 888 d888b 888 
     888   888   888  88888     "Y88b.    d88P  888 888d88888b888 
     888   888   888    888       "888   d88P   888 88888P Y88888 
     88P   888   Y88b  d88P Y88b  d88P  d8888888888 8888P   Y8888 
     888 8888888  "Y8888P88  "Y8888P"  d88P     888 888P     Y888 
   .d88P                                                          
 .d88P"                                                           
888P"                                                             
```

**Joint Intelligence Generator for Strategic Attack Wordlists**

JIGSAW is a high-performance, Rust-based intelligent wordlist generator designed for modern offensive security. It moves beyond simple brute-force by employing probabilistic models, targeted personal profiling, and advanced permutation logic to generate high-value password candidates.

![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)
![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)

---

## [➤] Key Features

[+] **Blazing Fast**: Built on Rust with multi-threaded parallel processing (Rayon) for maximum throughput.  
[+] **Mask Attack**: Support for Hashcat-style masks (`?l?u?d`) with custom charsets.  
[+] **Markov Attack**: Train probabilistic models on existing wordlists (e.g., RockYou) to generate highly probable candidates.  
[+] **Personal Attack**: Create targeted wordlists based on user profiles (Names, Dates, Interests) with deep permutation logic.  
[+] **Deep Mutation**:
    [>>] **Smart Date Parsing**: `2007` -> `07`, `7`, `007`.  
    [>>] **String Reversal**: `John` -> `nhoj`.  
    [>>] **Leetspeak**: `e` -> `3`, `a` -> `@`, etc.  
    [>>] **Complex Combinations**: `Name@Year`, `Year#Name`, Sandwich patterns (`!Pass123!`).
[+] **Wizard Mode**: Interactive CLI wizard for easy profile creation.

---

## [➤] Installation

### [!] APT Repository (Official)

You can install JIGSAW directly from our official APT repository hosted on GitHub Pages.

```bash
# Add the repository to your sources list
echo "deb [trusted=yes] https://hul0.github.io/jigsaw ./" | sudo tee /etc/apt/sources.list.d/jigsaw.list

# Update and install
sudo apt update
sudo apt install jigsaw
```

### [!] Manual Installation (.deb)

Download the latest `.deb` package from the [Releases](https://github.com/hul0/jigsaw/releases) page.

```bash
sudo dpkg -i jigsaw_*.deb
sudo apt-get install -f # Fix missing dependencies
```

### [!] Build from Source

Ensure you have Rust and Cargo installed.

```bash
git clone https://github.com/hul0/jigsaw.git
cd jigsaw
cargo vendor
cargo build --release
```

The binary will be located at `target/release/jigsaw`.

---

## [➤] Usage

### [!] Interactive Wizard (Recommended)
The easiest way to start is using the interactive wizard.

```bash
./target/release/jigsaw --interactive
```
Select **Personal Attack** to build a target profile step-by-step.

### 1. Mask Mode
Generate candidates based on a specific pattern.

```bash
# Generate 8-char lower/numeric candidates
./jigsaw -m "?l?l?l?l?d?d?d?d" --output mask_list.txt
```

### 2. Markov Mode (Probabilistic)
Train a model on a source wordlist and generate similar passwords.

**Train:**
```bash
./jigsaw --train rockyou.txt
# Saves model to 'markov.json'
```

**Generate:**
```bash
./jigsaw --markov --model markov.json --count 1000000 --output efficient_list.txt
```

### 3. Personal Attack (Targeted)
Generate a wordlist specific to a target's personal information.

**Create a Profile (target.json):**
```json
{
  "first_names": ["John"],
  "last_names": ["Doe"],
  "dates": ["1990"],
  "company": ["Acme"],
  "city": ["NewYork"]
}
```

**Run Attack:**
```bash
./jigsaw --personal --profile target.json --output targeted.txt
```

**What it generates:**
- `John1990`
- `nhoj@90` (Reversed + Smart Date)
- `Acme_2020!`
- `!NewYork123!`

---

## [➤] Performance

JIGSAW uses a buffered, threaded writer to ensure I/O doesn't become a bottleneck. Mask generation acts as a parallel iterator, scaling linearly with CPU cores.

---

## [➤] Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

---

## [➤] License

This project is licensed under the GNU General Public License v3.0 or later - see the [LICENSE](LICENSE) file for details.

Copyright (c) 2026 Rupam Ghosh.

```text
 JIGSAW - JOINT INTELLIGENCE GENERATOR FOR STRATEGIC ATTACK WORDLISTS
```
