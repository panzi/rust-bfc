use std::io::Write;

pub fn generate_asm_str(out: &mut Write, name: &str, data: &[u8]) -> std::io::Result<()> {
    write!(out, "{:-8}db ", format!("{}:", name))?;
    if data.len() > 0 {
        let indent = " ".to_string()
            .repeat(std::cmp::max(name.len() + 1, 8) + 3)
            .into_bytes();
        let mut quote   = false;
        let mut endline = false;
        let mut first   = true;

        for c in data.iter() {
            match *c {
                b'\n' => {
                    if endline {
                        out.write_all(b", \\\n")?;
                        out.write_all(&indent)?;
                    } else if quote {
                        out.write_all(b"\",")?;
                    } else if !first {
                        out.write_all(b",")?;
                    }
                    out.write_all(b"10")?;
                    endline = true;
                    quote = false;
                },
                b' '..=b'&' | b'('..=b'[' | b']'..=b'~' => {
                    if endline {
                        out.write_all(b", \\\n")?;
                        out.write_all(&indent)?;
                        out.write_all(b"\"")?;
                    } else if quote {
                        // continuing quoted string
                    } else if first {
                        out.write_all(b"\"")?;
                    } else {
                        out.write_all(b",\"")?;
                    }

                    out.write_all(&[*c])?;

                    endline = false;
                    quote = true;
                },
                c => {
                    if endline {
                        out.write_all(b", \\\n")?;
                        out.write_all(&indent)?;
                    } else if quote {
                        out.write_all(b"\",")?;
                    } else if !first {
                        out.write_all(b",")?;
                    }

                    write!(out, "{}", c as u32)?;

                    endline = false;
                    quote = false;
                }
            }
            first = false;
        }

        if quote {
            out.write_all(b"\"\n")?;
        } else {
            out.write_all(b"\n")?;
        }
    } else {
        out.write_all(b"\"\"\n")?;
    }

    return Ok(());
}
