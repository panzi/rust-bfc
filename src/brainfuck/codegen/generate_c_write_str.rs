use std::io::Write;
use super::super::indent::indent;

pub fn generate_c_write_str(out: &mut Write, data: &[u8], nesting: usize) -> std::io::Result<()> {
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

        write!(out, "\", {}, 1, stdout);\n", data.len())?;
    }

    return Ok(());
}
