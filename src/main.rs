pub mod keys;
use crate::keys::{KeyPair, Key};
use std::io::{self, Write, BufRead};
extern crate clap;
use clap::Parser;

/// Simple 64-bit RSA encryption implementation
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// File to encrypt
    file: Option<std::path::PathBuf>
}

fn main() {
    let args = Cli::parse();
    println!("input file: {:?}", args.file.as_deref());

    let pair = KeyPair::new().generate(&32);
    
    // Test two arbitrary u64s
    test64(&pair, 0x00000C9F);
    test64(&pair, 0x0036449e);

    // Write keys to file
    pair.pkey.write_to_file("rsa.pem.pub");
    pair.skey.write_to_file("rsa.pem");
    println!("Saved keys to disk.\n");

    // Ask for input
    print!("Enter text to encrypt: ");
    io::stdout().flush().unwrap();
    let mut encrypted: String = String::new();
    for line in io::stdin().lock().lines() {
        let data: String = line.unwrap();
        println!("{:?}", data);
        encrypted = pair.pkey.encrypt_str(data);
        println!("Result: {}", encrypted);
        break;
    }

    // Test reading key from file
    let key_from_file: Key = Key::from_file("rsa.pem");

    // Test decryption
    println!("Decrypting result...");
    let decrypted = key_from_file.decrypt_str(encrypted);
    println!("Final message: \n\n{}\n", decrypted);

}

fn test64(pair: &KeyPair, t: u64) -> () {
    let c = pair.pkey.encrypt64(t);
    let d = pair.skey.decrypt64(c);
    println!("Testing 0x{:016X}\npkey => 0x{:016X}\nskey => 0x{:016X}\t{}", t, c, d, match t==d {
        true => "PASS",
        false => "FAIL"
    });
}