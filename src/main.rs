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
                print!("{}",ch);
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
    pub fn run_to_input(&mut self) {
        while self.running {
            let op = self.peek_op();
            if op == Op::In && self.input.is_empty()
            { break; }
            self.step();
        }
    }
}

fn main() -> io::Result<()> {
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
    vm.run_to_input();
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
            vm.run_to_input();
            step_no += 1;
        } else if s.starts_with("dissassemble") {
            println!("{}",vm.disassemble());
        } else {
            vm.input = s.chars().filter(|x| x != &'\r').rev().collect();
            vm.run_to_input();
            step_no += 1;
        }
    }
    print!("{}", vm.take_output());
    Ok(())
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
take teleporter";


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
