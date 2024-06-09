use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim_end();

        match input {
            "exit 0" => break,
            _ => println!("{input}: command not found"),
        }
    }
    Ok(())
}
