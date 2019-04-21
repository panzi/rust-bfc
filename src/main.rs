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

use std::fs::File;
use brainfuck::Brainfuck;
use brainfuck::error::Error;
use brainfuck::codegen::c::generate as generate_c_code;

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
