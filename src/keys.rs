/* 
    Key Pair Usage
*/


extern crate base64;
use base64::{encode, decode};
use core::panic;
use std::io::{Write, Read};
use std::fs::{File};
use rand::Rng;

pub struct Key {
    pub n: u64,
    pub exp: u64,
}

impl Key {

    pub fn from_file(filename: &str) -> Self {
        let mut keyfile = File::open(filename).ok().expect("Key File not found.");
        let mut keystring = String::new();
        let _read_bytes = keyfile.read_to_string(&mut keystring).ok();
        let keybytes = decode(keystring).unwrap();


        let mut expbytes: [u8; 8] = [0; 8];
        expbytes.clone_from_slice(&keybytes[0..8]);
        let exp: u64 = u64::from_ne_bytes(expbytes);

        let mut modbytes: [u8; 8] = [0; 8];
        modbytes.clone_from_slice(&keybytes[8..16]);
        let n: u64 = u64::from_ne_bytes(modbytes);

        Key { n, exp }
    }
    
    pub fn encrypt_str(&self, s: String) -> String {
        let bytes = self.encrypt_bytes(s.into_bytes());
        encode(&bytes)
    }

    pub fn decrypt_str(&self, s: String) -> String {
        let cipher = decode(s).unwrap();
        let bytes = self.decrypt_bytes(cipher);

        String::from_utf8(bytes).unwrap()
    }

    pub fn encrypt_bytes(&self, bytes: Vec<u8>) -> Vec<u8> {
        /* Encrypts 64-bit chunks and returns Vec<u8> */

        println!("original chars: {:X?}", bytes);
    
        // Build a vector of encrypted words
        /*
        */

        let mut words: Vec<u64> = Vec::new();
        for byte in &bytes {
            words.push(*byte as u64);
        }

        println!("original words: {:#010x?}", words);

        // Encrypt each word
        for word in &mut words {
            *word = self.encrypt64(*word);
        }
        
        println!("encrypted words {:#010x?}", words);

        // Unpack into encrypted u8 array
        let mut chars: Vec<u8> = Vec::with_capacity(words.len()*4);
        let mut temp8: u8;
        for (i, word) in words.iter().enumerate() {
            for n in 0..8 {
                chars.push(0);
                temp8 = ((*word & (0xFF << ((7-n)*8))) >> ((7-n)*8)) as u8;
                chars[i*8 + n] = temp8;
            }
        }

        println!("encrypted chars: {:X?}", chars);

        return chars;
    }

    pub fn decrypt_bytes(&self, bytes: Vec<u8>) -> Vec<u8> {

        println!("original bytes: {:X?}", bytes);

        // Chunk bytes into 64-bit words
        let mut words: Vec<u64> = Vec::new();
        let mut buff: u64 = 0;
        for (i, byte) in bytes.iter().enumerate() {
            buff |= (*byte as u64) << ((7-(i%8))*8);
            if (7 - (i % 8)) == 0 || i == (bytes.len() - 1) { // every fourth byte...
                words.push(buff); // push to words
                buff = 0; // reset buffer
            }
        }

        println!("original words: {:#10X?}", words);

        // Decrypt each word and extract characters
        let mut chars: Vec<u8> = Vec::new();
        for word in &mut words {
            *word = self.decrypt64(*word);
            chars.push((*word & 0xFF) as u8);
        }

        println!("decrypted words: {:#10X?}", words);
        println!("decrypted chars: {:X?}", chars);

        return chars;
    } 

    pub fn encrypt64(&self, t: u64) -> u64 {
        let exp_table:[u64; 64] = self.gen_table(t);
        let mut acc: u128 = 1;
        let mut idx: u64 = self.exp;
        for i in 0..64 {
            if (idx & 1) == 1 {
                acc = (acc * exp_table[i] as u128) % (self.n as u128);
            }
            idx = idx >> 1;
        }

        assert!(acc < std::u64::MAX as u128);
        acc as u64
    }

    pub fn decrypt64(&self, t: u64) -> u64 {
        self.encrypt64(t)
    }

    fn gen_table(&self, t: u64) -> [u64; 64] {
        let mut table: [u64; 64] = [0; 64];
        table[0] = t % self.n;
        for i in 1..64 {
            table[i] = (table[i-1].pow(2)) % self.n;
        }
        return table;
    }

    pub fn write_to_file(&self, filename: &str) -> usize {
        // let mut keyfile = base64::write::EncoderWriter::new(File::create(filename).unwrap(), base64::STANDARD);
        let mut keyfile = File::create(filename).ok().expect("Filepath incorrect.");
        keyfile.write(&mut self.dump_key_base64().as_bytes()).unwrap()
    }

    pub fn dump_key_bytes(&self) -> [u8; 16] {
        let expbytes: [u8; 8 * 1] = self.exp.to_ne_bytes();
        let modbytes: [u8; 8 * 1] = self.n.to_ne_bytes();

        let mut keybytes: [u8; 8 * 2] = [0; 8 * 2];
        keybytes[0..8].copy_from_slice(&expbytes);
        keybytes[8..].copy_from_slice(&modbytes);

        keybytes
    }

    pub fn dump_key_base64(&self) -> String {
        encode(&self.dump_key_bytes())
    }


}


pub struct KeyPair {
    pub skey: Key,
    pub pkey: Key
}


impl KeyPair {

    pub fn new () -> Self {
        KeyPair {skey: Key { n: 3233, exp: 2753}, pkey: Key {n: 3233, exp: 17}}

    }

    pub fn generate(&self, k: &u32) -> Self {
        let (d, e, _m, n) = self.calc_exponents(k);
        KeyPair {skey: Key { n, exp: d}, pkey: Key {n, exp: e}}
    }

    fn calc_exponents(&self, k: &u32) -> (u64, u64, u64, u64) {
        //print!("Calculating Exponents of KeyPair...");
        let e: u64 = self.rand_e();
        let (p, q) = self.rand_pq(*k, e);
        let n: u64 = q * p; 
        let m: u64 = (p -1) * (q -1);
        let d = self.ext_gcd(e, m);
        println!("\nKeyPair Length: {}\nKey Len: {}\n", k*2, k);
        println!("\nCalculated Exponents:\np: {}\nq: {}\nd: {}\ne: {}\nm: {}\nn: {}\n", p, q, d, e, m, n);
        (d, e, m, n)
    }


    fn ext_gcd(&self, a: u64, b: u64) -> u64 {
        let (mut a, mut b) = (i128::try_from(a).unwrap(), i128::try_from(b).unwrap());
        let mut x: [i128; 2] = [0, 1];
        let mut y: [i128; 2] = [1, 0];
        let mut q: i128;
        let  old_b: i128 = b;    

        while a != 0 {
            ((q, a), b) = ((b / a, b % a), a);
            (y[0], y[1]) = (y[1], y[0] - q * y[1]);
            (x[0], x[1]) = (x[1], x[0] - q * x[1]);    
        }
        if b != 1 {
            panic!("gcd(a, b) != 1");
        }
        if x[0] < 0 {
            x[0] = x[0] + old_b;
        }
        u64::try_from(x[0]).unwrap()
    }

    fn rand_pq(&self, mut k: u32, e: u64) -> (u64, u64) {
        if k > 32 {
            panic!("Key Bit-Length is too long, must be <=32")
        }
        k = k/2;
        let max: u64 = u64::try_from(2_u128.pow(k) - 1).unwrap();
        let min: u64 = (max >> 1)^max;
        let mut p: [u64;2] = [0,0];
    
        for n in 0..2{
            p[n] = rand::thread_rng().gen_range(min..max);
            loop {
                if !self.is_prime(p[n]) {
                    if p[n] & 1 == 0 {
                        p[n]+=1;
                        continue;
                    }
                    p[n]+=2;
                    continue;
                }
                if p[n] % e != 1 {
                    break;
                }
                p[n]+=2;
            }  
        } 
        (p[0], p[1])
    }
    
    fn is_prime(&self, n:u64) -> bool {
        if n == 2 || n == 3 {
            return true;
        }
        if n <= 1 || n % 2 == 0 || n % 3 == 0 {
            return false;  
        }
        let mut i = 5;
        while  i*i <=n {
            if n % i == 0 || n % (i + 2) == 0 {
                return false;
            }
            i = i+6;
        }
        return true;
    }

    fn rand_e(&self) -> u64 {
        let e: u64;
        let p: [u64; 5] = [3, 5, 17, 257, 65537];
 
        for _n in 0..4 {
            e = p[rand::thread_rng().gen_range(0..4)];
            return e;
        }
        panic!("Can't Find E!!");
    }
}
