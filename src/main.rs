// Gitconf by DomesticMoth
//
// To the extent possible under law, the person who associated CC0 with
// gitconf has waived all copyright and related or neighboring rights
// to gitconf.
//
// You should have received a copy of the CC0 legalcode along with this
// work.  If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
mod cfg;
mod profiles;
mod pth;

use inquire::Select;
use log;
use profiles::{get_current_config, get_current_profiles, set_profile};
use simplelog;
use std::env;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use which::which;

fn select_tui(options: Vec<String>) -> Option<String> {
    let opt = vec!["Yes", "No"];
    if Select::new("Setup profile", opt).prompt().unwrap() == "No" {
        return None;
    }
    let choice = Select::new("Selected profile", options).prompt().unwrap();
    Some(choice)
}

fn cmd_show_profiles(_args: Vec<String>) {
    let profiles = match get_current_profiles() {
        Ok(profiles) => profiles,
        Err(e) => {
            log::error!("Cannot get available profiles : {:?}", e);
            return;
        }
    };
    if profiles.len() > 0 {
        log::info!("Available profiles:");
        for (name, path) in profiles.into_iter() {
            println!("\t {} at {:?}", name, path);
        }
    } else {
        log::info!("There is no available profiles");
    }
}

fn cmd_show_profile(_args: Vec<String>) {
    let (config, path) = match get_current_config() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Cannot get current profile : {:?}", e);
            return;
        }
    };
    if let Some(path) = path {
        let buf = PathBuf::from(path.clone());
        let name = buf.file_name().unwrap().to_str().unwrap();
        log::info!("Current profile \"{}\" at {}", name, path);
    } else {
        log::info!("Current profile is default");
    }
    for line in format!("{:#?}", config).lines() {
        println!("\t {}", line);
    }
}

fn cmd_set_profile_path(args: Vec<String>) {
    if args.len() < 3 {
        log::error!("Profile not selected");
        return;
    }
    let path = PathBuf::from(args[2].clone());
    let cur = match std::env::current_dir() {
        Ok(cur) => cur,
        Err(e) => {
            log::error!("Could not set profile {:?}", e);
            return;
        }
    };
    if set_profile(path.clone(), cur) {
        if get_current_config().unwrap().0.apply() {
            log::info!("Profile \"{}\" has been successfully set from {:?}", args[2], path);
        }
    }
}

fn cmd_set_profile(args: Vec<String>) {
    let (config, _) = match get_current_config() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Cannot get current profile : {:?}", e);
            return;
        }
    };
    let profiles = match get_current_profiles() {
        Ok(profiles) => profiles,
        Err(e) => {
            log::error!("Cannot get available profiles : {:?}", e);
            return;
        }
    };
    if profiles.len() < 1 {
        log::error!("There is no available profiles");
        return;
    }
    let mut name = String::from("");
    if args.len() < 3 {
        if !config.interactive {
            log::error!("Profile not selected");
            return;
        }
        if let Some(n) = select_tui(profiles.keys().map(|f| f.clone()).collect()) {
            name = n;
        } else {
            log::error!("Profile not selected");
            return;
        }
    }
    if name.as_str() == "" {
        name = args[2].clone();
    }
    let path = match profiles.get(&name) {
        Some(path) => path.clone(),
        None => {
            log::error!("Could not find a profile with name \"{}\"", args[2]);
            return;
        }
    };
    let cur = match std::env::current_dir() {
        Ok(cur) => cur,
        Err(e) => {
            log::error!("Could not set profile {:?}", e);
            return;
        }
    };
    if set_profile(path.clone(), cur) {
        if get_current_config().unwrap().0.apply() {
            log::info!("Profile \"{}\" has been successfully set from {:?}", name, path);
        }
    }
}

fn run_git_command(args: Vec<String>) {
    // Get path to git executable
    let git = match which("git") {
        Ok(git) => git.into_os_string().into_string().unwrap(),
        Err(e) => {
            log::error!("Cannot find git command : {:?}", e);
            return;
        }
    };
    // Collect cuurent profile configuration
    let (mut config, mut path) = match get_current_config() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Cannot get current profile : {:?}", e);
            return;
        }
    };
    // If SelectProfileOnFirstUse and Interactive mods enable
    if config.interactive && config.select_profile_on_first_use {
        // If current path is git repo
        let mut dir = std::env::current_dir().unwrap();
        dir.push(".git");
        if dir.exists() && dir.is_dir() {
            dir.push(".gitconf");
            // If first use gitconf in this repo
            if !dir.exists() {
                // Grub availablse profiles
                let profiles = match get_current_profiles() {
                    Ok(profiles) => profiles,
                    Err(e) => {
                        log::error!("Cannot get available profiles : {:?}", e);
                        return;
                    }
                };
                // If there is at least one available profile
                if profiles.len() > 0 {
                    // Display dialog with the profile selection
                    if let Some(name) = select_tui(profiles.keys().map(|f| f.clone()).collect()) {
                        dir.pop();
                        dir.pop();
                        let p = profiles.get(&name).unwrap();
                        // Setup selected profile
                        if set_profile(p.clone(), dir) {
                            // Update info about current profile
                            let r = match get_current_config() {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!("Cannot get current profile : {:?}", e);
                                    return;
                                }
                            };
                            config = r.0;
                            path = r.1;
                        } else {
                            log::error!("Cannot setup selected profile");
                            return;
                        }
                    // Else creade viod .git/.gitconf dir
                    } else {
                        if let Err(_) = std::fs::create_dir_all(dir) {} // No matter
                    }
                } else {
                    if let Err(_) = std::fs::create_dir_all(dir) {} // No matter
                }
            }
        }
    }
    // Apply current profile
    if !config.apply() {
        return;
    }
    if config.show_current_profile {
        if let Some(path) = path {
            let buf = PathBuf::from(path.clone());
            let name = buf.file_name().unwrap().to_str().unwrap();
            log::info!("Current profile \"{}\" at {}", name, path);
        } else {
            log::info!("Current profile is default");
        }
    }
    // Exec git command
    let mut command = Command::new(git);
    for arg in args[1..].iter() {
        command.arg(arg);
    }
    let err = command.exec();
    log::error!("Cannot run git command {:?}", err);
}

fn main() {
    // Setup terminal logget
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Debug,
        simplelog::ConfigBuilder::new().set_time_format_str("").build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();
    // Collect environment args
    let args: Vec<String> = env::args().collect();
    // Command selection based on the first env argument
    let cmd = if args.len() > 1 {
        match args[1].as_str() {
            "show-profiles" => cmd_show_profiles,
            "show-profile" => cmd_show_profile,
            "set-profile" => cmd_set_profile,
            "set-profile-path" => cmd_set_profile_path,
            _ => run_git_command,
        }
    } else {
        run_git_command
    };
    // Run selected command
    cmd(args);
}
