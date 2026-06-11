#![forbid(unsafe_code)]

use snark_lab_interchange::{parse_and_verify, Protocol};
use std::{env, fs, process};

fn usage() -> ! {
    eprintln!("usage: snark-lab-cli verify-transcript <path.json>");
    process::exit(2);
}

fn main() {
    let mut args = env::args().skip(1);
    if args.next().as_deref() != Some("verify-transcript") {
        usage();
    }
    let path = args.next().unwrap_or_else(|| usage());
    if args.next().is_some() {
        usage();
    }

    let json = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("error: could not read {path}: {error}");
        process::exit(2);
    });
    match parse_and_verify(&json) {
        Ok(transcript) => {
            let protocol = match transcript.protocol {
                Protocol::Sumcheck => "sumcheck",
                Protocol::Zerocheck => "zerocheck",
            };
            println!(
                "accepted: {protocol} transcript verified over F_{}",
                transcript.field.modulus
            );
        }
        Err(error) => {
            eprintln!("rejected: {error}");
            process::exit(1);
        }
    }
}
