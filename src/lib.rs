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

    for line in raw.lines() {
        if name.is_some() && desc.is_some() && exec.is_some() {
            break;
        } else {
            name = name.or_else(|| line.strip_prefix("Name=").map(|f| f.to_owned()));
            desc = desc.or_else(|| line.strip_prefix("Comment=").map(|f| f.to_owned()));
            exec = exec.or_else(|| line.strip_prefix("Exec=").map(|f| f.to_owned()));
        }
    }

    let name = name.ok_or(())?;
    let desc = desc.unwrap_or_else(|| String::from("No description available"));
    let exec = exec.ok_or(())?;

    Ok(App { name, desc, exec })
}

pub fn run_fzf(apps: Vec<App>) -> String {
    let (margin, padding, border, color) = get_config();

    let mut child = Command::new("fzf")
        //      .arg("--color=fg:#636363,hl:#cccccc,hl+:#ff0055,pointer:#ff0055,bg+:-1,query:#00ff6e,prompt:#00ff6e,gutter:-1")
        .arg(color)
        //.arg("--padding=1")
        .arg(margin)
        .arg(padding)
        .arg(border)
        //.arg("--border")
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

    let exec_str_iter = fzf_return_values[fzf_return_values.len() - 1]
        .lines()
        .next()
        .exit_on_err(130, "Cancelled")
        .split_inclusive(' ')
        .filter(|a| !a.contains('%'));

    format!("'{}'", exec_str_iter.collect::<String>().trim())
}

struct Config {
    border: Option<String>,
    padding: Option<String>,
    margin: Option<String>,
    color_fg: Option<String>,
    color_bg: Option<String>,
    color_preview_fg: Option<String>,
    color_preview_bg: Option<String>,
    color_hl: Option<String>,
    color_fg_plus: Option<String>,
    color_bg_plus: Option<String>,
    color_gutter: Option<String>,
    color_hl_plus: Option<String>,
    color_query: Option<String>,
    color_disabled: Option<String>,
    color_info: Option<String>,
    color_border: Option<String>,
    color_prompt: Option<String>,
    color_pointer: Option<String>,
    color_marker: Option<String>,
    color_spinner: Option<String>,
    color_header: Option<String>,
}

impl Config {
    fn new() -> Self {
        Self {
            border: None,
            padding: None,
            margin: None,
            color_fg: None,
            color_bg: None,
            color_preview_fg: None,
            color_preview_bg: None,
            color_hl: None,
            color_fg_plus: None,
            color_bg_plus: None,
            color_gutter: None,
            color_hl_plus: None,
            color_query: None,
            color_disabled: None,
            color_info: None,
            color_border: None,
            color_prompt: None,
            color_pointer: None,
            color_marker: None,
            color_spinner: None,
            color_header: None,
        }
    }
}

fn parse_config(raw: &str) -> Config {
    let mut config = Config::new();

    for line in raw.lines() {
        if line.trim().starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once(':').unwrap_or(("none", "none"));
        match key.trim() {
            "border" => config.border = Some(value.trim().to_owned()),
            "padding" => config.padding = Some(value.trim().to_owned()),
            "margin" => config.margin = Some(value.trim().to_owned()),
            "color_fg" => config.color_fg = Some(value.trim().to_owned()),
            "color_bg" => config.color_bg = Some(value.trim().to_owned()),
            "color_preview_fg" => config.color_preview_fg = Some(value.trim().to_owned()),
            "color_preview_bg" => config.color_preview_bg = Some(value.trim().to_owned()),
            "color_hl" => config.color_hl = Some(value.trim().to_owned()),
            "color_fg_plus" => config.color_fg_plus = Some(value.trim().to_owned()),
            "color_bg_plus" => config.color_bg_plus = Some(value.trim().to_owned()),
            "color_gutter" => config.color_gutter = Some(value.trim().to_owned()),
            "color_hl_plus" => config.color_hl_plus = Some(value.trim().to_owned()),
            "color_query" => config.color_query = Some(value.trim().to_owned()),
            "color_disabled" => config.color_disabled = Some(value.trim().to_owned()),
            "color_info" => config.color_info = Some(value.trim().to_owned()),
            "color_border" => config.color_border = Some(value.trim().to_owned()),
            "color_prompt" => config.color_prompt = Some(value.trim().to_owned()),
            "color_pointer" => config.color_pointer = Some(value.trim().to_owned()),
            "color_marker" => config.color_marker = Some(value.trim().to_owned()),
            "color_spinner" => config.color_spinner = Some(value.trim().to_owned()),
            "color_header" => config.color_header = Some(value.trim().to_owned()),
            _ => (),
        }
    }
    config
}

fn get_config() -> (String, String, String, String) {
    let home_path = env::var("HOME").exit_on_err(1, "Home directory not found");
    let config_home_path =
        env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home_path));
    let irongen_config = config_home_path.trim_end_matches('/').to_owned() + "/irongen/config";

    let file = File::open(irongen_config);
    let mut contents = String::new();

    match file {
        Ok(f) => {
            let mut buf_reader = BufReader::new(f);

            match buf_reader.read_to_string(&mut contents) {
                Ok(_) => (),
                Err(_) => {
                    eprintln!("Could not read the config file, using default config");
                    contents = String::from("");
                }
            }
        }
        Err(_) => {
            eprintln!("Cound not open the config file, using default config");
            contents = String::from("");
        }
    }

    let config = parse_config(&contents);

    let margin = format!(
        "--margin={}",
        config.margin.unwrap_or_else(|| String::from("10,20"))
    );

    let padding = format!(
        "--padding={}",
        config.padding.unwrap_or_else(|| String::from("1"))
    );

    let border = format!(
        "--border={}",
        config.border.unwrap_or_else(|| String::from("rounded"))
    );
    let color_fg = config
        .color_fg
        .map(|x| format!("fg:{},", x))
        .unwrap_or_else(|| String::from("fg:#636363,"));
    let color_bg = config
        .color_bg
        .map(|x| format!("bg:{},", x))
        .unwrap_or_else(|| String::from(""));
    let color_preview_fg = config
        .color_preview_fg
        .map(|x| format!("preview-fg:{},", x))
        .unwrap_or_else(|| String::from(""));
    let color_preview_bg = config
        .color_preview_bg
        .map(|x| format!("preview-bg:{},", x))
        .unwrap_or_else(|| String::from(""));
    let color_hl = config
        .color_hl
        .map(|x| format!("hl:{},", x))
        .unwrap_or_else(|| String::from("hl:#cccccc,"));
    let color_fg_plus = config
        .color_fg_plus
        .map(|x| format!("fg+:{},", x))
        .unwrap_or_else(|| String::from(""));
    let color_bg_plus = config
        .color_bg_plus
        .map(|x| format!("bg+:{},", x))
        .unwrap_or_else(|| String::from("bg+:-1,"));
    let color_gutter = config
        .color_gutter
        .map(|x| format!("gutter:{},", x))
        .unwrap_or_else(|| String::from("gutter:-1,"));
    let color_hl_plus = config
        .color_hl_plus
        .map(|x| format!("hl+:{},", x))
        .unwrap_or_else(|| String::from("hl+:#ff0055,"));
    let color_query = config
        .color_query
        .map(|x| format!("query:{},", x))
        .unwrap_or_else(|| String::from("query:#00ff6e,"));
    let color_disabled = config
        .color_disabled
        .map(|x| format!("disabled:{},", x))
        .unwrap_or_else(|| String::from(""));
    let color_info = config
        .color_info
        .map(|x| format!("info:{},", x))
        .unwrap_or_else(|| String::from(""));
    let color_border = config
        .color_border
        .map(|x| format!("border:{},", x))
        .unwrap_or_else(|| String::from(""));
    let color_prompt = config
        .color_prompt
        .map(|x| format!("prompt:{},", x))
        .unwrap_or_else(|| String::from("prompt:#00ff6e,"));
    let color_pointer = config
        .color_pointer
        .map(|x| format!("pointer:{},", x))
        .unwrap_or_else(|| String::from("pointer:#ff0055,"));
    let color_marker = config
        .color_marker
        .map(|x| format!("marker:{},", x))
        .unwrap_or_else(|| String::from(""));
    let color_spinner = config
        .color_spinner
        .map(|x| format!("spinner:{},", x))
        .unwrap_or_else(|| String::from(""));
    let color_header = config
        .color_header
        .map(|x| format!("header:{},", x))
        .unwrap_or_else(|| String::from(""));

    let color = format!(
        "--color={}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
        color_fg,
        color_bg,
        color_fg_plus,
        color_bg_plus,
        color_preview_fg,
        color_preview_bg,
        color_hl,
        color_hl_plus,
        color_gutter,
        color_query,
        color_disabled,
        color_info,
        color_border,
        color_prompt,
        color_pointer,
        color_marker,
        color_spinner,
        color_header
    )
    .trim_end_matches(',')
    .to_owned();

    (margin, padding, border, color)
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
