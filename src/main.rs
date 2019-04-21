// <{n}                   ->  ptr -= n
// >{n}                   ->  ptr += n
// -{n}                   -> *ptr -= n
// +{n}                   -> *ptr += n
// +{n}-{m}               -> *ptr += n - m // etc.
// [-{*}]                 -> *ptr  = 0
// [-{*}]+{n}             -> *ptr  = n
// .                      -> putchar(*ptr)
// ,                      -> *ptr = getchar()
// [                      -> while (*ptr) {
// ]                      -> }
// [-{*}]+{n}.[-{*}]+{m}. -> write(STDOUT_FILENO, (unsigned char)[] {n, m}, 2)

extern crate num_traits;

use num_traits::{PrimInt, Signed, WrappingShl, WrappingAdd};

use std::fs::File;
use std::io::{Read, Write};

trait BrainfuckInteger: PrimInt + WrappingShl + WrappingAdd + std::fmt::Debug {
    fn c_type() -> &'static str;
    fn get_least_byte(self) -> u8;
    fn from_byte(value: u8) -> Self;
}

impl BrainfuckInteger for u8 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        self
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value
    }

    fn c_type() -> &'static str {
        return "uint8_t";
    }
}

impl BrainfuckInteger for i8 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        self as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as i8
    }

    fn c_type() -> &'static str {
        return "int8_t";
    }
}

impl BrainfuckInteger for u16 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as u16
    }

    fn c_type() -> &'static str {
        return "uint16_t";
    }
}

impl BrainfuckInteger for i16 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as i16
    }

    fn c_type() -> &'static str {
        return "int16_t";
    }
}

impl BrainfuckInteger for u32 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as u32
    }

    fn c_type() -> &'static str {
        return "uint32_t";
    }
}

impl BrainfuckInteger for i32 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as i32
    }

    fn c_type() -> &'static str {
        return "int32_t";
    }
}

impl BrainfuckInteger for u64 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as u64
    }

    fn c_type() -> &'static str {
        return "uint64_t";
    }
}

impl BrainfuckInteger for i64 {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as i64
    }

    fn c_type() -> &'static str {
        return "int64_t";
    }
}

impl BrainfuckInteger for isize {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as isize
    }

    fn c_type() -> &'static str {
        return "ssize_t";
    }
}

impl BrainfuckInteger for usize {
    #[inline]
    fn get_least_byte(self) -> u8 {
        (self & 0xFF) as u8
    }

    #[inline]
    fn from_byte(value: u8) -> Self {
        value as usize
    }

    fn c_type() -> &'static str {
        return "size_t";
    }
}

#[derive(Debug)]
enum Instruct<Int: BrainfuckInteger + Signed> {
    Move(isize),
    Add(Int),
    Set(Int),
    Read,
    Write,
    LoopStart(usize),
    LoopEnd(usize),
    WriteStr(Vec<u8>)
}

struct Brainfuck<Int: BrainfuckInteger + Signed> {
    code: Vec<Instruct<Int>>,
    loop_stack: Vec<usize>,
    phantom: std::marker::PhantomData<Int>
}

#[derive(Debug)]
enum Error {
    IO(std::io::Error),
    UnmatchedLoopStart(usize, usize, usize, usize),
    UnmatchedLoopEnd(usize, usize)
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
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

fn indent(out: &mut Write, nesting: usize) -> std::io::Result<()> {
    for _ in 0..nesting {
        out.write_all(b"    ")?;
    }
    Ok(())
}

fn generate_c_code<Int: BrainfuckInteger + Signed>(code: &Brainfuck<Int>, out: &mut Write) -> std::io::Result<()> {
    write!(out, r##"#include <stdio.h>
#include <sys/mman.h>

{0}* mem = NULL; // TODO: mmap based memory handling with guard pages and singal handling
{0}* ptr = mem;

int main() {{
"##, Int::c_type())?;

    let mut nesting = 1usize;
    for instr in code.iter() {
        match instr {
            Instruct::Move(off) => {
                indent(out, nesting)?;
                write!(out, "ptr += {}\n", off)?;
            },

            Instruct::Add(val) => {
                indent(out, nesting)?;
                write!(out, "*ptr += {:?}\n", val)?;
            },

            Instruct::Set(val) => {
                indent(out, nesting)?;
                write!(out, "*ptr = {:?}\n", val)?;
            },

            Instruct::Read => {
                indent(out, nesting)?;
                write!(out, "*ptr = getchar()\n")?;
            },

            Instruct::Write => {
                indent(out, nesting)?;
                write!(out, "getchar(*ptr)\n")?;
            },

            Instruct::LoopStart(_) => {
                indent(out, nesting)?;
                write!(out, "while (*ptr) {{\n")?;
                nesting += 1;
            },

            Instruct::LoopEnd(_) => {
                nesting -= 1;
                indent(out, nesting)?;
                write!(out, "}}\n")?;
            },

            Instruct::WriteStr(data) => {
                if data.len() > 0 {
                    indent(out, nesting)?;
                    let multiline = if let Some(pos) = data.iter().position(|b| *b == b'\n') {
                        pos < data.len() - 1
                    } else {
                        false
                    };

                    if multiline {
                        write!(out, "fwrite(\n")?;
                        indent(out, nesting + 1)?;
                        write!(out, "\"")?;
                    } else {
                        write!(out, "fwrite(\"")?;
                    }

                    for c in data.iter() {
                        match *c {
                            b'\\' | b'"' => {
                                out.write_all(&[b'\\', *c])?;
                            },

                            b'\n' => {
                                if multiline {
                                    out.write_all(b"\\n\"\n")?;
                                    indent(out, nesting + 1)?;
                                    write!(out, "\"")?;
                                } else {
                                    out.write_all(b"\\n")?;
                                }
                            },

                            b'\0' => {
                                out.write_all(b"\\0")?;
                            },

                            b'\r' => {
                                out.write_all(b"\\r")?;
                            },

                            b'\t' => {
                                out.write_all(b"\\t")?;
                            },

                            11u8 => {
                                out.write_all(b"\\v")?;
                            },

                            8u8 => {
                                out.write_all(b"\\b")?;
                            },

                            c if c >= 32 && c <= 126 => {
                                out.write_all(&[c])?;
                            },

                            _ => {
                                write!(out, "\\x{:02x}", c)?;
                            }
                        }
                    }

                    write!(out, "\", {}, 1, stdout);", data.len())?;
                }
            },
        }
    }
    write!(out, r##"
    return 0;
}}"##)?;
    Ok(())
}

fn main() -> std::result::Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        panic!("Usage: bfc <input-file>");
    }
    let code = std::fs::read_to_string(&args[1])?;
    let code = Brainfuck::<i64>::from_str(&code)?;
    let code = code.optimize();
    //generate_c_code(code, file)

    let mut file = File::create("out.c")?;
    generate_c_code(&code, &mut file)?;
    //code.debug_code(&mut file)?;

    //code.exec()?;

    Ok(())
}
