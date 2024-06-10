#![feature(absolute_path)]

use std::{
    env,
    io::{self, Cursor, ErrorKind, Write},
    path::{self, Path, PathBuf},
    process::Command,
};

use bytes::Buf;
use strum::VariantNames;

#[allow(dead_code)] // TODO
#[derive(Debug, VariantNames)]
enum BuiltIn<'a> {
    Exit(u8),
    Echo(&'a str),
    Type(&'a str),
    Pwd,
    Cd(&'a Path),
}

impl<'a> BuiltIn<'a> {
    #[inline]
    fn is_builtin(cmd: &str) -> bool {
        Self::VARIANTS.iter().any(|x| x.eq_ignore_ascii_case(cmd))
    }
}

fn main() -> anyhow::Result<()> {
    let env_path = env::var("PATH")
        .map(|x| env::split_paths(&x).collect::<Vec<_>>())
        .unwrap_or_default();

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
            "pwd" => println!("{}", env::current_dir()?.display()),
            "cd" => {
                let chunk = str_chunk(cur);
                let path = if chunk == "~" || chunk.is_empty() {
                    dirs::home_dir().ok_or_else(|| io::Error::from(ErrorKind::NotFound))
                } else {
                    path::absolute(chunk)
                };

                if let Err(e) = path.and_then(env::set_current_dir) {
                    match e.kind() {
                        io::ErrorKind::NotFound => {
                            println!("cd: {chunk}: No such file or directory");
                        }
                        _ => return Err(e.into()),
                    }
                }
            }
            cmd => {
                if let Some(exe_path) = find_exe(&env_path, cmd) {
                    let args = str_chunk(cur).split_whitespace();
                    let output = Command::new(&exe_path).args(args).output()?;
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
