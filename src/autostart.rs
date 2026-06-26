use auto_launch::AutoLaunch;
use std::process::Command;
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
use auto_launch::WindowsEnableMode;

#[cfg(not(target_os = "windows"))]
use auto_launch::LinuxLaunchMode;

static AUTO_LAUNCH: OnceLock<AutoLaunch> = OnceLock::new();

#[cfg(target_os = "windows")]
pub fn auto_launch() -> &'static AutoLaunch {
    AUTO_LAUNCH.get_or_init(|| {
        AutoLaunch::new(
            "somanysweats-loader",
            std::env::current_exe().unwrap().to_str().unwrap(),
            WindowsEnableMode::CurrentUser,
            &[] as &[&str],
        )
    })
}

#[cfg(not(target_os = "windows"))]
pub fn auto_launch() -> &'static AutoLaunch {
    AUTO_LAUNCH.get_or_init(|| {
        AutoLaunch::new(
            "somanysweats-loader",
            std::env::current_exe().unwrap().to_str().unwrap(),
            LinuxLaunchMode::XdgAutostart,
            &[] as &[&str],
        )
    })
}

pub fn relaunch_in_terminal() {
    let exe = std::env::current_exe().unwrap();

    #[cfg(target_os = "windows")]
    let terminals: &[&[&str]] = &[
        &["cmd", "/c", "start", "cmd", "/k"],
        &["powershell", "-NoExit", "-Command", "&"],
    ];

    #[cfg(not(target_os = "windows"))]
    let terminals: &[&[&str]] = &[
        &["ghostty", "-e"],
        &["kitty"],
        &["alacritty", "-e"],
        &["wezterm", "start", "--"],
        &["gnome-terminal", "--"],
        &["konsole", "-e"],
        &["xterm", "-e"],
    ];
    for term in terminals {
        let (cmd, args) = term.split_first().unwrap();
        if Command::new(cmd).args(args).arg(&exe).spawn().is_ok() {
            return;
        }
    }

    eprintln!("Could not find a terminal emulator to launch in");
}
