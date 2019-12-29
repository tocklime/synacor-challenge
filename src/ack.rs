use std::collections::HashMap;
use std::io::{stdout, Write};
use std::thread;
use rayon::prelude::*;

pub fn pure_ack(a: u16, b: u16, c: u16) -> u16 {
    if a == 0 {
        return (b + 1) % 32768;
    }
    if b == 0 {
        return pure_ack((a + 32767) % 32768, c, c);
    }
    let intermediate = pure_ack(a, (b + 32767) % 32768, c);
    pure_ack((a + 32767) % 32768, intermediate , c)
}

pub fn memo_ack(memo: &mut HashMap<(u16, u16), u16>, a: u16, b: u16, c: u16) -> u16 {
    if let Some(ans) = memo.get(&(a, b)) {
        return *ans;
    }
    if a == 0 {
        let ret = (b + 1) % 32768;
        memo.insert((a, b), ret);
        return ret;
    }
    if b == 0 {
        let ret = memo_ack(memo,(a + 32767) % 32768, c, c);
        memo.insert((a, b), ret);
        return ret;
    }
    let intermediate = memo_ack(memo,a, (b + 32767) % 32768, c);
    let ret = memo_ack(memo,(a + 32767) % 32768, intermediate, c);
    memo.insert((a, b), ret);
    return ret;
}

pub fn search() {
    rayon::ThreadPoolBuilder::new()
        .stack_size(1000000000).build_global().unwrap();
    (1..32768_u16).into_par_iter().for_each(|i| {
        if i % 32 == 0 {
            print!(".");
            let _ = stdout().flush();
        }
        if memo_ack(&mut HashMap::new(), 4, 1, i) == 6 {
            println!("Found {}", i);
        }
    });
}

