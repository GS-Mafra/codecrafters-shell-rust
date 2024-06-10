use std::{
    io::{self, Cursor, Write},
    path::{Path, PathBuf},
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

                find_exe(&env_path, cmd).map_or_else(
                    || println!("{cmd} not found"),
                    |path| {
                        println!("{cmd} is {}", path.display());
                    },
                );
            }
            cmd => {
                if let Some(exe_path) = find_exe(&env_path, cmd) {
                    let args = str_chunk(cur).split_whitespace();
                    let output = std::process::Command::new(&exe_path).args(args).output()?;
                    anyhow::ensure!(
                        output.status.success(),
                        "Failed to execute {cmd}: {}",
                        output.status,
                    );
                    io::stdout().write_all(&output.stdout)?;
                    io::stderr().write_all(&output.stderr)?;
                } else {
                    println!("{cmd}: command not found");
                }
            }
        }
    }
    Ok(())
}

fn find_exe<I, P>(dirs: I, exe: impl AsRef<Path>) -> Option<PathBuf>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let exe = exe.as_ref();
    dirs.into_iter().find_map(|path| {
        let path = path.as_ref().join(exe);
        std::fs::metadata(&path)
            .is_ok_and(|x| x.is_file())
            .then_some(path)
    })
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
