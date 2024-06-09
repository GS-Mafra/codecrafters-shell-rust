use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    print!("$ ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(())
}
