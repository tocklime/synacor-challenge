use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Read;
use std::convert::TryInto;
use itertools::Itertools;

#[derive(Debug)]
pub struct Vm {
    memory: HashMap<u16, u16>,
    registers: [u16; 8],
    stack: Vec<u16>,
    instruction_pointer: u16,
    running: bool,
    dumping: bool,
}

impl Vm {
    pub fn new(program: &[u16]) -> Self {
        Vm {
            memory: program.iter().enumerate().map(|(i, x)| (i.try_into().unwrap(), *x)).collect(),
            registers: [0_u16; 8],
            stack: Vec::new(),
            instruction_pointer: 0,
            running: true,
            dumping: false,
        }
    }
    fn fetch_read(&mut self) -> u16 {
        let i = self.fetch_set();
        if i >= 32768 {
            self.registers[(i - 32768) as usize]
        } else { i }
    }
    fn log(&self, s:String) {
        if self.dumping {
            print!("{}", s);
        }
    }
    fn fetch_set(&mut self) -> u16 {
        let i = self.memory[&self.instruction_pointer];
        self.instruction_pointer += 1;
        self.log(format!("{} ", i));
        i
    }
    fn set(&mut self, address: u16, value: u16) {
        if address >= 32768 {
            self.log(format!(" R{}={} ", address - 32768, value));
            self.registers[(address - 32768) as usize] = value
        } else {
            self.log(format!("M{}={}", address, value));
            self.memory.insert(address, value);
        }
    }
    fn get(&mut self, address: u16) -> u16 {
        if address >= 32768 {
            self.registers[(address - 32768) as usize]
        } else {
            *self.memory.entry(address).or_default()
        }
    }
    fn binop<FN>(&mut self, f: FN)
        where FN: Fn(u16, u16) -> u16
    {
        let (a, b, c) = (self.fetch_set(), self.fetch_read(), self.fetch_read());
        self.set(a, (f(b, c)) % 32768);
    }
    fn step(&mut self) {
        self.log(format!("@{} ",self.instruction_pointer));
        let op = self.fetch_read();
        match op {
            0 => self.running = false,
            1 => {
                let a = self.fetch_set();
                let b = self.fetch_read();
                self.set(a, b);
            }
            2 => {
                let a = self.fetch_read();
                self.stack.push(a);
            }
            3 => {
                let a = self.fetch_set();
                let v = self.stack.pop().expect("Empty stack!");
                self.set(a, v);
            }
            4 => self.binop(|a, b| (a == b).into()),
            5 => self.binop(|a, b| (a > b).into()),
            6 => self.instruction_pointer = self.fetch_read(),
            7 => {
                let (a, b) = (self.fetch_read(), self.fetch_read());
                if a != 0 {
                    self.instruction_pointer = b;
                }
            }
            8 => {
                let (a, b) = (self.fetch_read(), self.fetch_read());
                if a == 0 {
                    self.instruction_pointer = b;
                }
            }
            9 => self.binop(|a, b| a + b),
            10 => self.binop(|a, b| a.wrapping_mul(b)),
            11 => self.binop(|a, b| a % b),
            12 => self.binop(|a, b| a & b),
            13 => self.binop(|a, b| a | b),
            14 => {
                let a = self.fetch_set();
                let b = self.fetch_read();
                self.set(a, (!b) % 32768);
            }
            15 => {
                let a = self.fetch_set();
                let ab = self.fetch_read();
                let b = self.get(ab);
                self.set(a, b);
            }
            16 => {
                let a = self.fetch_read();
                let b = self.fetch_read();
                self.set(a, b);
            }
            17 => {
                let a = self.fetch_read();
                self.stack.push(self.instruction_pointer);
                self.instruction_pointer = a;
            }
            18 => {
                if self.stack.is_empty()
                { self.running = false; } else {
                    self.instruction_pointer = self.stack.pop().expect("Empty stack after check");
                }
            }
            19 => {
                let ch: u16 = self.fetch_read();
                let ch: char = std::char::from_u32(ch.into()).expect("Invalid char");
                print!("{}", ch);
            }
            21 => (), // NoOp
            _ => panic!("Op not implemented: {}@{}", op, self.instruction_pointer - 1),
        }
    }
    pub fn dump(&self){
        println!("\nVM:\n R:{:?}\n St:{:?}\n IP: {}",self.registers,self.stack,self.instruction_pointer);
    }
    pub fn run(&mut self) {
        while self.running {
            if self.instruction_pointer == 9999 {
                self.dumping = true;
            }
            if self.dumping {
                self.dump();
            }
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
