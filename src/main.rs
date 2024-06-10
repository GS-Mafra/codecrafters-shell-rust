use std::{
    io::{self, Cursor, Write},
    path::PathBuf,
};

use bytes::Buf;
use strum::VariantNames;

#[allow(dead_code)] // TODO
#[derive(Debug, VariantNames)]
enum BuiltIn<'a> {
    Exit(u8),
    Echo(&'a str),
    Type(&'a str),
}

impl<'a> BuiltIn<'a> {
    #[inline]
    fn is_builtin(cmd: &str) -> bool {
        Self::VARIANTS.iter().any(|x| x.eq_ignore_ascii_case(cmd))
    }
}

fn main() -> anyhow::Result<()> {
    let env_path = std::env::var("PATH").map_or_else(
        |_| Vec::new(),
        |x| x.split(':').map(PathBuf::from).collect::<Vec<_>>(),
    );

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
                // BuiltIn::Exit(code);
                break;
            }
            "echo" => {
                // BuiltIn::Echo(str_chunk(cur));
                println!("{}", str_chunk(cur));
            }
            "type" => 'ty: {
                let cmd = str_chunk(cur);

                if BuiltIn::is_builtin(cmd) {
                    println!("{cmd} is a shell builtin");
                    break 'ty;
                }

                env_path
                    .iter()
                    .find_map(|path| {
                        let path = path.join(cmd);
                        std::fs::metadata(&path)
                            .is_ok_and(|x| x.is_file())
                            .then_some(path)
                    })
                    .map_or_else(
                        || println!("{cmd} not found"),
                        |path| {
                            println!("{cmd} is {}", path.display());
                        },
                    );
            }
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
