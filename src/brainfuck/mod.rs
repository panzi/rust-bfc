pub mod codegen;
pub mod error;
pub mod instruct;
pub mod integer;
pub mod indent;

extern crate num_traits;

use std::io::{Read, Write};
use num_traits::Signed;
use integer::BrainfuckInteger;
use error::Error;
use instruct::Instruct;
use indent::indent;

pub struct Brainfuck<Int: BrainfuckInteger + Signed> {
    code: Vec<Instruct<Int>>,
    loop_stack: Vec<usize>,
    phantom: std::marker::PhantomData<Int>
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

    pub fn optimize_fold(&self) -> Self {
        let mut code = Self::new();
        let mut index = 0usize;

        loop {
            if let Some(instr) = self.code.get(index) {
                index += 1;
                match *instr {
                    Instruct::Move(val1) => {
                        let mut val = val1;
                        while let Some(Instruct::Move(val2)) = self.code.get(index) {
                            index += 1;
                            val += *val2;
                        }
                        code.push_move(val);
                    },
                    Instruct::Add(val1) => {
                        let mut val = val1;
                        while let Some(Instruct::Add(val2)) = self.code.get(index) {
                            index += 1;
                            val = val + *val2;
                        }
                        code.push_add(val);
                    },
                    Instruct::Set(val1) => {
                        let mut val = val1;
                        while let Some(Instruct::Set(val2)) = self.code.get(index) {
                            index += 1;
                            val = *val2;
                        }
                        code.push_set(val);
                    },
                    _ => code.push_ref(instr)
                }
            } else {
                break;
            }
        }

        return code;
    }

    pub fn optimize_set(&self) -> Self {
        let mut code = Self::new();
        let mut index = 0usize;

        loop {
            match (self.code.get(index), self.code.get(index + 1), self.code.get(index + 2), self.code.get(index + 3)) {
                (Some(Instruct::LoopStart(_)), Some(Instruct::Add(_)), Some(Instruct::LoopEnd(_)), Some(Instruct::Add(val))) => {
                    index += 4;
                    code.push_set(*val);
                    continue;
                },
                _ => {}
            }

            if let (Some(Instruct::LoopStart(_)), Some(Instruct::Add(_)), Some(Instruct::LoopEnd(_))) =
                    (self.code.get(index), self.code.get(index + 1), self.code.get(index + 2)) {
                index += 3;
                code.push_set(Int::zero());
                continue;
            }

            if let Some(instr) = self.code.get(index) {
                index += 1;
                code.push_ref(instr);
            } else {
                break;
            }
        }

        return code;
    }

    fn optimize_write_str(&self, mut index: usize, data: &mut Vec<u8>) -> usize {
        let mut last_val = data[data.len() - 1];
        loop {
            if let (Some(Instruct::Set(val)), Some(Instruct::Write)) = (self.code.get(index), self.code.get(index + 1)) {
                index += 2;
                last_val = val.get_least_byte();
                data.push(last_val);
            } else if let Some(Instruct::Write) = self.code.get(index) {
                index += 1;
                data.push(last_val);
            } else if let Some(Instruct::WriteStr(data2)) = self.code.get(index) {
                index += 1;
                data.extend(data2);
            } else {
                break;
            }
        }
        return index;
    }

    pub fn optimize_write(&self) -> Self {
        let mut code = Self::new();
        let mut index = 0usize;

        loop {
            match (self.code.get(index), self.code.get(index + 1)) {
                (Some(Instruct::Set(val)), Some(Instruct::Write)) => {
                    index += 2;
                    let mut data = vec![val.get_least_byte()];
                    index = self.optimize_write_str(index, &mut data);
                    code.push_write_str(data);
                    continue;
                },
                _ => {}
            }

            if let Some(instr) = self.code.get(index) {
                index += 1;
                if let Instruct::WriteStr(data) = instr {
                    if data.len() > 0 {
                        let mut data = data.to_vec();
                        index = self.optimize_write_str(index, &mut data);
                        code.push_write_str(data);
                    }
                } else {
                    code.push_ref(instr);
                }
            } else {
                break;
            }
        }

        return code;
    }

    pub fn optimize_constexpr(&self) -> Self {
        let mut code = Self::new();
        let mut mem = Vec::<Int>::new();
        let mut ptr = 0usize;
        let mut pc  = 0usize;

        loop {
            if let Some(instr) = self.code.get(pc) {
                match *instr {
                    Instruct::Move(off) => {
                        pc += 1;
                        if off == std::isize::MIN || (ptr as isize) < -off {
                            // XXX: what to do when pointer < 0?
                            code.push_move(off);
                            break;
                        }
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
                        code.push_read();
                        break;
                    },

                    Instruct::Write => {
                        pc += 1;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        let data = vec![mem[ptr].get_least_byte()];
                        std::io::stdout().write_all(&data); // DEBUG
                        code.push_write_str(data);
                    },

                    Instruct::WriteStr(ref data) => {
                        pc += 1;
                        std::io::stdout().write_all(data); // DEBUG
                        code.push_write_str(data.to_vec());
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

        if pc < self.code.len() {
            let mut current_ptr = ptr;
            for (target_ptr, val) in mem.iter().enumerate() {
                if *val != Int::zero() {
                    if current_ptr != target_ptr {
                        let off = (target_ptr as isize) - (ptr as isize);
                        code.push_move(off);
                        current_ptr = target_ptr;
                    }
                    code.push_set(*val);
                }
            }

            if current_ptr != ptr {
                let off = (ptr as isize) - (ptr as isize);
                code.push_move(off);
            }

            while let Some(instr) = self.code.get(pc) {
                code.push_ref(instr);
                pc += 1;
            }
        }

        return code;
    }

    pub fn optimize(&self) -> Self {
        let code = self.optimize_fold();
        let code = code.optimize_set();
        let code = code.optimize_write();
        let code = code.optimize_fold();
        let code = code.optimize_constexpr();
        let code = code.optimize_fold();
        let code = code.optimize_set();
        let code = code.optimize_write();
        let code = code.optimize_fold();
        return code;
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
