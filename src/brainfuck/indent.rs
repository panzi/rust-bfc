pub fn indent(out: &mut std::io::Write, nesting: usize) -> std::io::Result<()> {
    for _ in 0..nesting {
        out.write_all(b"    ")?;
    }
    Ok(())
}