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

trait BFInt: PrimInt + WrappingShl + WrappingAdd + std::fmt::Debug {
    fn c_type() -> &'static str;
    fn get_least_byte(self) -> u8;
    fn from_byte(value: u8) -> Self;
}

impl BFInt for u8 {
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

impl BFInt for i8 {
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

impl BFInt for u16 {
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

impl BFInt for i16 {
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

impl BFInt for u32 {
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

impl BFInt for i32 {
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

impl BFInt for u64 {
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

impl BFInt for i64 {
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

impl BFInt for isize {
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

impl BFInt for usize {
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

enum Instruct<Int: BFInt + Signed> {
    Move(isize),
    Add(Int),
    Set(Int),
    Read,
    Write,
    LoopStart(usize),
    LoopEnd(usize),
    WriteStr(Vec<u8>)
}

const MOVE       :u8 = 1;
const ADD        :u8 = 2;
const SET        :u8 = 3;
const READ       :u8 = 4;
const WRITE      :u8 = 5;
const LOOP_START :u8 = 6;
const LOOP_END   :u8 = 7;
const WRITE_STR  :u8 = 8;

struct Bytecode<Int: BFInt + Signed> {
    bytes: Vec<u8>,
    loop_stack: Vec<usize>,
    phantom: std::marker::PhantomData<Int>
}

struct BytecodeIter<'a, Int: BFInt + Signed> {
    code: &'a Bytecode<Int>,
    index: usize,
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

impl<Int: BFInt + Signed> Bytecode<Int> {
    pub fn iter<'a>(&'a self) -> BytecodeIter<'a, Int> {
        BytecodeIter {
            code: &self,
            index: 0,
            phantom: std::marker::PhantomData
        }
    }

    pub fn new() -> Bytecode<Int> {
        Bytecode {
            bytes: vec![],
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
        self.bytes.push(MOVE);
        self.push_val::<isize>(val);
    }

    pub fn push_add(&mut self, val: Int) {
        self.bytes.push(ADD);
        self.push_int(val);
    }

    pub fn push_set(&mut self, val: Int) {
        self.bytes.push(SET);
        self.push_int(val);
    }

    pub fn push_read(&mut self) {
        self.bytes.push(READ);
    }

    pub fn push_write(&mut self) {
        self.bytes.push(WRITE);
    }

    pub fn push_loop_start(&mut self) {
        self.loop_stack.push(self.bytes.len());
        self.bytes.push(LOOP_START);
        self.push_usize(std::usize::MAX);
    }

    pub fn push_loop_end(&mut self) {
        let ptr = self.loop_stack.pop().unwrap();
        self.bytes.push(LOOP_END);
        self.push_usize(ptr);
        self.set_val::<usize>(ptr + 1, self.bytes.len());
    }

    pub fn push_write_str(&mut self, val: &[u8]) {
        self.bytes.push(WRITE_STR);
        self.push_usize(val.len());
        self.bytes.extend(val);
    }

    pub fn push(&mut self, instr: Instruct<Int>) {
        match instr {
            Instruct::Move(val)    => self.push_move(val),
            Instruct::Add(val)     => self.push_add(val),
            Instruct::Set(val)     => self.push_set(val),
            Instruct::Read         => self.push_read(),
            Instruct::Write        => self.push_write(),
            Instruct::LoopStart(_) => self.push_loop_start(),
            Instruct::LoopEnd(_)   => self.push_loop_end(),
            Instruct::WriteStr(ref val) => self.push_write_str(val)
        }
    }

    fn push_val<Val: BFInt>(&mut self, value: Val) {
        let bytecount = std::mem::size_of::<Val>();
        for index in (0..=bytecount * 8 - 8).rev().step_by(8) {
            let byte: u8 = (value >> index).get_least_byte();
            self.bytes.push(byte);
        }
    }

    fn set_val<Val: BFInt>(&mut self, mut index: usize, value: Val) {
        let bytecount = std::mem::size_of::<Val>();
        for i in (0..=bytecount * 8 - 8).rev().step_by(8) {
            let byte: u8 = (value >> i).get_least_byte();
            self.bytes[index] = byte;
            index += 1;
        }
    }

    fn push_int(&mut self, value: Int) {
        self.push_val::<Int>(value);
    }

    fn push_usize(&mut self, value: usize) {
        self.push_val::<usize>(value);
    }

    fn get(&self, mut index: usize) -> Option<(Instruct<Int>, usize)> {
        if index >= self.bytes.len() {
            return None;
        }

        let instr = self.bytes[index];
        index += 1;

        match instr {
            self::MOVE => {
                if let Some((val, index)) = self.get_val::<isize>(index) {
                    return Some((Instruct::<Int>::Move(val), index));
                }
            },
            self::ADD => {
                if let Some((val, index)) = self.get_int(index) {
                    return Some((Instruct::<Int>::Add(val), index));
                }
            },
            self::SET => {
                if let Some((val, index)) = self.get_int(index) {
                    return Some((Instruct::<Int>::Set(val), index));
                }
            },
            self::READ => {
                return Some((Instruct::<Int>::Read, index));
            },
            self::WRITE => {
                return Some((Instruct::<Int>::Write, index));
            },
            self::LOOP_START => {
                if let Some((val, index)) = self.get_val::<usize>(index) {
                    return Some((Instruct::<Int>::LoopStart(val), index));
                }
            },
            self::LOOP_END => {
                if let Some((val, index)) = self.get_val::<usize>(index) {
                    return Some((Instruct::<Int>::LoopEnd(val), index));
                }
            },
            self::WRITE_STR => {
                if let Some((len, index)) = self.get_usize(index) {
                    let end_index = index + len;
                    if end_index > self.bytes.len() {
                        return None;
                    }
                    let data = self.bytes[index..end_index].to_vec();
                    return Some((Instruct::<Int>::WriteStr(data), end_index));
                }
            },
            _ => {}
        }

        return None;
    }

    fn get_val<Val: BFInt>(&self, index: usize) -> Option<(Val, usize)> {
        let bytecount = std::mem::size_of::<Val>();
        let end_index = index + bytecount;
        if end_index > self.bytes.len() {
            return None;
        }

        let mut value = Val::zero();
        for i in index..end_index {
            value = value.wrapping_shl(8) | Val::from_byte(self.bytes[i]);
        }

        return Some((value, end_index));
    }

    #[inline]
    fn get_int(&self, index: usize) -> Option<(Int, usize)> {
        self.get_val::<Int>(index)
    }

    #[inline]
    fn get_usize(&self, index: usize) -> Option<(usize, usize)> {
        self.get_val::<usize>(index)
    }

    pub fn optimize_fold(&self) -> Self {
        let mut code = Self::new();
        let mut it = self.iter();

        loop {
            if let Some(instr) = it.next() {
                match instr {
                    Instruct::Move(val1) => {
                        let mut val = val1;
                        while let Some(Instruct::Move(val2)) = it.peek() {
                            it.next();
                            val += val2;
                        }
                        code.push_move(val);
                    },
                    Instruct::Add(val1) => {
                        let mut val = val1;
                        while let Some(Instruct::Add(val2)) = it.peek() {
                            it.next();
                            val = val + val2;
                        }
                        code.push_add(val);
                    },
                    Instruct::Set(val1) => {
                        let mut val = val1;
                        while let Some(Instruct::Set(val2)) = it.peek() {
                            it.next();
                            val = val2;
                        }
                        code.push_set(val);
                    },
                    _ => code.push(instr)
                }
            } else {
                break;
            }
        }

        return code;
    }

    pub fn optimize_set(&self) -> Self {
        let mut code = Self::new();
        let mut it = self.iter();

        loop {
            match it.peek4() {
                Some((Instruct::LoopStart(_), Instruct::Add(_), Instruct::LoopEnd(_), Instruct::Add(val))) => {
                    it.next(); it.next(); it.next(); it.next();
                    code.push_set(val);
                    continue;
                },
                _ => {}
            }

            if let Some((Instruct::LoopStart(_), Instruct::Add(_), Instruct::LoopEnd(_))) = it.peek3() {
                it.next(); it.next(); it.next();
                code.push_set(Int::zero());
                continue;
            }

            if let Some(instr) = it.next() {
                code.push(instr);
            } else {
                break;
            }
        }

        return code;
    }

    fn optimize_write_str(&self, it: &mut BytecodeIter<Int>, data: &mut Vec<u8>) {
        let mut last_val = if data.len() > 0 { data[data.len() - 1] } else { 0u8 };
        loop {
            if let Some((Instruct::Set(val), Instruct::Write)) = it.peek2() {
                it.next(); it.next();
                last_val = val.get_least_byte();
                data.push(last_val);
            } else if let Some(Instruct::Write) = it.peek() {
                it.next();
                data.push(last_val);
            } else if let Some(Instruct::WriteStr(data2)) = it.peek() {
                it.next();
                data.extend(data2);
            } else {
                break;
            }
        }
    }

    pub fn optimize_write(&self) -> Self {
        let mut code = Self::new();
        let mut it = self.iter();

        loop {
            match it.peek2() {
                Some((Instruct::Set(val), Instruct::Write)) => {
                    it.next(); it.next();
                    let mut data = vec![val.get_least_byte()];
                    self.optimize_write_str(&mut it, &mut data);
                    code.push_write_str(&data);
                    continue;
                },
                _ => {}
            }

            if let Some(instr) = it.next() {
                if let Instruct::WriteStr(data) = instr {
                    let mut data = data;
                    self.optimize_write_str(&mut it, &mut data);
                    code.push_write_str(&data);
                } else {
                    code.push(instr);
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
            if let Some((instr, pc2)) = self.get(pc) {
                match instr {
                    Instruct::Move(off) => {
                        pc = pc2;
                        if off == std::isize::MIN || (ptr as isize) < -off {
                            // XXX: what to do when pointer < 0?
                            code.push(instr);
                            break;
                        }
                        ptr = ((ptr as isize) + off) as usize;
                    },

                    Instruct::Add(val) => {
                        pc = pc2;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        mem[ptr] = mem[ptr].wrapping_add(&val);
                    },

                    Instruct::Set(val) => {
                        pc = pc2;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        mem[ptr] = val;
                    },

                    Instruct::Read => {
                        pc = pc2;
                        code.push(instr);
                        break;
                    },

                    Instruct::Write => {
                        pc = pc2;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        let data = [mem[ptr].get_least_byte()];
                        std::io::stdout().write_all(&data); // DEBUG
                        code.push_write_str(&data);
                    },

                    Instruct::WriteStr(ref data) => {
                        pc = pc2;
                        std::io::stdout().write_all(data); // DEBUG
                        code.push_write_str(data);
                    },

                    Instruct::LoopStart(pc_false) => {
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        if mem[ptr] == Int::zero() {
                            pc = pc_false;
                        } else {
                            pc = pc2;
                        }
                    },

                    Instruct::LoopEnd(pc2) => {
                        pc = pc2;
                    }
                }
            } else {
                break;
            }
        }

        if pc < self.bytes.len() {
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

            while let Some((instr, pc2)) = self.get(pc) {
                code.push(instr);
                pc = pc2;
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
            if let Some((instr, pc_next)) = self.get(pc) {
                match instr {
                    Instruct::Move(off) => {
                        pc = pc_next;
                        ptr = ((ptr as isize) + off) as usize;
                    },

                    Instruct::Add(val) => {
                        pc = pc_next;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        mem[ptr] = mem[ptr].wrapping_add(&val);
                    },

                    Instruct::Set(val) => {
                        pc = pc_next;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        mem[ptr] = val;
                    },

                    Instruct::Read => {
                        pc = pc_next;
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
                        pc = pc_next;
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        let data = [mem[ptr].get_least_byte()];
                        std::io::stdout().write_all(&data)?;
                    },

                    Instruct::WriteStr(ref data) => {
                        pc = pc_next;
                        std::io::stdout().write_all(data)?;
                    },

                    Instruct::LoopStart(pc_false) => {
                        if ptr >= mem.len() {
                            mem.resize(ptr + 1, Int::zero());
                        }
                        if mem[ptr] == Int::zero() {
                            pc = pc_false;
                        } else {
                            pc = pc_next;
                        }
                    },

                    Instruct::LoopEnd(pc_start) => {
                        pc = pc_start;
                    }
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn dump(&self, out: &mut Write) -> std::io::Result<()> {
        out.write_all(self.bytes.as_slice())
    }

    pub fn debug_code(&self, out: &mut Write) -> std::io::Result<()> {
        let mut nesting: usize = 0;
        for instr in self.iter() {

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

impl<'a, Int: BFInt + Signed> BytecodeIter<'a, Int> {
    fn read_val<Val: BFInt>(&mut self) -> Option<Val> {
        let bytecount = std::mem::size_of::<Val>();
        if self.index + bytecount > self.code.bytes.len() {
            return None;
        }

        let mut value = Val::zero();
        for index in self.index..(self.index + bytecount) {
            value = value.wrapping_shl(8) | Val::from_byte(self.code.bytes[index]);
        }
        self.index += bytecount;

        return Some(value);
    }

    #[inline]
    fn read_int(&mut self) -> Option<Int> {
        self.read_val::<Int>()
    }

    #[inline]
    fn read_usize(&mut self) -> Option<usize> {
        self.read_val::<usize>()
    }

    fn peek(&self) -> Option<Instruct<Int>> {
        if let Some((instr, _)) = self.code.get(self.index) {
            return Some(instr);
        }
        None
    }

    fn peek2(&self) -> Option<(Instruct<Int>, Instruct<Int>)> {
        if let Some((instr1, index)) = self.code.get(self.index) {
            if let Some((instr2, _)) = self.code.get(index) {
                return Some((instr1, instr2));
            }
        }

        None
    }

    fn peek3(&self) -> Option<(Instruct<Int>, Instruct<Int>, Instruct<Int>)> {
        if let Some((instr1, index)) = self.code.get(self.index) {
            if let Some((instr2, index)) = self.code.get(index) {
                if let Some((instr3, _)) = self.code.get(index) {
                    return Some((instr1, instr2, instr3));
                }
            }
        }

        None
    }

    fn peek4(&self) -> Option<(Instruct<Int>, Instruct<Int>, Instruct<Int>, Instruct<Int>)> {
        if let Some((instr1, index)) = self.code.get(self.index) {
            if let Some((instr2, index)) = self.code.get(index) {
                if let Some((instr3, index)) = self.code.get(index) {
                    if let Some((instr4, _)) = self.code.get(index) {
                        return Some((instr1, instr2, instr3, instr4));
                    }
                }
            }
        }

        None
    }
}

impl<'a, Int: BFInt + Signed> Iterator for BytecodeIter<'a, Int> {
    type Item = Instruct<Int>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.code.bytes.len() {
            return None;
        }

        let instr = self.code.bytes[self.index];
        self.index += 1;

        match instr {
            self::MOVE => {
                let val = self.read_val::<isize>().expect(&format!("unexpected end of bytecode at index {}", self.index));
                return Some(Instruct::<Int>::Move(val));
            },
            self::ADD => {
                let val = self.read_int().expect(&format!("unexpected end of bytecode at index {}", self.index));
                return Some(Instruct::<Int>::Add(val));
            },
            self::SET => {
                let val = self.read_int().expect(&format!("unexpected end of bytecode at index {}", self.index));
                return Some(Instruct::<Int>::Set(val));
            },
            self::READ => {
                return Some(Instruct::<Int>::Read);
            },
            self::WRITE => {
                return Some(Instruct::<Int>::Write);
            },
            self::LOOP_START => {
                let val = self.read_val::<usize>().expect(&format!("unexpected end of bytecode at index {}", self.index));
                return Some(Instruct::<Int>::LoopStart(val));
            },
            self::LOOP_END => {
                let val = self.read_val::<usize>().expect(&format!("unexpected end of bytecode at index {}", self.index));
                return Some(Instruct::<Int>::LoopEnd(val));
            },
            self::WRITE_STR => {
                let len = self.read_usize().expect(&format!("unexpected end of bytecode at index {}", self.index));
                let end_index = self.index + len;
                if end_index > self.code.bytes.len() {
                    panic!(format!("unexpected end of bytecode at index {}", self.index));
                }
                let data = self.code.bytes[self.index..end_index].to_vec();
                self.index = end_index;
                return Some(Instruct::<Int>::WriteStr(data));
            },
            _ => {
                panic!(format!("illegal instruction {} at index {}", instr, self.index));
            }
        }
    }
}

fn indent(out: &mut Write, nesting: usize) -> std::io::Result<()> {
    for _ in 0..nesting {
        out.write_all(b"    ")?;
    }
    Ok(())
}

fn generate_c_code<Int: BFInt + Signed>(code: &Bytecode<Int>, out: &mut Write) -> std::io::Result<()> {
    write!(out, r##"
#include <stdio.h>
#include <sys/mman.h>

int main() {{
    {0}* mem = NULL; // TODO: mmap based memory handling with guard pages and singal handling
    {0}* ptr = mem;

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
                indent(out, nesting)?;
                write!(out, "fwrite(\"")?;

                for c in data.iter() {
                    match *c {
                        92u8 | 34u8 => {
                            out.write_all(&[92u8, *c])?;
                        },

                        10u8 => {
                            out.write_all(b"\\n")?;
                        },

                        0u8 => {
                            out.write_all(b"\\0")?;
                        },

                        13u8 => {
                            out.write_all(b"\\r")?;
                        },

                        9u8 => {
                            out.write_all(b"\\t")?;
                        },

                        11u8 => {
                            out.write_all(b"\\v")?;
                        },

                        8u8 => {
                            out.write_all(b"\\b")?;
                        },

                        c if c >= 32 && c <= 126 => {
                            out.write_all(&[c]);
                        },

                        _ => {
                            write!(out, "\\x{:02x}", c);
                        }
                    }
                }

                write!(out, "\", {}, 1, stdout);", data.len())?;
            },
        }
    }
    write!(out, r##"
    return 0;
}}
    "##)?;
    Ok(())
}

fn main() -> std::result::Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        panic!("Usage: bfc <input-file>");
    }
    let code = std::fs::read_to_string(&args[1])?;
    let code = Bytecode::<i64>::from_str(&code)?;
    let code = code.optimize();
    //generate_c_code(code, file)

    let mut file = File::create("out.c")?;
    generate_c_code(&code, &mut file)?;
    //code.debug_code(&mut file)?;

    //code.exec()?;

    Ok(())
}
