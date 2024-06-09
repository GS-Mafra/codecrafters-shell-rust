use std::io::{self, Cursor, Write};

use bytes::Buf;

fn main() -> anyhow::Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let cur = &mut Cursor::new(input);
        match until_space(cur) {
            "exit" => {
                // TODO
                let _code = str_chunk(cur).parse::<u8>()?;
                break;
            }
            "echo" => println!("{}", str_chunk(cur)),
            cmd => println!("{cmd}: command not found"),
        }
    }
    Ok(())
}

fn until_space<'a>(cur: &mut Cursor<&'a str>) -> &'a str {
    let chunk = cur.chunk();
    let start = cur.get_ref().len() - chunk.len();
    let pos = chunk.iter().position(u8::is_ascii_whitespace);

    if let Some(pos) = pos {
        cur.advance(pos + 1);
        &cur.get_ref()[start..start + pos]
    } else {
        cur.advance(chunk.len());
        cur.get_ref()
    }
}

#[inline]
fn str_chunk<'a>(cur: &'a Cursor<&'a str>) -> &'a str {
    let chunk = cur.chunk();
    unsafe { std::str::from_utf8_unchecked(chunk) }
}
