use args::{CurrentCommands, InstallOrUpdate};
use chroot::install_important_packages;
use clap::Parser;
use data_types::{KeyboardDiff, ShellDiff, UserDiff};
use helper::execute_status;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

use crate::args::Cli;
use crate::chroot::install_grub;
use crate::data_types::{
    DirectoriesDiff, DownloadsDiff, Fail2BanDiff, FilesDiff, GrubDiff, LanguageDiff,
    MkinitcpioDiff, MonitorDiff, PackagesDiff, PacmanDiff, ServicesDiff, SystemDiff, TimeDiff,
    UfwDiff,
};
use crate::data_types::{New, Populate};
use crate::function::{Add, Remove};
use crate::structure::CargoToml;
use crate::version::AllVersions;

mod args;
mod chroot;
mod data_types;
mod disk_management;
mod function;
mod get_config;
mod get_diff;
mod get_system;
mod get_system_from_other;
mod helper;
mod structure;
mod version;

const CONFIG_PATH: &str = "/home/falk/config.toml";
const CONFIG_DIR_PATH: &str = "/home/falk/.versions";
const PACMAN_CONF_PATH: &str = "/etc/pacman.conf";
const LOCALE_GEN_PATH: &str = "/etc/locale.gen";
const LOCALE_CONF_PATH: &str = "/etc/locale.conf";
const HOSTNAME_PATH: &str = "/etc/hostname";
const HOSTS_PATH: &str = "/etc/hosts";
const GRUB_PATH: &str = "/etc/default/grub";
const MKINITCPIO_PATH: &str = "/etc/mkinitcpio.conf";
const FAIL2BAN_JAIL_LOCAL_PATH: &str = "/etc/fail2ban/jail.local";
const HYPR_MONITOR_CONF_PATH: &str = "/home/falk/.config/hypr/monitor.conf";

macro_rules! generate_Type_tests {
    ($Type_name:ident, $Var_name:ident, $cargo_name:ident, $element_name:ident) => {
        let mut $Var_name: $Type_name = $Type_name::new();
        $Var_name.populate(&$cargo_name.$element_name);
        dbg!($Var_name.clone().diff);
    };
}

#[allow(unused_variables)]
fn main() {
    let args: Cli = Cli::parse();

    match args.command {
        args::Commands::Version { command } => match command {
            args::VersionCommands::List => {
                let mut version_vec: AllVersions = AllVersions::new();
                version_vec.get_versions();
                version_vec.list_versions();
            }
            args::VersionCommands::Diff { old, new } => {
                let mut version_vec: AllVersions = AllVersions::new();
                version_vec.get_versions();
                version_vec.print_diff(old, new, false);
            }
            args::VersionCommands::Current { command } => match command {
                CurrentCommands::Build => {
                    let cargo_toml: CargoToml = get_cargo_struct(Path::new(CONFIG_PATH));
                    build_current(&cargo_toml);
                }
                CurrentCommands::Commit => {
                    let mut version_vec: AllVersions = AllVersions::new();
                    version_vec.get_versions();
                    version_vec.commit();
                }
                CurrentCommands::ToLatest => {
                    let mut version_vec: AllVersions = AllVersions::new();
                    version_vec.get_versions();
                    let latest_index: usize = version_vec.to_latest();
                    version_vec.rollback(latest_index);
                }
                CurrentCommands::Rollback(i) => {
                    let mut version_vec: AllVersions = AllVersions::new();
                    version_vec.get_versions();
                    version_vec.rollback(i.index);
                }
                CurrentCommands::Diff { other } => {
                    let mut version_vec: AllVersions = AllVersions::new();
                    version_vec.get_versions();
                    version_vec.print_diff(0, other, true);
                }
            },
            args::VersionCommands::Align => {
                let mut version_vec: AllVersions = AllVersions::new();
                version_vec.get_versions();
                version_vec.align_version_indexes();
                version_vec.update_indexes_to_path();
                version_vec.update_paths();
                version_vec.list_versions();
            }
            args::VersionCommands::Delete { command } => match command {
                args::DeleteCommands::Last(i) => {
                    let mut version_vec: AllVersions = AllVersions::new();
                    version_vec.get_versions();
                    version_vec.delete_last_versions(i.number);
                }
                args::DeleteCommands::Version(i) => {
                    let mut version_vec: AllVersions = AllVersions::new();
                    version_vec.get_versions();
                    version_vec.delete_version(i.index);
                }
            },
        },
        args::Commands::Build => {
            let cargo_toml: CargoToml = get_cargo_struct(Path::new(CONFIG_PATH));
            build_current(&cargo_toml);
        }
        args::Commands::Installation { command } => {
            let cargo_toml: CargoToml = get_cargo_struct(Path::new(CONFIG_PATH));
            match command {
                args::InstallationCommands::Part1 {
                    command,
                    setup,
                    partitioning,
                } => {
                    let is_install: bool;
                    match command {
                        InstallOrUpdate::Update => is_install = false,
                        InstallOrUpdate::Install => is_install = true,
                    }
                    if !setup && !partitioning {
                        match is_install {
                            true => {
                                setup_environment_on_installation();
                                generate_Type_tests!(
                                    KeyboardDiff,
                                    keyboard_diff,
                                    cargo_toml,
                                    keyboard
                                );
                                generate_Type_tests!(TimeDiff, time_diff, cargo_toml, time);
                                generate_Type_tests!(PacmanDiff, pacman_diff, cargo_toml, pacman);
                                keyboard_diff.add();
                                time_diff.add();
                                pacman_diff.add();
                                cargo_toml.partitioning.clone().install_or_update(true);
                            }
                            false => {
                                setup_environment_on_installation();
                                generate_Type_tests!(
                                    KeyboardDiff,
                                    keyboard_diff,
                                    cargo_toml,
                                    keyboard
                                );
                                generate_Type_tests!(TimeDiff, time_diff, cargo_toml, time);
                                generate_Type_tests!(PacmanDiff, pacman_diff, cargo_toml, pacman);
                                keyboard_diff.add();
                                time_diff.add();
                                pacman_diff.add();
                                cargo_toml.partitioning.clone().install_or_update(false);
                            }
                        };
                    } else {
                        if setup {
                            setup_environment_on_installation();
                            generate_Type_tests!(KeyboardDiff, keyboard_diff, cargo_toml, keyboard);
                            generate_Type_tests!(TimeDiff, time_diff, cargo_toml, time);
                            generate_Type_tests!(PacmanDiff, pacman_diff, cargo_toml, pacman);
                            keyboard_diff.add();
                            time_diff.add();
                            pacman_diff.add();
                        }
                        match (partitioning, is_install) {
                            (true, true) => cargo_toml.partitioning.clone().install_or_update(true),
                            (true, false) => {
                                cargo_toml.partitioning.clone().install_or_update(false)
                            }
                            _ => (),
                        }
                    }
                }
                args::InstallationCommands::Part2 {
                    //setup,
                    system,
                    shell,
                    user,
                    services,
                    packages,
                    grub,
                    mkinitcpio,
                } => {
                    if !system && !shell && !user && !services && !packages && !grub && !mkinitcpio
                    {
                        setup_environment_on_installation();
                        generate_Type_tests!(KeyboardDiff, keyboard_diff, cargo_toml, keyboard);
                        generate_Type_tests!(TimeDiff, time_diff, cargo_toml, time);
                        generate_Type_tests!(PacmanDiff, pacman_diff, cargo_toml, pacman);
                        generate_Type_tests!(LanguageDiff, language_diff, cargo_toml, language);
                        generate_Type_tests!(SystemDiff, system_diff, cargo_toml, system);
                        generate_Type_tests!(ShellDiff, shell_diff, cargo_toml, shell);
                        generate_Type_tests!(UserDiff, user_diff, cargo_toml, users);
                        generate_Type_tests!(ServicesDiff, services_diff, cargo_toml, services);
                        generate_Type_tests!(GrubDiff, grub_diff, cargo_toml, grub);
                        generate_Type_tests!(
                            MkinitcpioDiff,
                            mkinitcpio_diff,
                            cargo_toml,
                            mkinitcpio
                        );

                        keyboard_diff.add();
                        time_diff.add();
                        language_diff.add();
                        system_diff.add();
                        shell_diff.add();
                        user_diff.remove();
                        user_diff.add();
                        pacman_diff.add();
                        services_diff.remove();
                        services_diff.add();
                        grub_diff.add();
                        mkinitcpio_diff.add();
                    } else {
                        //if setup {
                        //    setup_environment_on_installation();
                        //    generate_Type_tests!(KeyboardDiff, keyboard_diff, cargo_toml, keyboard);
                        //    generate_Type_tests!(TimeDiff, time_diff, cargo_toml, time);
                        //    generate_Type_tests!(PacmanDiff, pacman_diff, cargo_toml, pacman);
                        //    generate_Type_tests!(LanguageDiff, language_diff, cargo_toml, language);
                        //    keyboard_diff.add();
                        //    time_diff.add();
                        //    language_diff.add();
                        //    pacman_diff.add();
                        //}
                        if system {
                            generate_Type_tests!(SystemDiff, system_diff, cargo_toml, system);
                            system_diff.add();
                        };
                        if shell {
                            generate_Type_tests!(ShellDiff, shell_diff, cargo_toml, shell);
                            shell_diff.add();
                        };
                        if user {
                            generate_Type_tests!(UserDiff, user_diff, cargo_toml, users);
                            user_diff.remove();
                            user_diff.add();
                        };
                        if services {
                            generate_Type_tests!(ServicesDiff, services_diff, cargo_toml, services);
                            services_diff.remove();
                            services_diff.add();
                        };
                        if packages {
                            let important_packages: Vec<String> = vec![
                                "grub".to_string(),
                                "efibootmgr".to_string(),
                                "dosfstools".to_string(),
                                "os-prober".to_string(),
                                "networkmanager".to_string(),
                                "lvm2".to_string(),
                            ];
                            install_important_packages(important_packages);
                        };
                        if grub {
                            install_grub(cargo_toml.partitioning);
                            generate_Type_tests!(GrubDiff, grub_diff, cargo_toml, grub);
                            grub_diff.add();
                        };
                        if mkinitcpio {
                            generate_Type_tests!(
                                MkinitcpioDiff,
                                mkinitcpio_diff,
                                cargo_toml,
                                mkinitcpio
                            );
                            mkinitcpio_diff.add();
                        };
                    }
                }
            };
        }
    }
}

fn get_cargo_struct(path: &Path) -> CargoToml {
    let mut file = match File::open(path) {
        Ok(handle) => handle,
        Err(why) => panic!(
            "Error (panic) Unable to open config file: {:?} => {}",
            path, why
        ),
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(why) => panic!("Error (panic): Unable to read the content of the config file: {why}"),
    };

    match toml::from_str(&contents) {
        Ok(out) => out,
        Err(why) => {
            panic!("Error (panic): Unable to retrieve struct from config file content: {why}")
        }
    }
}

fn setup_environment_on_installation() {
    let command: String = String::from("pacman -Sy");
    match execute_status(&command, "/") {
        true => (),
        false => panic!("Error (panic): Failed to execute '{command}'"),
    }
    let command: String = String::from("timedatectl set-ntp true");
    match execute_status(&command, "/") {
        true => (),
        false => panic!("Error (panic): Failed to execute '{command}'"),
    }
}

fn build_current(cargo_toml: &CargoToml) {
    generate_Type_tests!(KeyboardDiff, keyboard_diff, cargo_toml, keyboard);
    generate_Type_tests!(TimeDiff, time_diff, cargo_toml, time);
    generate_Type_tests!(LanguageDiff, language_diff, cargo_toml, language);
    generate_Type_tests!(SystemDiff, system_diff, cargo_toml, system);
    generate_Type_tests!(ShellDiff, shell_diff, cargo_toml, shell);
    generate_Type_tests!(UserDiff, user_diff, cargo_toml, users);
    generate_Type_tests!(PacmanDiff, pacman_diff, cargo_toml, pacman);
    generate_Type_tests!(PackagesDiff, packages_diff, cargo_toml, packages);
    generate_Type_tests!(ServicesDiff, services_diff, cargo_toml, services);
    generate_Type_tests!(DirectoriesDiff, directories_diff, cargo_toml, directories);
    generate_Type_tests!(GrubDiff, grub_diff, cargo_toml, grub);
    generate_Type_tests!(MkinitcpioDiff, mkinitcpio_diff, cargo_toml, mkinitcpio);
    generate_Type_tests!(UfwDiff, ufw_diff, cargo_toml, ufw);
    generate_Type_tests!(Fail2BanDiff, fail2ban_diff, cargo_toml, fail2ban);
    generate_Type_tests!(DownloadsDiff, downloads_diff, cargo_toml, downloads);
    generate_Type_tests!(MonitorDiff, monitor_diff, cargo_toml, monitor);
    generate_Type_tests!(FilesDiff, files_diff, cargo_toml, files);

    keyboard_diff.add();
    time_diff.add();
    language_diff.add();
    system_diff.add();
    shell_diff.add();
    user_diff.add();
    user_diff.remove();
    pacman_diff.add();
    packages_diff.add();
    packages_diff.remove();
    services_diff.add();
    services_diff.remove();
    grub_diff.add();
    mkinitcpio_diff.add();
    ufw_diff.add();
    ufw_diff.remove();
    fail2ban_diff.add();
    fail2ban_diff.remove();
    directories_diff.add();
    downloads_diff.add();
    files_diff.add();
    monitor_diff.add();
}
