use std::{
    env,
    fs::{self, File},
    io::{BufReader, Read, Write},
    process::{Command, Stdio},
    thread,
};

fn main() {
    let home_dir = env::var("HOME").expect("home not found");
    let xdg_dirs = match env::var("XDG_DATA_DIRS") {
        Ok(v) => v,
        Err(_) => String::from("/usr/local/share:/usr/share"),
    };
    let xdg_home = match env::var("XDG_DATA_HOME") {
        Ok(v) => v,
        Err(_) => format!("{}{}", home_dir, "/.local/share"),
    };
    let joined = format!("{}:{}", xdg_dirs, xdg_home);

    let dirs = joined
        .split(":")
        .map(|y| format!("{}/applications", y.trim_end_matches('/')))
        .collect::<Vec<_>>();

    let all_apps = dirs
        .iter()
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().ends_with(".desktop"))
        .collect::<Vec<_>>();

    let mut apps = all_apps
        .iter()
        .filter_map(|a| {
            let file = File::open(&a).expect("cant open file");
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader
                .read_to_string(&mut contents)
                .expect("problem in reading");

            parse_data(&contents).ok()
        })
        .collect::<Vec<_>>();

    apps.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    apps.dedup_by(|a, b| a.name == b.name);

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
        .expect("failed to execute child");

    let mut stdin = child.stdin.take().expect("failed to get stdin");
    thread::spawn(move || {
        stdin
            .write_all(
                apps.iter()
                    .map(|a| format!("{}\n{}\n{}\0", a.exec, a.name, a.desc))
                    .collect::<String>()
                    .as_bytes(),
            )
            .expect("failed to write to stdin");
    });

    let output = child.wait_with_output().expect("failed to wait on child");

    let stdout_str = String::from_utf8(output.stdout).unwrap();

    let fzf_return_values = stdout_str
        .trim_matches('\0')
        .split('\0')
        .collect::<Vec<_>>();

    let cmd = fzf_return_values[fzf_return_values.len() - 1]
        .lines()
        .nth(0)
        .unwrap();

    print!("{}", cmd);
}

struct App {
    name: String,
    desc: String,
    exec: String,
}

fn parse_data(raw: &str) -> Result<App, &'static str> {
    let mut name = None;
    let mut desc = None;
    let mut exec = None;

    for line in raw.lines() {
        if line.starts_with("Name=") && name == None {
            name = Some(line.trim_start_matches("Name=").to_owned());
        } else if line.starts_with("Comment=") && desc == None {
            desc = Some(line.trim_start_matches("Comment=").to_owned());
        } else if line.starts_with("Exec=") && exec == None {
            exec = Some(line.trim_start_matches("Exec=").to_owned());
        }
    }

    let name = match name {
        Some(n) => n,
        None => return Err("Name is needed"),
    };

    let desc = match desc {
        Some(d) => d,
        None => String::from("No description available"),
    };

    let exec = match exec {
        Some(e) => e,
        None => return Err("Exec is needed"),
    };

    Ok(App { name, desc, exec })
}
