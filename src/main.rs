pub mod keys;
pub mod mathlib;
use crate::keys::{KeyPair, Key};
use std::io::{self, Write, BufRead};


fn main() {

    let pair = KeyPair::new(&32);
    // Test number below 3233
    let mut hex: u64 = 0x00000C9F;
    print!("Num Test 1: {:X} -> ", hex);
    print!("{:X} -> ", pair.pkey.encrypt64(hex));
    print!("{:X}    PASS\n", pair.skey.encrypt64(pair.pkey.encrypt64(hex)));

    // Test number above 3233
    hex = 0x0036449e;
    print!("Num Test 2: {:X} -> ", hex);
    print!("{:X} -> ", pair.pkey.encrypt64(hex));
    print!("{:X}    FAIL\n\n", pair.skey.encrypt64(pair.pkey.encrypt64(hex)));

    // Write keys to file
    pair.pkey.write_to_file("rsa.pub");
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

