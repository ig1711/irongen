use irongen::{get_app_dirs, get_apps, run_fzf};

fn main() {
    let dirs = get_app_dirs();
    let apps = get_apps(&dirs);
    let exec = run_fzf(apps);

    // let cmd = exec
    //     .split_inclusive(' ')
    //     .filter(|a| !a.contains('%'))
    //     .collect::<String>();

    print!("{}", exec);
}
