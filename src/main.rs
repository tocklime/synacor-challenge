#![recursion_limit="10000000"]
use std::collections::{HashMap, BTreeMap, HashSet};
use std::cmp::{min, max};
use std::fs::File;
use std::io;
use std::io::{Read, stdout, Write, stdin};
use std::convert::TryInto;
use itertools::Itertools;
use num_enum::TryFromPrimitive;
use std::borrow::Cow;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod mod_arith;
use mod_arith::*;
#[derive(Debug, TryFromPrimitive, PartialEq, Eq, Clone, Copy)]
#[repr(u16)]
pub enum Op {
    Halt = 0,
    Set,
    Push,
    Pop,
    Eq,
    Gt,
    Jmp,
    Jt,
    Jf,
    Add,
    Mult,
    Mod,
    And,
    Or,
    Not,
    Rmem,
    Wmem,
    Call,
    Ret,
    Out,
    In,
    Nop,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Vm<'a> {
    rom: Cow<'a, [u16]>,
    memory: BTreeMap<u16, u16>,
    stack: Vec<u16>,
    instruction_pointer: u16,
    running: bool,
    input: Vec<char>,
    output: String,
    live_output: bool,
}

impl<'a> Vm<'a> {
    pub fn new(program: &'a [u16]) -> Self {
        Vm {
            rom: Cow::from(program),
            memory: (32768..32776).map(|x| (x, 0)).collect(),
            stack: Vec::new(),
            instruction_pointer: 0,
            running: true,
            input: Vec::new(),
            output: String::new(),
            live_output: true,
        }
    }
    fn flash_rom(&mut self) {
        let mem_max: u16 = min(32768, max(self.memory.keys().max().unwrap_or(&0) + 1, self.rom.len().try_into().unwrap()));
        let mut new_rom = vec![0; mem_max.into()];
        for i in 0..mem_max {
            let as_usize: usize = i.into();
            new_rom[as_usize] = self.try_get(i).unwrap_or(0)
        }
        self.rom = Cow::from(new_rom);
        let old_regs = self.memory.split_off(&32768);
        self.memory = old_regs;
    }
    fn dump(&mut self) {
        println!("VM: Stack: {:?}, IP: {}", self.stack, self.instruction_pointer);
    }
    fn fetch_read(&mut self) -> u16 {
        let i = self.fetch_set();
        if i >= 32768 {
            self.get(i)
        } else { i }
    }
    fn fetch_set(&mut self) -> u16 {
        let i = self.get(self.instruction_pointer);
        self.instruction_pointer += 1;
        //self.log(format!("{} ", i));
        i
    }
    fn get_rom(&self, addr: u16) -> Option<u16> {
        let a_us: usize = addr.into();
        if a_us < self.rom.len() {
            Some(self.rom[a_us])
        } else {
            None
        }
    }
    fn set(&mut self, address: u16, value: u16) {
        if self.get_rom(address.into()) == Some(value) {
            self.memory.remove(&address);
        } else {
            self.memory.insert(address, value);
        }
    }
    fn try_get(&self, address: u16) -> Option<u16> {
        self.memory.get(&address).cloned().or_else(|| self.get_rom(address))
    }
    fn get(&self, address: u16) -> u16 {
        self.try_get(address).unwrap_or_else(||
            panic!("Can't get memory at {}", address))
    }
    fn binop<FN>(&mut self, f: FN)
        where FN: Fn(u16, u16) -> u16
    {
        let (a, b, c) = (self.fetch_set(), self.fetch_read(), self.fetch_read());
        self.set(a, (f(b, c)) % 32768);
    }
    fn arg_count(o: Op) -> u16 {
        match o {
            Op::Halt | Op::Ret | Op::Nop => 0,
            Op::Push | Op::Pop | Op::Call | Op::Out | Op::In => 1,
            Op::Set | Op::Jt | Op::Jf | Op::Not | Op::Rmem |
            Op::Wmem => 2,
            Op::Eq | Op::Gt | Op::Jmp | Op::Add | Op::Mult |
            Op::Mod | Op::And | Op::Or => 3,
        }
    }
    fn step(&mut self) {
        //self.log(format!("@{} ",self.instruction_pointer));
        let op: Op = self.fetch_read().try_into().expect("Unknown op code");
        match op {
            Op::Halt => self.running = false,
            Op::Set => {
                let a = self.fetch_set();
                let b = self.fetch_read();
                self.set(a, b);
            }
            Op::Push => {
                let a = self.fetch_read();
                self.stack.push(a);
            }
            Op::Pop => {
                let a = self.fetch_set();
                let v = self.stack.pop().expect("Empty stack!");
                self.set(a, v);
            }
            Op::Eq => self.binop(|a, b| (a == b).into()),
            Op::Gt => self.binop(|a, b| (a > b).into()),
            Op::Jmp => self.instruction_pointer = self.fetch_read(),
            Op::Jt => {
                let (a, b) = (self.fetch_read(), self.fetch_read());
                if a != 0 {
                    self.instruction_pointer = b;
                }
            }
            Op::Jf => {
                let (a, b) = (self.fetch_read(), self.fetch_read());
                if a == 0 {
                    self.instruction_pointer = b;
                }
            }
            Op::Add => self.binop(|a, b| a + b),
            Op::Mult => self.binop(|a, b| a.wrapping_mul(b)),
            Op::Mod => self.binop(|a, b| a % b),
            Op::And => self.binop(|a, b| a & b),
            Op::Or => self.binop(|a, b| a | b),
            Op::Not => {
                let a = self.fetch_set();
                let b = self.fetch_read();
                self.set(a, (!b) % 32768);
            }
            Op::Rmem => {
                let a = self.fetch_set();
                let ab = self.fetch_read();
                let b = self.get(ab);
                self.set(a, b);
            }
            Op::Wmem => {
                let a = self.fetch_read();
                let b = self.fetch_read();
                self.set(a, b);
            }
            Op::Call => {
                let a = self.fetch_read();
                self.stack.push(self.instruction_pointer);
                self.instruction_pointer = a;
            }
            Op::Ret => {
                if self.stack.is_empty()
                { self.running = false; } else {
                    self.instruction_pointer = self.stack.pop().expect("Empty stack after check");
                }
            }
            Op::Out => {
                let ch: u16 = self.fetch_read();
                let ch: char = std::char::from_u32(ch.into()).expect("Invalid char");
                self.output.push(ch);
                if self.live_output {
                    print!("{}", ch);
                }
            }
            Op::In => {
                if self.input.is_empty() {
                    let _ = stdout().flush();
                    let mut s = String::new();
                    stdin().read_line(&mut s).expect("Bad input");
                    self.input = s.chars().filter(|x| x != &'\r').rev().collect();
                }
                let a = self.fetch_set();
                let i = self.input.pop().unwrap() as u16;
                self.set(a, i);
            }
            Op::Nop => (), // NoOp
        }
    }
    pub fn disassemble(&self) -> String {
        let mut my_ip = 0_u16;
        let mut ans = String::new();
        loop {
            ans += &format!("@{} ", my_ip);
            let val = self.try_get(my_ip);
            if val == None {
                break;
            }
            if let Ok(op) = val.unwrap().try_into() {
                let c = Vm::arg_count(op);
                ans += &format!("{:?}", op);
                for i in 0..c {
                    ans += &format!(" {}", self.get(my_ip + 1 + i));
                }
                my_ip += 1 + c;
            } else {
                ans += &format!("{}", val.unwrap());
                my_ip += 1;
            }
            ans += "\n";
        }
        ans
    }
    pub fn peek_op(&self) -> Op {
        self.get(self.instruction_pointer).try_into().unwrap()
    }
    pub fn take_output(&mut self) -> String {
        let mut ans = String::new();
        std::mem::swap(&mut self.output, &mut ans);
        ans
    }
    pub fn run_to_input(&mut self, running: Arc<AtomicBool>) {
        running.store(true, Ordering::SeqCst);
        while self.running && running.load(Ordering::SeqCst) {
            let op = self.peek_op();
            if op == Op::In && self.input.is_empty()
            { break; }
            self.step();
        }
        running.store(false, Ordering::SeqCst);
    }
}

fn main() -> io::Result<()> {
    for i in 3..32768 {
        if i % 1000 == 0 {
            println!("{}", i);
        }
        if fn6027a(4, 1, i) == 6 {
            println!("Found solution {}", i);
        }
    }
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        if !r.swap(false, Ordering::SeqCst) {
            println!("Got Ctrl-C whilst not running, exiting");
            std::process::exit(1);
        }
    }).expect("Error setting ctrl-c handler");
    let mut file = File::open("doc/challenge.bin")?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let program: Vec<u16> = data
        .chunks(2)
        .map(|s| {
            let hi: u16 = s[1] as u16;
            let lo: u16 = s[0] as u16;
            hi << 8 | lo
        }).collect_vec();
    let mut vm = Vm::new(&program);
    vm.run_to_input(running.clone());
    let mut step_no = 0;
    let mut saves: HashMap<Vm, usize> = HashMap::new();
    let mut by_step: HashMap<usize, Vm> = HashMap::new();
    let mut all_input = String::new();
    //vm.flash_rom();
    loop {
        let first_seen = *saves.entry(vm.clone()).or_insert(step_no);
        if first_seen == step_no {
            by_step.insert(step_no, vm.clone());
        }
        let output = vm.take_output();
        //println!("{}", output);
        print!("STEP {} (first seen {}): ", step_no, first_seen);
        let _ = stdout().flush();
        let mut s = String::new();
        stdin().read_line(&mut s).expect("Bad input");
        all_input += &s;
        if &s == "quit" {
            break;
        } else if s.starts_with("diff ") {
            let ws = s.split(" ").collect_vec();
            let a: Result<usize, _> = ws[1].parse();
            let b: Result<usize, _> = ws[2].parse();
            match (a, b) {
                (Ok(a), Ok(b)) => {
                    println!("Diffing {} and {}", a, b);
                    let vma = by_step.get(&a).expect("First diff item");
                    let vmb = by_step.get(&b).expect("Second diff item");
                    let keysa: HashSet<u16> = vma.memory.keys().cloned().collect();
                    let keysb: HashSet<u16> = vmb.memory.keys().cloned().collect();
                    let changed = keysa.union(&keysb).filter(|k| vma.memory.get(k) != vmb.memory.get(k)).collect_vec();
                    println!("Changed: ");
                    for a in changed {
                        println!("  @{:?} = {:?} ==> {:?}", a, vma.memory.get(a), vmb.memory.get(a));
                    }
                }
                (a, b) => println!("usage: diff <a> <b> (a and b both ints)\n{:?}\n{:?}", a, b)
            }
        } else if s.starts_with("load ") {
            let ws = s.split(" ").collect_vec();
            match ws[1].parse() {
                Ok(x) => {
                    if let Some(sav) = by_step.get(&x) {
                        vm = sav.clone();
                    } else {
                        println!("Unknown state: {:?}", x);
                    }
                }
                _ => {
                    println!("usage: load <a>");
                }
            }
        } else if s.starts_with("get ") {
            let ws = s.trim().split(" ").collect_vec();
            match ws[1].parse() {
                Ok(x) => {
                    println!("@{} = {:?}", x, vm.try_get(x));
                }
                _ => {
                    println!("usage: get <a>");
                }
            }
        } else if s.starts_with("set ") {
            let ws = s.split(" ").collect_vec();
            let a: Result<u16, _> = ws[1].parse();
            let b: Result<u16, _> = ws[2].parse();
            match (a, b) {
                (Ok(a), Ok(b)) => {
                    vm.set(a, b);
                }
                _ => {
                    println!("usage: set <loc> <value>");
                }
            }
        } else if s.starts_with("input") {
            println!("{}", all_input);
        } else if s.starts_with("solve") {
            vm.input = PARTIAL_SOLUTION.chars().filter(|x| x != &'\r').rev().collect();
            vm.run_to_input(running.clone());
            step_no += 1;
        } else if s.starts_with("dissassemble") {
            println!("{}", vm.disassemble());
        } else if s.starts_with("dump") {
            vm.dump();
        } else if s.starts_with("search") {
            let mut v_ref = vm.clone();
            v_ref.live_output = false;
            v_ref.set(6054, 21);
            v_ref.set(6055, 21);
            v_ref.set(6058, 0);
            v_ref.flash_rom();
            v_ref.input = "use teleporter\n".chars().rev().collect();
            let _ = v_ref.take_output();
            for i in 1..32768 {
                if i % 100 == 0 {
                    println!("{}", i);
                }
                let mut this_v = v_ref.clone();
                this_v.set(32775, i);
                this_v.run_to_input(running.clone());
                let out_str = this_v.take_output();
                if !out_str.contains("Miscalibration detected!") {
                    println!("Got no miscalibration with R8 = {}", i);
                }
            }
        } else {
            vm.input = s.chars().filter(|x| x != &'\r').rev().collect();
            vm.run_to_input(running.clone());
            step_no += 1;
        }
    }
    print!("{}", vm.take_output());
    Ok(())
}
pub fn fn6027a(a: u16, b: u16, c: u16) -> u16 {
//Called with a=4, b = 1. Find c to make it return 6 in a.
    /*
        if a = 0 then b +1.
        if b = 0 then f(a-1, c)
        otherwise, f(a-1, f(a,b-1))
    */
    let a = match a {
        0 => b                    + 1,
        //1 => b            +     c + 1,
        //2 => mod_mul(c + 1, b,32768)  + mod_mul(2, c, 32768) + 1,
        //3 => (mod_pow(c+1,b+3,32768)).wrapping_sub(1 + mod_mul(2,c,32768)) / c,
        _ => match b {
            0 => fn6027a(a-1, c, c),
            b => {
                let new_r1 = fn6027a(a,b - 1, c);
                return fn6027a(a-1, new_r1, c);
            }
        }
    };
    return a;

 /*   match (a, b) {
        (0,_) => b + 1,
        (_,0) => fn6027a(a-1, c, c),
        (_,_) => {
            let new_r1 = fn6027a(a,b - 1, c);
            return fn6027a(a-1, new_r1, c);
        }
    }*/
}
pub fn fn6027(cache: &mut HashMap<(u16,u16),u16>, r0: u16, r1: u16, r7: u16) -> u16 {
//Called with r0=4, r1 = 1. Find r7 to make it return 6 in r0.
    //if r0 < 3 {
    //    return fn6027a(r0,r1,r7);
   // }
    let k = &(r0,r1);
    if cache.contains_key(k) {
        return cache[k];
    }
    //println!("Fn({}, {}, {})", r0, r1, r7);
    let ans = match (r0, r1) {
        (0,_) => (r1 + 1) % 32768,
        (_,0) => //fn6027(cache, r0-1, r7, r7),
            (r7+1).pow((r0-1).into()) + r7,
        (_,_) => {
            let new_r1 = fn6027(cache, r0,r1 - 1, r7);
            fn6027(cache, r0-1, new_r1, r7)
        }
    };
    cache.insert(*k,ans);
    if r0 < 4{
        assert_eq!(fn6027a(r0,r1,r7),ans);
    }
    if r0 == 3
    {
        println!("Fn({}, {}, {}) == {}", r0, r1, r7, ans);
    }
    ans
}

const PARTIAL_SOLUTION: &str = "doorway
north
north
bridge
continue
down
east
take empty lantern
west
west
passage
ladder
west
south
north
take can
west
ladder
use can
use lantern
darkness
continue
west
west
west
west
north
take red coin
north
east
take concave coin
down
take corroded coin
up
west
west
up
take shiny coin
down
take blue coin
east
use blue coin
use red coin
use shiny coin
use concave coin
use corroded coin
north
take teleporter
";


/*
        |
        b
        |
   c   -a
   |    |
   c----L-
   |    |
  -d-
   |
a: twisty maze of little passages, all alike
b: maze of little twisty passages, all alike
c: little maze of twisty passages, all alike
d: twisty alike of little passages, all maze
*/
