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
    pub fn new() -> Brainfuck<Int> {
        Brainfuck {
            code: vec![],
            loop_stack: Vec::<usize>::new(),
            phantom: std::marker::PhantomData
        }
    }

    pub fn from_file(filename: &str) -> std::result::Result<Self, Error> {
        let code = std::fs::read_to_string(filename)?;
        Brainfuck::<Int>::from_str(&code)
    }

    pub fn from_str(input: &str) -> std::result::Result<Self, Error> {
        let mut code = Self::new();
        code.parse(input)?;
        Ok(code)
    }

    pub fn iter(&self) -> std::slice::Iter<Instruct<Int>> {
        self.code.iter()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.code.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&Instruct<Int>> {
        self.code.get(index)
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
                            return Err(Error::UnmatchedLoopEnd { lineno, column });
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
            return Err(Error::UnmatchedLoopStart { lineno: begin_lineno, column: begin_column });
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

    pub fn push_add_to(&mut self, val: isize) {
        self.code.push(Instruct::AddTo(val));
    }

    pub fn push_sub_from(&mut self, val: isize) {
        self.code.push(Instruct::SubFrom(val));
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
        let ptr = self.loop_stack.pop().expect("unmatched ']'");
        self.code.push(Instruct::LoopEnd(ptr));
        let end_ptr = self.code.len();
        std::mem::replace(&mut self.code[ptr], Instruct::LoopStart(end_ptr));
    }

    pub fn push_write_str(&mut self, val: Vec<u8>) {
        self.code.push(Instruct::WriteStr(val));
    }

    pub fn push(&mut self, instr: &Instruct<Int>) {
        match instr {
            Instruct::Move(off)     => self.push_move(*off),
            Instruct::Add(val)      => self.push_add(*val),
            Instruct::Set(val)      => self.push_set(*val),
            Instruct::AddTo(off)    => self.push_add_to(*off),
            Instruct::SubFrom(off)  => self.push_sub_from(*off),
            Instruct::Read          => self.push_read(),
            Instruct::Write         => self.push_write(),
            Instruct::LoopStart(_)  => self.push_loop_start(),
            Instruct::LoopEnd(_)    => self.push_loop_end(),
            Instruct::WriteStr(val) => self.push_write_str(val.to_vec())
        }
    }

    pub fn find_set_before(&self, mut index: usize) -> Option<Int> {
        if index >= self.len() {
            if self.len() == 0 {
                return None;
            }
            index = self.len() - 1;
        }
        let mut ptr = 0isize;
        while index > 0 {
            index -= 1;
            match self.code[index] {
                Instruct::Set(val) => {
                    if ptr == 0 {
                        return Some(val);
                    }
                },
                Instruct::Move(off) => {
                    ptr += off;
                },
                Instruct::Add(_) | Instruct::Read => {
                    if ptr == 0 {
                        return None;
                    }
                },
                Instruct::AddTo(_) | Instruct::SubFrom(_) => {
                    // XXX: why does it break when I restrict this return to if ptr + off == 0?
                    return None;
                },
                Instruct::LoopStart(_) => return None,
                // TODO: if loop doesn't move ptr overall and isn't touching *ptr it can be skipped
                Instruct::LoopEnd(_) => {
                    if ptr == 0 {
                        return Some(Int::zero());
                    }
                    return None;
                },
                Instruct::Write | Instruct::WriteStr(_) => {},
            }
        }
        return None;
    }

    pub fn optimize(&self, options: optimize::Options) -> std::io::Result<Self> {
        let mut code = if options.fold { optimize::fold(self) } else { self.clone() };
        if options.set      { code = optimize::set(&code); }
        if options.add_to   { code = optimize::add_to(&code); }
        if options.write    { code = optimize::write(&code); }
        if options.deadcode { code = optimize::deadcode(&code); }
        if options.fold     { code = optimize::fold(&code); }
        if options.skip     { code = optimize::skip(&code); }
        if options.constexpr {
            code = optimize::constexpr(&code, options.constexpr_echo)?;

            if options.fold     { code = optimize::fold(&code); }
            if options.set      { code = optimize::set(&code); }
            if options.add_to   { code = optimize::add_to(&code); }
            if options.write    { code = optimize::write(&code); }
            if options.deadcode { code = optimize::deadcode(&code); }
            if options.fold     { code = optimize::fold(&code); }
            if options.skip     { code = optimize::skip(&code); }
        }
        return Ok(code);
    }

    pub fn exec(&self) -> std::io::Result<()> {
        let mut mem = Vec::<Int>::new();
        let mut ptr = 0usize;
        let mut pc  = 0usize;
        let mut need_flush = false;

        loop {
            if let Some(instr) = self.code.get(pc) {
                match *instr {
                    Instruct::Move(off) => {
                        pc += 1;
                        if -(ptr as isize) > off {
                            let diff = (-(ptr as isize) - off) as usize;
                            let chunk = vec![Int::zero(); diff];
                            mem.splice(..0, chunk);
                            ptr += diff;
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

                    Instruct::AddTo(off) => {
                        pc += 1;
                        if let Some(val) = mem.get(ptr) {
                            let val = *val;
                            if val != Int::zero() {
                                if -(ptr as isize) > off {
                                    let diff = (-(ptr as isize) - off) as usize;
                                    let chunk = vec![Int::zero(); diff];
                                    mem.splice(..0, chunk);
                                    ptr += diff;
                                    mem[0] = val;
                                } else {
                                    let target_ptr = (ptr as isize + off) as usize;
                                    if target_ptr >= mem.len() {
                                        mem.resize(target_ptr + 1, Int::zero());
                                    }
                                    mem[target_ptr] = mem[target_ptr].wrapping_add(&val);
                                }
                            }
                        }
                    },

                    Instruct::SubFrom(off) => {
                        pc += 1;
                        if let Some(val) = mem.get(ptr) {
                            let val = -*val;
                            if val != Int::zero() {
                                if -(ptr as isize) > off {
                                    let diff = (-(ptr as isize) - off) as usize;
                                    let chunk = vec![Int::zero(); diff];
                                    mem.splice(..0, chunk);
                                    ptr += diff;
                                    mem[0] = val;
                                } else {
                                    let target_ptr = (ptr as isize + off) as usize;
                                    if target_ptr >= mem.len() {
                                        mem.resize(target_ptr + 1, Int::zero());
                                    }
                                    mem[target_ptr] = mem[target_ptr].wrapping_add(&val);
                                }
                            }
                        }
                    },

                    Instruct::Read => {
                        pc += 1;
                        let mut data = [0u8];
                        if need_flush {
                            std::io::stdout().flush()?;
                            need_flush = false;
                        }
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
                        let byte = mem[ptr].get_least_byte();
                        std::io::stdout().write_all(&[byte])?;
                        need_flush = byte != b'\n';
                    },

                    Instruct::WriteStr(ref data) => {
                        pc += 1;
                        if data.len() > 0 {
                            std::io::stdout().write_all(data)?;
                            need_flush = data[data.len() - 1] != b'\n';
                        }
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

        if need_flush {
            std::io::stdout().flush()?;
        }

        Ok(())
    }

    pub fn write_debug(&self, out: &mut Write) -> std::io::Result<()> {
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

                Instruct::AddTo(off) => {
                    indent(out, nesting)?;
                    write!(out, "add_to {:?}\n", off)?;
                },

                Instruct::SubFrom(off) => {
                    indent(out, nesting)?;
                    write!(out, "sub_from {:?}\n", off)?;
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

    pub fn write_bf(&self, out: &mut Write) -> std::io::Result<()> {
        let mut index = 0usize;
        loop {
            if let Some(instr) = self.code.get(index) {
                match *instr {
                    Instruct::Move(off) => {
                        if off > 0 {
                            print_repeat(out, b">", off as usize)?;
                        } else {
                            print_repeat(out, b"<", -off as usize)?;
                        }
                        index += 1;
                    }

                    Instruct::Add(val) => {
                        if val > Int::zero() {
                            print_repeat(out, b"+", val.wrapping_usize())?;
                        } else {
                            print_repeat(out, b"-", (-val).wrapping_usize())?;
                        }
                        index += 1;
                    },

                    Instruct::Set(val) => {
                        write!(out, "[-]")?;
                        if val > Int::zero() {
                            print_repeat(out, b"+", val.wrapping_usize())?;
                        } else {
                            print_repeat(out, b"-", (-val).wrapping_usize())?;
                        }
                        index += 1;
                    },

                    Instruct::AddTo(_) | Instruct::SubFrom(_) => {
                        let mut offsets = Vec::new();
                        index += 1;
                        loop {
                            match self.code.get(index) {
                                Some(Instruct::AddTo(off)) => {
                                    offsets.push((off, b"+"));
                                    index += 1;
                                },
                                Some(Instruct::SubFrom(off)) => {
                                    offsets.push((off, b"-"));
                                    index += 1;
                                },
                                _ => break
                            }
                        }
                        write!(out, "[-")?;
                        let mut current_off = 0isize;
                        for (target_off, instr) in offsets {
                            let off = target_off - current_off;
                            if off > 0 {
                                print_repeat(out, b">", off as usize)?;
                                out.write_all(instr)?;
                                print_repeat(out, b"<", off as usize)?;
                            } else {
                                print_repeat(out, b"<", -off as usize)?;
                                out.write_all(instr)?;
                                print_repeat(out, b">", -off as usize)?;
                            }
                            current_off = off;
                        }
                        if current_off > 0 {
                            print_repeat(out, b"<", current_off as usize)?;
                        } else if current_off < 0 {
                            print_repeat(out, b">", -current_off as usize)?;
                        }
                        write!(out, "]")?;

                        match self.code.get(index) {
                            Some(Instruct::Set(val)) if *val == Int::zero() => {
                                index += 1;
                            },
                            _ => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "this optimized brainfuck program cannot (easily) be converted back to brainfuck anymore"));
                            }
                        }
                    },

                    Instruct::Read => {
                        out.write_all(b",")?;
                        index += 1;
                    },

                    Instruct::Write => {
                        out.write_all(b".")?;
                        index += 1;
                    },

                    Instruct::LoopStart(_) => {
                        out.write_all(b"[")?;
                        index += 1;
                    },

                    Instruct::LoopEnd(_) => {
                        out.write_all(b"]")?;
                        index += 1;
                    },

                    Instruct::WriteStr(ref val) => {
                        for byte in val.iter() {
                            out.write_all(b"[-]")?;
                            print_repeat(out, b"+", *byte as usize)?;
                            out.write_all(b".")?;
                        }
                        index += 1;
                    }
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}

fn print_repeat(out: &mut Write, bytes: &[u8], count: usize) -> std::io::Result<()> {
    for _ in 0..count {
        out.write_all(bytes)?;
    }

    Ok(())
}