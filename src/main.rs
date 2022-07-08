use irongen::{get_app_dirs, get_apps, run_fzf};

fn main() {
    let config: Option<String> = std::env::args().nth(1);
    let config_dir: &str = &format!("{}/irongen", dirs::config_dir().unwrap().to_str().unwrap()); // $HOME/.config
    match config {
        Some(value) => {
            if value == "--config" {
                std::fs::create_dir_all(config_dir).expect("Could not ccreate config directory.");
                std::fs::File::create(&format!("{}/config", config_dir)).expect("Could not create config file.");
            }
            println!("Config file has been created, located at: {}/config", config_dir);
        },
        None => (),
    }
    let dirs = get_app_dirs();
    let apps = get_apps(&dirs);
    let exec = run_fzf(apps);

    // let cmd = exec
    //     .split_inclusive(' ')
    //     .filter(|a| !a.contains('%'))
    //     .collect::<String>();

    print!("{}", exec);
}
