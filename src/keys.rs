/* 
    Key Pair Usage
*/

use std::ptr::null;

use rand::Rng;

pub struct Key {
    pub n: u64,
    pub exp: u64,
}

impl Key {
    
    pub fn encrypt_str(&self, s: String) -> String {
        let mut bytes = s.into_bytes();
        let mut result: Vec<u64> = Vec::new();
    
        let mut temp: u64;
        let mut buff: u64 = 0;
        for (i, byte) in bytes.iter().enumerate() {
            //println!("{:#X}", *byte);
            temp = (*byte as u64) << ((3-(i%4))*8);
            //println!("{:#X}", temp);
            buff = temp | buff;
            //println!("buff: {:#X}", buff);
            if (3-(i%4)) == 0 {
                result.push(self.encrypt64(buff));
                buff = 0;
            }
        }

        let mut temp8: u8;
        for (i, word) in result.iter_mut().enumerate() {
            for n in 0..3 {
                temp8 = ((*word & (0xFF << ((3-n)*8))) >> ((3-n)*8)) as u8;
                println!("{:X}", temp8);
                bytes[i*4 + n] = temp8;
            }
        }

        println!("{:X?}", bytes);
        
        String::from_utf8(bytes).ok().expect("Failed to convert to UTF8.")
    }

    pub fn encrypt64(&self, t: u64) -> u64 {
        let exp_table:[u64; 64] = self.gen_table(t);
        let mut acc: u64 = 1;
        let mut idx: u64 = self.exp;
        for i in 0..64 {
            if (idx & 1) == 1 {
                acc = (acc * exp_table[i]) % self.n;
            }
            idx = idx >> 1;
        }

        acc
    }

    fn gen_table(&self, t: u64) -> [u64; 64] {
        let mut table: [u64; 64] = [0; 64];
        table[0] = t % self.n;
        for i in 1..64 {
            table[i] = (table[i-1].pow(2)) % self.n;
        }
        table
    }

}

pub struct KeyPair {
    pub skey: Key,
    pub pkey: Key,
}

pub trait KeyPairGenerate {
    fn calcExponents(&self) -> (u64, u64, u64, u64);

    fn RandInitExponents(&self) -> (u64, u64);

    fn Gcd(&self, first: u64, second: u64) -> u64;
    
    fn findE(&self, m: u64) -> u64;

    fn new () -> Self;
}

impl KeyPairGenerate for KeyPair {

    fn new () -> Self {
        KeyPair {skey: Key { n: 0, exp: 0}, pkey: Key {n: 0, exp: 0}}

    }

    fn calcExponents(&self) -> (u64, u64, u64, u64) {
        print!("Calculating Exponents of KeyPair...");
        let (p, q) = self.RandInitExponents();
        let n: u64 = q * p;
        let m: u64 = (p - 1) * (q - 1);
        let e: u64 = self.findE(m);
        let d: u64 = (1 + n * m) / e;
        println!("\nCalculated Exponents:\nd: {}\ne: {}\nm: {}\nn: {}", d, e, m, n);
        (d, e, m, n)
    }

    fn RandInitExponents(&self) -> (u64, u64) {
        let mut rng = rand::thread_rng();
        (rng.gen_range(0..10), rng.gen_range(0..10))
    }


    fn Gcd(&self, first: u64, second: u64) -> u64 {
        let mut max = first;
        let mut min = second;
        if min > max {
            let val = max;
            max = min;
            min = val;
        }
    
        loop {
            let res = max % min;
            if res == 0 {
                return min;
            }
    
            max = min;
            min = res;
        }
    }
    
    fn findE(&self, m: u64) -> u64 {
        // let e: Vec<u64> = range(2, m);
        for n in 2..m {
            if self.Gcd(n, m) == 1 {
                return n;
            }
        }
        return 1;
    }
}
