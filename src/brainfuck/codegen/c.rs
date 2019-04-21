extern crate num_traits;

use num_traits::Signed;
use std::io::Write;
use super::super::Brainfuck;
use super::super::integer::BrainfuckInteger;
use super::super::instruct::Instruct;
use super::super::indent::indent;

pub fn generate<Int: BrainfuckInteger + Signed>(code: &Brainfuck<Int>, out: &mut Write) -> std::io::Result<()> {
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
