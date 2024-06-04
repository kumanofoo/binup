use clap::Parser;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};

enum PatchError {
    TooLongString(String),
    ConversionFailure(String),
    InsufficientLines(usize),
    TooManyLines(usize),
}

impl Display for PatchError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            PatchError::TooLongString(s) => {
                write!(f, "Too long String '{s}'.")
            }
            PatchError::ConversionFailure(s) => {
                write!(f, "Failed to conversion: '{s}'.")
            }
            PatchError::InsufficientLines(n) => {
                write!(f, "Insufficient lines: block({n}).")
            }
            PatchError::TooManyLines(n) => {
                write!(f, "Too many lines: block({n}).")
            }
        }
    }
}

fn hex_string_to_array<const N: usize>(hex_string: &str) -> Result<[u8; N], PatchError> {
    let mut result = [0u8; N];
    for (i, s) in hex_string.split_whitespace().enumerate() {
        if i >= N {
            return Err(PatchError::TooLongString(hex_string.to_string()));
        }
        result[i] = match u8::from_str_radix(s, 16) {
            Ok(u) => u,
            Err(e) => return Err(PatchError::ConversionFailure(format!("{e} ({s})"))),
        }
    }
    Ok(result)
}

#[derive(Debug)]
struct Patch {
    old: [u8; 9],
    new: [u8; 9],
}

impl Patch {
    fn new(old_str: &str, new_str: &str) -> Result<Patch, PatchError> {
        let old = match hex_string_to_array::<9>(old_str) {
            Ok(arr) => arr,
            Err(e) => return Err(e),
        };
        let new = match hex_string_to_array::<9>(new_str) {
            Ok(arr) => arr,
            Err(e) => return Err(e),
        };

        Ok(Patch { old, new })
    }
}

fn clean_string(line: &str) -> String {
    if let Some(index) = line.find('#') {
        let before_hash = &line[..index];
        before_hash.trim().to_string()
    } else {
        line.trim().to_string()
    }
}

fn read_patches_from_file(filename: &str) -> Result<Vec<Vec<String>>, PatchError> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut blocks = Vec::new();
    let mut current_block = Vec::new();

    let mut count = 1;

    for line in reader.lines() {
        let line = clean_string(&line.unwrap());
        if line.is_empty() {
            if !current_block.is_empty() {
                if current_block.len() == 1 {
                    return Err(PatchError::InsufficientLines(count));
                }
                if current_block.len() > 2 {
                    return Err(PatchError::TooManyLines(count));
                }
                blocks.push(current_block);
                current_block = Vec::new();
                count += 1;
            }
        } else {
            current_block.push(line);
        }
    }

    if current_block.len() == 1 {
        return Err(PatchError::InsufficientLines(count));
    }
    if current_block.len() > 2 {
        return Err(PatchError::TooManyLines(count));
    }
    if current_block.len() == 2 {
        blocks.push(current_block);
    }
    Ok(blocks)
}

#[derive(Debug)]
struct Patches {
    patches: Vec<Patch>,
}

impl Patches {
    fn new(filename: &str) -> Result<Patches, PatchError> {
        let patches_str = match read_patches_from_file(filename) {
            Ok(p) => p,
            Err(e) => return Err(e),
        };
        let mut patches = Vec::new();
        for patch in patches_str {
            let p = match Patch::new(&patch[0], &patch[1]) {
                Ok(p) => p,
                Err(e) => return Err(e),
            };
            patches.push(p);
        }
        return Ok(Patches { patches });
    }
}

fn array_to_hexstring(arr: &[u8]) -> String {
    arr.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<String>>()
        .join(" ")
}

fn binary_patch(filename: &str, output: &str, patches: Patches) {
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open '{filename}': {e}"),
    };
    let mut output = match File::create(output) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open '{filename}': {e}"),
    };

    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    for p in patches.patches {
        let mut position = 0;
        while let Some(pos) = find_pattern(&data[position..], &p.old) {
            println!(
                "\n    pattern: {}\nreplacement: {}",
                array_to_hexstring(&p.old),
                array_to_hexstring(&p.new)
            );
            let absolute_pos = position + pos;
            data[absolute_pos..absolute_pos + p.old.len()].copy_from_slice(&p.new);
            position = absolute_pos + p.old.len();
        }
    }

    output.write_all(&data).unwrap();
}

fn find_pattern(data: &[u8], pat: &[u8]) -> Option<usize> {
    data.windows(pat.len()).position(|window| window == pat)
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(help = "Target file")]
    target_file: String,

    #[arg(
        help = "Patch file\n [binary pattern1]\n [replacement1]\n\n [binary pattern2]\n [replacement2]\n\n e.g.\n 00 01 02 03 04 05 \n 10 11 12 13 14 15\n\n 29 2a 2b 2c 2d 2e\n 39 3A 3B 3C 3D 3E"
    )]
    patch_file: String,

    #[arg(short = 'o', help = "Output file (Default '<TARGET FILE>_new')")]
    output_file: Option<String>,
}

fn main() {
    let args = Args::parse();
    let patches = match Patches::new(&args.patch_file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    let output_file = match args.output_file {
        Some(file) => file,
        None => format!("{}_new", args.target_file),
    };
    binary_patch(&args.target_file, &output_file, patches);
}
