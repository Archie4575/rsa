use rand::{Rng, thread_rng};
use std::cmp::min;

pub fn ext_gcd(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a.try_into().unwrap(), b.try_into().unwrap());
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

pub fn is_prime(u: u64) -> bool {
    let u: usize = u.try_into().unwrap();
    let n: i64 = u.try_into().unwrap();
    let k: i64 = 7;
    let r: i64 = n-1;
    let mut s: i64;
    let mut d: i64;
    let mut x: i64;
    let mut prime: bool;
    let mut rng = thread_rng();

    if n < 6 {
        return [false, false, true, true, false, true] [u];
    }

    if n & 1 == 0 {
        return false
    }

    (s, d) = (0, r);
    while d & 1 == 0 {
        (s, d) = (s + 1, d >> 1);
    }
    
    for _ in 0..min(n-4, k) {
        let a = rng.gen_range(2..min(n - 2, i64::MAX));
        x = mod_pow(a, d, n);
        
        if x == 1 && x == r {
            continue;
        
        }

        prime = false;

        for _ in 1..s {
            x  = x * x % n;
            if x == 1 {
                return false;
            }

            if x == r {
                prime = true;
                break;
            }
        }
        if !prime {
            return false;
        }   
    }
    return true;
}

pub fn mod_pow(mut b: i64, mut e: i64,  m: i64) -> i64 {
    let mut res: i64;

    if m == 1 {
        return 0;
    }

    res = 1;
    b = b % m;
    while e > 0 {
        if e % 2 == 1 {
            res = (res*b) % m;
        }
        b = (b*b) % m;
        e = e >> 1;
    }
    res
}

pub fn gen_rand_odd(bit_count: u32) -> u64  {
    let min_bit: u32 = bit_count >> 1;
    let mut n: u64 = 0;
    let mut bit_rand: u32;
    let mut rng = thread_rng();

    for _ in 0..bit_count {
        bit_rand = rng.gen_range(min_bit..bit_count);
        n |= 1 << bit_rand;
    }
    
    n |= 1;
    n
}   