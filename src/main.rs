// some optimizations:
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

mod brainfuck;

extern crate num_traits;

use brainfuck::{Brainfuck, Error};
use brainfuck::optimize::Options;

fn main() -> std::result::Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        panic!("Usage: bfc <input-file>");
    }
    let code = std::fs::read_to_string(&args[1])?;
    let code = Brainfuck::<i64>::from_str(&code)?;
    let code = code.optimize(Options::default())?;

    brainfuck::codegen::c::compile(&code, "a.out", true)?;
    //code.debug_code(&mut file)?;

    //code.exec()?;

    Ok(())
}
