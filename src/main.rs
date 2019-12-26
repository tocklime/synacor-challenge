use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{Read, stdout, Write, stdin};
use std::convert::TryInto;
use itertools::Itertools;
use num_enum::TryFromPrimitive;

#[derive(Debug, TryFromPrimitive)]
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

#[derive(Debug)]
pub struct Vm<'a> {
    rom: &'a [u16],
    memory: HashMap<u16, u16>,
    stack: Vec<u16>,
    instruction_pointer: u16,
    running: bool,
    input: Vec<char>,
}

impl<'a> Vm<'a> {
    pub fn new(program: &'a [u16]) -> Self  {
        Vm {
            rom: program,
            memory: (32768..32776).map(|x| (x,0)).collect(),
            stack: Vec::new(),
            instruction_pointer: 0,
            running: true,
            input: Vec::new(),
        }
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
    fn get(&mut self, address: u16) -> u16 {
        self.memory.get(&address).cloned().or_else(|| self.get_rom(address)).unwrap_or_else(||
            panic!("Can't get memory at {}",address))
    }
    fn binop<FN>(&mut self, f: FN)
        where FN: Fn(u16, u16) -> u16
    {
        let (a, b, c) = (self.fetch_set(), self.fetch_read(), self.fetch_read());
        self.set(a, (f(b, c)) % 32768);
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
                print!("{}", ch);
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
    pub fn run(&mut self) {
        while self.running {
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

    vm.run();
    Ok(())
}
