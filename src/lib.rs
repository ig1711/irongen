use std::{
    env,
    fs::{self, File},
    io::{BufReader, Read, Write},
    process::{self, Command, Stdio},
    thread,
};

pub fn get_app_dirs() -> Vec<String> {
    let home_dir = env::var("HOME").exit_on_err(1, "home not found");
    let xdg_dirs =
        env::var("XDG_DATA_DIRS").unwrap_or_else(|_| String::from("/usr/local/share:/usr/share"));
    let xdg_home =
        env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}{}", home_dir, "/.local/share"));
    let joined = format!("{}:{}", xdg_dirs, xdg_home);

    joined
        .split(':')
        .map(|y| format!("{}/applications", y.trim_end_matches('/')))
        .collect::<Vec<_>>()
}

pub struct App {
    name: String,
    desc: String,
    exec: String,
}

pub fn get_apps(dirs: &[String]) -> Vec<App> {
    let all_apps = dirs
        .iter()
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .filter_map(|e| {
            e.ok().and_then(|f| {
                f.path()
                    .to_string_lossy()
                    .ends_with(".desktop")
                    .then(|| f.path())
            })
        })
        .collect::<Vec<_>>();

    let mut apps = all_apps
        .iter()
        .filter_map(|a| {
            let file = File::open(&a).ok()?;
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents).ok()?;

            parse_data(&contents).ok()
        })
        .collect::<Vec<_>>();

    apps.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    apps.dedup_by(|a, b| a.name == b.name);
    apps
}

fn parse_data(raw: &str) -> Result<App, ()> {
    let mut name = None;
    let mut desc = None;
    let mut exec = None;

    let x = &['a', 'v'];

    for line in raw.lines() {
        if name.is_some() && desc.is_some() && exec.is_some() {
            break;
        } else {
            name = name.or_else(|| line.strip_prefix("Name=").map(|f| f.to_owned()));
            desc = desc.or_else(|| line.strip_prefix("Comment=").map(|f| f.to_owned()));
            exec = exec.or_else(|| {
                line.strip_prefix("Exec=")
                    .map(|f| f.trim_end_matches(x).to_owned())
            });
        }
    }

    let name = name.ok_or(())?;
    let desc = desc.unwrap_or_else(|| String::from("No description available"));
    let exec = exec.ok_or(())?;

    Ok(App { name, desc, exec })
}

pub fn run_fzf(apps: Vec<App>) -> String {
    let mut child = Command::new("fzf")
      .arg("--color=fg:#636363,hl:#cccccc,hl+:#ff0055,pointer:#ff0055,bg+:-1,query:#00ff6e,prompt:#00ff6e,gutter:-1")
      .arg("--padding=1")
      .arg("--margin=10,20")
      .arg("--border")
      .arg("--no-info")
      .arg("--no-bold")
      .arg("--print-query")
      .arg("--bind=ctrl-space:print-query,tab:replace-query")
      .arg("--with-nth=2")
      .arg("--delimiter=\n")
      .arg("--preview=echo {2..3}")
      .arg("--preview-window=border-left")
      .arg("--read0")
      .arg("--print0")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()
      .exit_on_err(4, "fzf could not be executed");

    let mut stdin = child.stdin.take().exit_on_err(8, "failed to get stdin");
    thread::spawn(move || {
        stdin
            .write_all(
                apps.iter()
                    .map(|a| format!("{}\n{}\n{}\0", a.exec, a.name, a.desc))
                    .collect::<String>()
                    .as_bytes(),
            )
            .exit_on_err(16, "failed to write to stdin");
    });

    let output = child
        .wait_with_output()
        .exit_on_err(32, "failed to wait on child");

    let stdout_str =
        String::from_utf8(output.stdout).exit_on_err(64, "fzf output isnt utf-8 encoded string");

    let fzf_return_values = stdout_str
        .trim_matches('\0')
        .split('\0')
        .collect::<Vec<_>>();

    fzf_return_values[fzf_return_values.len() - 1]
        .lines()
        .next()
        .exit_on_err(130, "Cancelled")
        .to_owned()
}

trait ExitOnErr<T> {
    fn exit_on_err(self, code: i32, message: &'static str) -> T;
}

impl<T, E> ExitOnErr<T> for Result<T, E> {
    fn exit_on_err(self, code: i32, message: &'static str) -> T {
        match self {
            Ok(value) => value,
            Err(_) => {
                eprintln!("{}", message);
                process::exit(code);
            }
        }
    }
}

impl<T> ExitOnErr<T> for Option<T> {
    fn exit_on_err(self, code: i32, message: &'static str) -> T {
        match self {
            Some(value) => value,
            None => {
                eprintln!("{}", message);
                process::exit(code);
            }
        }
    }
}
