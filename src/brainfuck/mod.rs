pub mod codegen;
pub mod optimize;
pub mod error;
pub mod instruct;
pub mod integer;
pub mod indent;

extern crate num_traits;

use std::io::{Read, Write};
use num_traits::Signed;
pub use integer::BrainfuckInteger;
pub use error::Error;
pub use instruct::Instruct;
use indent::indent;

pub struct Brainfuck<Int: BrainfuckInteger + Signed> {
    code: Vec<Instruct<Int>>,
    loop_stack: Vec<usize>,
    phantom: std::marker::PhantomData<Int>
}

impl<Int: BrainfuckInteger + Signed> Clone for Brainfuck<Int> {
    fn clone(&self) -> Self {
        Brainfuck {
            code: self.code.to_vec(),
            loop_stack: self.loop_stack.to_vec(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<Int: BrainfuckInteger + Signed> Brainfuck<Int> {
    pub fn iter(&self) -> std::slice::Iter<Instruct<Int>> {
        self.code.iter()
    }

    pub fn new() -> Brainfuck<Int> {
        Brainfuck {
            code: vec![],
            loop_stack: Vec::<usize>::new(),
            phantom: std::marker::PhantomData
        }
    }

    pub fn from_str(input: &str) -> std::result::Result<Self, Error> {
        let mut code = Self::new();
        code.parse(input)?;
        Ok(code)
    }

    pub fn parse(&mut self, input: &str) -> std::result::Result<(), Error> {
        let mut it = input.chars();
        let mut lineno: usize = 1;
        let mut column: usize = 1;
        let mut begin_lineno: usize = 0;
        let mut begin_column: usize = 0;

        loop {
            if let Some(c) = it.next() {
                match c {
                    '<' => {
                        self.push_move(-1);
                        column += 1;
                    },
                    '>' => {
                        self.push_move(1);
                        column += 1;
                    },
                    '-' => {
                        self.push_add(-Int::one());
                        column += 1;
                    },
                    '+' => {
                        self.push_add(Int::one());
                        column += 1;
                    },
                    '[' => {
                        self.push_loop_start();
                        begin_lineno = lineno;
                        begin_column = column;
                        column += 1;
                    },
                    ']' => {
                        if self.loop_stack.len() == 0 {
                            return Err(Error::UnmatchedLoopEnd(lineno, column));
                        }
                        self.push_loop_end();
                        column += 1;
                    },
                    '.' => {
                        self.push_write();
                        column += 1;
                    },
                    ',' => {
                        self.push_read();
                        column += 1;
                    },
                    '\n' => {
                        column = 1;
                        lineno += 1;
                    }
                    _ => {
                        column += 1;
                    }
                }
            } else {
                break;
            }
        }

        if self.loop_stack.len() > 0 {
            return Err(Error::UnmatchedLoopStart(lineno, column, begin_lineno, begin_column));
        }

        Ok(())
    }

    pub fn push_move(&mut self, val: isize) {
        self.code.push(Instruct::Move(val));
    }

    pub fn push_add(&mut self, val: Int) {
        self.code.push(Instruct::Add(val));
    }

    pub fn push_set(&mut self, val: Int) {
        self.code.push(Instruct::Set(val));
    }

    pub fn push_read(&mut self) {
        self.code.push(Instruct::Read);
    }

    pub fn push_write(&mut self) {
        self.code.push(Instruct::Write);
    }

    pub fn push_loop_start(&mut self) {
        self.loop_stack.push(self.code.len());
        self.code.push(Instruct::LoopStart(std::usize::MAX));
    }

    pub fn push_loop_end(&mut self) {
        let ptr = self.loop_stack.pop().unwrap();
        self.code.push(Instruct::LoopEnd(ptr));
        let end_ptr = self.code.len();
        std::mem::replace(&mut self.code[ptr], Instruct::LoopStart(end_ptr));
    }

    pub fn push_write_str(&mut self, val: Vec<u8>) {
        self.code.push(Instruct::WriteStr(val));
    }

    pub fn push(&mut self, instr: Instruct<Int>) {
        self.push_ref(&instr);
    }

    pub fn push_ref(&mut self, instr: &Instruct<Int>) {
        match instr {
            Instruct::Move(val)     => self.push_move(*val),
            Instruct::Add(val)      => self.push_add(*val),
            Instruct::Set(val)      => self.push_set(*val),
            Instruct::Read          => self.push_read(),
            Instruct::Write         => self.push_write(),
            Instruct::LoopStart(_)  => self.push_loop_start(),
            Instruct::LoopEnd(_)    => self.push_loop_end(),
            Instruct::WriteStr(val) => self.push_write_str(val.to_vec())
        }
    }

    pub fn optimize(&self, options: optimize::Options) -> std::io::Result<Self> {
        let mut code = if options.fold { optimize::fold(self) } else { self.clone() };
        if options.set   { code = optimize::set(&code); }
        if options.write { code = optimize::write(&code); }
        if options.fold  { code = optimize::fold(&code); }
        if options.constexpr {
            code = optimize::constexpr(&code, options.constexpr_echo)?;
        }
        if options.fold  { code = optimize::fold(&code); }
        if options.set   { code = optimize::set(&code); }
        if options.write { code = optimize::write(&code); }
        if options.fold  { code = optimize::fold(&code); }
        return Ok(code);
    }

    pub fn exec(&self) -> std::io::Result<()> {
        let mut mem = Vec::<Int>::new();
        let mut ptr = 0usize;
        let mut pc  = 0usize;

        loop {
            if let Some(instr) = self.code.get(pc) {
                match *instr {
                    Instruct::Move(off) => {
                        pc += 1;
                        ptr = ((ptr as isize) + off) as usize;
                    },

                    Instruct::Add(val) => {
                        pc += 1;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        mem[ptr] = mem[ptr].wrapping_add(&val);
                    },

                    Instruct::Set(val) => {
                        pc += 1;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        mem[ptr] = val;
                    },

                    Instruct::Read => {
                        pc += 1;
                        let mut data = [0u8];
                        let count = std::io::stdin().read(&mut data)?;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        if count < 1 {
                            mem[ptr] = -Int::one();
                        } else {
                            mem[ptr] = Int::from_byte(data[0]);
                        }
                    },

                    Instruct::Write => {
                        pc += 1;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        let data = [mem[ptr].get_least_byte()];
                        std::io::stdout().write_all(&data)?;
                    },

                    Instruct::WriteStr(ref data) => {
                        pc += 1;
                        std::io::stdout().write_all(data)?;
                    },

                    Instruct::LoopStart(pc_false) => {
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        if mem[ptr] == Int::zero() {
                            pc = pc_false;
                        } else {
                            pc += 1;
                        }
                    },

                    Instruct::LoopEnd(pc_loop_start) => {
                        pc = pc_loop_start;
                    }
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn debug_code(&self, out: &mut Write) -> std::io::Result<()> {
        let mut nesting: usize = 0;
        for instr in self.code.iter() {

            match instr {
                Instruct::Move(val) => {
                    indent(out, nesting)?;
                    write!(out, "move {:?}\n", val)?;
                }

                Instruct::Add(val) => {
                    indent(out, nesting)?;
                    write!(out, "add {:?}\n", val)?;
                },

                Instruct::Set(val) => {
                    indent(out, nesting)?;
                    write!(out, "set {:?}\n", val)?;
                },

                Instruct::Read => {
                    indent(out, nesting)?;
                    out.write_all(b"read\n")?;
                },

                Instruct::Write => {
                    indent(out, nesting)?;
                    out.write_all(b"write\n")?;
                },

                Instruct::LoopStart(_) => {
                    indent(out, nesting)?;
                    out.write_all(b"loop {\n")?;
                    nesting += 1;
                },

                Instruct::LoopEnd(_) => {
                    nesting -= 1;
                    indent(out, nesting)?;
                    out.write_all(b"}\n")?;
                },

                Instruct::WriteStr(val) => {
                    indent(out, nesting)?;
                    write!(out, "write {:?}\n", val)?;
                }
            }
        }

        Ok(())
    }
}