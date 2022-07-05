use irongen::{get_app_dirs, get_apps, run_fzf};

fn main() {
    let dirs = get_app_dirs();
    let apps = get_apps(&dirs);
    let cmd = run_fzf(apps);

    print!("{}", cmd);
}
