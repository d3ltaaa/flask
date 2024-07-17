use core::panic;
use std::{fs, path::Path};

use crate::{
    data_types::{
        DirectoriesDiff, DownloadsDiff, Fail2BanDiff, FilesDiff, GrubDiff, KeyboardDiff,
        LanguageDiff, MkinitcpioDiff, MonitorDiff, PackagesDiff, PacmanDiff, ServicesDiff,
        ShellDiff, SystemDiff, TimeDiff, UfwDiff, UserDiff,
    },
    helper::{
        self, append_to_file, execute_output, execute_status, is_user_root, prepend_to_file,
        printmsg, remove_from_file, replace_line, write_to_file,
    },
    FAIL2BAN_JAIL_LOCAL_PATH, GRUB_PATH, HOSTNAME_PATH, HOSTS_PATH, HYPR_MONITOR_CONF_PATH,
    LOCALE_CONF_PATH, LOCALE_GEN_PATH, MKINITCPIO_PATH, PACMAN_CONF_PATH,
};

pub trait Add {
    fn add(&self) -> bool;
}

pub trait Remove {
    fn remove(&self) -> bool;
}

impl Add for KeyboardDiff {
    fn add(&self) -> bool {
        match self.diff.add.mkinitcpio.clone() {
            Some(ref keyboard) => {
                printmsg("Adding", "Keyboard", &keyboard);
                let replace_string: String = format!("KEYMAP={}", keyboard);
                replace_line(
                    Path::new(MKINITCPIO_PATH),
                    "KEYMAP=",
                    replace_string.as_str(),
                )
            }
            None => true,
        }
    }
}

impl Add for TimeDiff {
    fn add(&self) -> bool {
        match self.diff.add.timezone.clone() {
            Some(ref timezone) => {
                printmsg("Adding", "timezone", &timezone);
                let argument: String = format!("timedatectl set-timezone {}", timezone);
                execute_status(&argument, "/")
            }
            None => true,
        }
    }
}

impl Add for LanguageDiff {
    fn add(&self) -> bool {
        match (
            self.diff.add.character.clone(),
            self.diff.add.locale.clone(),
        ) {
            (None, None) => true,
            (_, _) => {
                let character: String = self.config.character.clone().unwrap();
                let locale: String = self.config.locale.clone().unwrap();
                let msg_string: String = format!("{character} + {locale}");
                printmsg("Adding", "Character + Locale", msg_string);
                let write: String = format!("LANG={}\n", locale);
                let add_character: bool =
                    helper::write_to_file(Path::new(LOCALE_CONF_PATH), &write);
                let write: String = format!("{} {}\n", locale, character);
                let add_locale: bool = helper::write_to_file(Path::new(LOCALE_GEN_PATH), &write);
                let generate_locale: bool = execute_status("locale-gen", "/");
                add_character && add_locale && generate_locale
            }
        }
    }
}

impl Add for SystemDiff {
    fn add(&self) -> bool {
        match self.diff.add.hostname {
            Some(ref hostname) => {
                printmsg("Adding", "Hostname", &hostname);
                let add_hostname: bool = helper::write_to_file(Path::new(HOSTNAME_PATH), &hostname);
                let write: String = format!(
                    "127.0.0.1       localhost\n::1             localhost\n127.0.1.1       {}.localdomain {}",
                    &hostname, &hostname
                );
                let add_hosts: bool = helper::write_to_file(Path::new(HOSTS_PATH), &write);
                add_hostname && add_hosts
            }
            None => true,
        }
    }
}

impl Add for ShellDiff {
    fn add(&self) -> bool {
        match self.diff.add.default_shell {
            Some(ref shell) => {
                printmsg("Adding", "Default Shell", &shell);
                if execute_status(&format!("chsh -s {}", shell), "/") {
                    println!("The new shell will be available after restart!");
                    true
                } else {
                    false
                }
            }
            None => true,
        }
    }
}

impl Add for UserDiff {
    fn add(&self) -> bool {
        let add_user: bool = match self.diff.add.user_list {
            Some(ref user_list) => {
                printmsg("Adding", "Userlist", &user_list);
                let mut result: bool = true;
                for user in user_list {
                    let arg_create_user: String = format!("useradd {}", user);
                    result = result && execute_status(&arg_create_user, "/");
                }
                result
            }
            None => true,
        };
        let add_groups: bool = match self.diff.add.user_groups {
            Some(ref user_groups) => {
                printmsg("Adding", "Usergroups", &user_groups);
                let mut result: bool = true;
                for user_group_struct in user_groups {
                    let mut arg_add_groups_to_user: String = String::from("usermod -aG ");
                    for group in user_group_struct.groups.clone() {
                        arg_add_groups_to_user.push_str(&group);
                        arg_add_groups_to_user.push(',');
                        let arg_check_for_group: String = format!("getent group {}", group);
                        let out_check_for_group: String =
                            match execute_output(&arg_check_for_group, "/") {
                                Ok(output) => String::from_utf8(output.stdout).expect(
                                    "Error (Expect): Conversion from utf8 to String failed",
                                ),
                                Err(_) => panic!(
                                    "Error (Panic): Failed to execute arg_check_for_group: {}",
                                    arg_check_for_group
                                ),
                            };
                        if out_check_for_group == "" {
                            let arg_create_group: String = format!("groupadd {}", group);
                            result = result && execute_status(&arg_create_group, "/");
                        }
                    }
                    arg_add_groups_to_user.pop();
                    arg_add_groups_to_user.push(' ');
                    arg_add_groups_to_user.push_str(&user_group_struct.name);
                    result = result && execute_status(&arg_add_groups_to_user, "/");
                }
                result
            }
            None => true,
        };
        add_user && add_groups
    }
}

impl Remove for UserDiff {
    fn remove(&self) -> bool {
        let remove_user_group: bool = match self.diff.remove.user_groups {
            Some(ref user_groups) => {
                printmsg("Removing", "Usergroups", &user_groups);
                let mut result: bool = true;
                for user_group_struct in user_groups {
                    for group in user_group_struct.groups.clone() {
                        let arg_delete_group_from_user: String =
                            format!("gpasswd -d {} {}", user_group_struct.name, group);
                        result = result && execute_status(&arg_delete_group_from_user, "/");
                    }
                }
                result
            }
            None => true,
        };

        let remove_user: bool = match self.diff.remove.user_list {
            Some(ref user_list) => {
                printmsg("Removing", "Users", &user_list);
                let mut result: bool = true;
                for user in user_list {
                    let arg_delete_user: String = format!("userdel {}", user);
                    result = result && execute_status(&arg_delete_user, "/");
                }
                result
            }
            None => true,
        };
        remove_user && remove_user_group
    }
}

impl Add for PacmanDiff {
    fn add(&self) -> bool {
        match self.diff.add.parallel {
            Some(ref parallel) => {
                printmsg("Adding", "Parralel", &parallel);
                let replace_str: String = format!("ParallelDownloads = {}\n", parallel);
                replace_line(
                    Path::new(PACMAN_CONF_PATH),
                    "ParallelDownloads",
                    &replace_str,
                )
            }
            None => true,
        }
    }
}

impl Add for PackagesDiff {
    fn add(&self) -> bool {
        let add_pacman: bool = match self.diff.add.pacman_packages {
            Some(ref pacman_packages) => {
                printmsg("Adding", "Pacman-Packages", &pacman_packages);
                let mut arg_download_pacman: String = String::from("pacman --noconfirm -S ");
                for package in pacman_packages {
                    arg_download_pacman.push_str(&package);
                    arg_download_pacman.push(' ');
                }
                execute_status(&arg_download_pacman, "/")
            }
            None => true,
        };
        let add_aur: bool = match self.diff.add.aur_packages {
            Some(ref aur_packages) => {
                printmsg("Adding", "Aur-Packages", &aur_packages);
                let mut arg_download_aur: String = String::from("paru --noconfirm -S ");
                for package in aur_packages {
                    arg_download_aur.push_str(&package);
                    arg_download_aur.push(' ');
                }
                execute_status(&arg_download_aur, "/")
            }
            None => true,
        };
        let add_manual: bool = match self.diff.add.manual_install_packages {
            Some(ref manual_packages) => {
                printmsg("Adding", "Manual-Packages", &manual_packages);
                let mut result: bool = true;
                for package in manual_packages {
                    if package.root && is_user_root() {
                        result = result && execute_status(&package.command, "/");
                    } else if !package.root && !is_user_root() {
                        result = result && execute_status(&package.command, "/");
                    }
                }
                result
            }
            None => true,
        };
        add_pacman && add_aur && add_manual
    }
}

impl Remove for PackagesDiff {
    fn remove(&self) -> bool {
        let remove_pacman: bool = match self.diff.remove.pacman_packages {
            Some(ref pacman_packages) => {
                printmsg("Removing", "Pacman-Packages", &pacman_packages);
                let mut arg_remove_pacman: String = String::from("pacman --noconfirm -Rns ");
                for package in pacman_packages {
                    arg_remove_pacman.push_str(&package);
                    arg_remove_pacman.push(' ');
                }
                execute_status(&arg_remove_pacman, "/")
            }
            None => true,
        };
        let remove_aur: bool = match self.diff.remove.aur_packages {
            Some(ref aur_packages) => {
                printmsg("Removing", "Aur-Packages", &aur_packages);
                let mut arg_remove_aur: String = String::from("paru --noconfirm -Rns ");
                for package in aur_packages {
                    arg_remove_aur.push_str(&package);
                    arg_remove_aur.push(' ');
                }
                execute_status(&arg_remove_aur, "/")
            }
            None => true,
        };
        remove_pacman && remove_aur
    }
}

impl Add for ServicesDiff {
    fn add(&self) -> bool {
        let add_services: bool = match self.diff.add.services {
            Some(ref services) => {
                printmsg("Adding", "Services", &services);
                let mut result: bool = true;
                for service in services {
                    let arg_enable_service: String = format!("systemctl enable {}", service);
                    result = result && execute_status(&arg_enable_service, "/");
                }
                result
            }
            None => true,
        };
        let add_user_services: bool = match self.diff.add.user_services {
            Some(ref user_services) => {
                printmsg("Adding", "User-Services", &user_services);
                let mut result: bool = true;
                for service in user_services {
                    let arg_enable_user_service: String =
                        format!("systemctl --user enable {}", service);
                    result = result && execute_status(&arg_enable_user_service, "/");
                }
                result
            }
            None => true,
        };

        add_services && add_user_services
    }
}

impl Remove for ServicesDiff {
    fn remove(&self) -> bool {
        let disable_services: bool = match self.diff.remove.services {
            Some(ref services) => {
                printmsg("Removing", "Services", &services);
                let mut result: bool = true;
                for service in services {
                    let arg_disable_service: String = format!("systemctl disable {}", service);
                    result = result && execute_status(&arg_disable_service, "/");
                }
                result
            }
            None => true,
        };
        let disable_user_services: bool = match self.diff.remove.user_services {
            Some(ref user_services) => {
                printmsg("Removing", "User-Services", &user_services);
                let mut result: bool = true;
                for service in user_services {
                    let arg_disable_user_service: String =
                        format!("systemctl --user disable {}", service);
                    result = result && execute_status(&arg_disable_user_service, "/");
                }
                result
            }
            None => true,
        };

        disable_services && disable_user_services
    }
}

impl Add for DirectoriesDiff {
    fn add(&self) -> bool {
        let mut add_reown_dirs: bool = true;
        if is_user_root() {
            add_reown_dirs = match self.diff.add.reown_dirs {
                Some(ref reown_dirs) => {
                    printmsg("Adding", "Reown-Dirs", &reown_dirs);
                    let mut result: bool = true;
                    for dir in reown_dirs {
                        if !Path::new(&dir.directory).exists() {
                            let arg_create_dir: String = format!("mkdir -p {}", dir.directory);
                            result = result && execute_status(&arg_create_dir, "/");
                        }
                        let arg_reown_dir: String =
                            format!("chown :{} {}", dir.group, dir.directory);
                        result = result && execute_status(&arg_reown_dir, "/");
                    }
                    result
                }
                None => true,
            };
        }
        let add_create_dirs: bool = match self.diff.add.create_dirs {
            Some(ref create_dirs) => {
                printmsg("Adding", "Create-Dirs", &create_dirs);
                let mut result: bool = true;
                for dir in create_dirs {
                    let arg_create_dir: String = format!("mkdir -p {}", dir.path);
                    if dir.root && is_user_root() {
                        result = result && execute_status(&arg_create_dir, "/");
                    } else if !dir.root && !is_user_root() {
                        result = result && execute_status(&arg_create_dir, "/");
                    }
                }
                result
            }
            None => true,
        };
        let add_links: bool = match self.diff.add.links {
            Some(ref links) => {
                printmsg("Adding", "Links", &links);
                let mut result: bool = true;
                for link in links {
                    if !Path::new(&link.origin).is_dir() {
                        println!("Origin ({}) does not exist. Skipping...", link.origin);
                    } else {
                        let arg_get_origin_content: String = format!("ls -A {}", link.origin);
                        let out_get_origin_content: String =
                            match execute_output(&arg_get_origin_content, "/") {
                                Ok(output) => String::from_utf8(output.stdout).expect(
                                    "Error (Expect): Conversion from utf8 to String failed",
                                ),
                                Err(_) => panic!(
                                    "Error (Panic): Failed to execute arg_get_origin_content: {}",
                                    arg_get_origin_content
                                ),
                            };

                        for line in out_get_origin_content.lines() {
                            let arg_create_link: String =
                                format!("ln -sf {}/{} {}", link.origin, line, link.destination);
                            let arg_create_dir: String = format!("mkdir -p {}", link.destination);
                            // check if destination dir has to be created
                            let mut create_destination_dir: bool = false;
                            if !Path::new(&link.destination).is_dir() {
                                create_destination_dir = true;
                            }
                            if !Path::new(&link.destination).is_symlink() {
                                if link.root && is_user_root() {
                                    if create_destination_dir {
                                        result = result && execute_status(&arg_create_dir, "/");
                                    }
                                    result = result && execute_status(&arg_create_link, "/");
                                } else if !link.root && !is_user_root() {
                                    if create_destination_dir {
                                        result = result && execute_status(&arg_create_dir, "/");
                                    }
                                    result = result && execute_status(&arg_create_link, "/");
                                }
                            }
                        }
                    }
                }
                result
            }
            None => true,
        };
        add_reown_dirs && add_create_dirs && add_links
    }
}

impl Add for GrubDiff {
    fn add(&self) -> bool {
        match (
            self.diff.add.grub_cmdline_linux_default.clone(),
            self.diff.remove.grub_cmdline_linux_default.clone(),
        ) {
            (None, None) => true,
            (_, _) => {
                let config_grub_args: Option<Vec<String>> =
                    self.config.grub_cmdline_linux_default.clone();
                printmsg("Adding", "GRUB_CMDLINE_LINUX_DEFAULT", &config_grub_args);
                let mut repl_str: String = String::from("GRUB_CMDLINE_LINUX_DEFAULT=\"");
                match config_grub_args {
                    None => (),
                    Some(args) => {
                        for arg in args.clone() {
                            repl_str.push_str(&arg);
                            repl_str.push(' ');
                        }
                        if args.len() > 0 {
                            repl_str.pop();
                        }
                    }
                }
                repl_str.push('\"');
                replace_line(
                    Path::new(GRUB_PATH),
                    "GRUB_CMDLINE_LINUX_DEFAULT=",
                    &repl_str,
                )
            }
        }
    }
}

impl Add for MkinitcpioDiff {
    fn add(&self) -> bool {
        let add_hooks: bool = match (self.diff.add.hooks.clone(), self.diff.remove.hooks.clone()) {
            (None, None) => true,
            (_, _) => match self.config.hooks.clone() {
                None => true,
                Some(hooks) => {
                    printmsg("Adding", "Mkinitcpio-Hooks", &hooks);
                    let mut repl_str: String = String::from("HOOKS=(");
                    for hook in hooks.clone() {
                        repl_str.push_str(&hook);
                        repl_str.push(' ');
                    }
                    if hooks.len() > 0 {
                        repl_str.pop();
                    }
                    repl_str.push(')');
                    replace_line(Path::new(MKINITCPIO_PATH), "HOOKS=(", &repl_str)
                }
            },
        };

        let add_modules: bool = match (
            self.diff.add.modules.clone(),
            self.diff.remove.modules.clone(),
        ) {
            (None, None) => true,
            (_, _) => match self.config.modules.clone() {
                None => true,
                Some(modules) => {
                    printmsg("Adding", "Mkinitcpio-Modules", &modules);
                    let mut repl_str: String = String::from("MODULES=(");
                    for module in modules.clone() {
                        repl_str.push_str(&module);
                        repl_str.push(' ');
                    }
                    if modules.len() > 0 {
                        repl_str.pop();
                    }
                    repl_str.push(')');
                    replace_line(Path::new(MKINITCPIO_PATH), "MODULES=(", &repl_str)
                }
            },
        };
        add_hooks && add_modules
    }
}

impl Add for UfwDiff {
    fn add(&self) -> bool {
        let add_incoming: bool = match self.diff.add.incoming {
            Some(ref incoming) => {
                printmsg("Adding", "Ufw-Incoming", &incoming);
                let arg_set_incoming: String = format!("ufw default {} incoming", incoming);
                execute_status(&arg_set_incoming, "/")
            }
            None => true,
        };
        let add_outgoing: bool = match self.diff.add.outgoing {
            Some(ref outgoing) => {
                printmsg("Adding", "Ufw-Outgoing", &outgoing);
                let arg_set_outgoing: String = format!("ufw default {} outgoing", outgoing);
                execute_status(&arg_set_outgoing, "/")
            }
            None => true,
        };
        let add_rule: bool = match self.diff.add.rules {
            Some(ref rules) => {
                printmsg("Adding", "Ufw-Rules", &rules);
                let mut result: bool = true;
                for rule in rules {
                    let arg_add_rule: String = format!("ufw allow {}", rule);
                    result = result && execute_status(&arg_add_rule, "/");
                }
                result
            }
            None => true,
        };

        add_incoming && add_outgoing && add_rule
    }
}

impl Remove for UfwDiff {
    fn remove(&self) -> bool {
        match self.diff.remove.rules {
            Some(ref rules) => {
                printmsg("Removing", "Ufw-Rules", &rules);
                let mut result: bool = true;
                for rule in rules {
                    let arg_delete_rule: String = format!("ufw delete allow {}", rule);
                    result = result && execute_status(&arg_delete_rule, "/");
                }
                result
            }
            None => true,
        }
    }
}

impl Add for Fail2BanDiff {
    fn add(&self) -> bool {
        let add_ignoreip: bool = match self.diff.add.ignoreip {
            Some(ref ignoreip) => {
                printmsg("Adding", "F2B-Ignoreip", &ignoreip);
                let repl_ignoreip_str: String = format!("ignoreip = {}", ignoreip);
                replace_line(
                    Path::new(FAIL2BAN_JAIL_LOCAL_PATH),
                    "ignoreip =",
                    &repl_ignoreip_str,
                )
            }
            None => true,
        };
        let add_bantime: bool = match self.diff.add.bantime {
            Some(ref bantime) => {
                printmsg("Adding", "F2B-Bantime", &bantime);
                let repl_bantime_str: String = format!("bantime = {}", bantime);
                replace_line(
                    Path::new(FAIL2BAN_JAIL_LOCAL_PATH),
                    "bantime =",
                    &repl_bantime_str,
                )
            }
            None => true,
        };
        let add_findtime: bool = match self.diff.add.findtime {
            Some(ref findtime) => {
                printmsg("Adding", "F2B-Findtime", &findtime);
                let repl_findtime_str: String = format!("findtime = {}", findtime);
                replace_line(
                    Path::new(FAIL2BAN_JAIL_LOCAL_PATH),
                    "findtime =",
                    &repl_findtime_str,
                )
            }
            None => true,
        };
        let add_maxretry: bool = match self.diff.add.maxretry {
            Some(ref maxretry) => {
                printmsg("Adding", "F2B-Maxretry", &maxretry);
                let repl_maxretry_str: String = format!("maxretry = {}", maxretry);
                replace_line(
                    Path::new(FAIL2BAN_JAIL_LOCAL_PATH),
                    "bantim =",
                    &repl_maxretry_str,
                )
            }
            None => true,
        };
        let add_services: bool = match self.diff.add.services {
            Some(ref services) => {
                printmsg("Adding", "F2B-Services", &services);
                let mut result: bool = true;
                for service in services {
                    let append_service_str: String = format!("\n[{}]\nenabled = true", service);
                    result = result
                        && append_to_file(Path::new(FAIL2BAN_JAIL_LOCAL_PATH), &append_service_str);
                }
                result
            }
            None => true,
        };
        add_ignoreip && add_bantime && add_findtime && add_maxretry && add_services
    }
}

impl Remove for Fail2BanDiff {
    fn remove(&self) -> bool {
        match self.diff.remove.services {
            Some(ref services) => {
                printmsg("Removing", "F2B-Services", &services);
                let mut result: bool = true;
                for service in services {
                    let del_service_str: String = format!("\n\n[{}]\nenabled = true", service);
                    result = result
                        && remove_from_file(Path::new(FAIL2BAN_JAIL_LOCAL_PATH), &del_service_str);
                }
                result
            }
            None => true,
        }
    }
}

impl Add for DownloadsDiff {
    fn add(&self) -> bool {
        let add_curl: bool = match self.diff.add.curl {
            Some(ref curls) => {
                printmsg("Adding", "Curls", &curls);
                let mut result: bool = true;
                for curl in curls {
                    let arg_create_dir: String = format!("mkdir -p {}", curl.path);
                    let arg_curl: String =
                        format!("curl -L {} > {}/{}", curl.url, curl.path, curl.file_name);
                    let mut create_dir: bool = false;
                    if !Path::new(&curl.path).is_dir() {
                        create_dir = true;
                    }
                    if curl.root && is_user_root() {
                        if create_dir {
                            result = result && execute_status(&arg_create_dir, "/");
                        }
                        result = result && execute_status(&arg_curl, "/");
                    } else if !curl.root && !is_user_root() {
                        if create_dir {
                            result = result && execute_status(&arg_create_dir, "/");
                        }
                        result = result && execute_status(&arg_curl, "/");
                    }
                }
                result
            }
            None => true,
        };
        let add_git: bool = match self.diff.add.git {
            Some(ref gits) => {
                printmsg("Adding", "Gits", &gits);
                let mut result: bool = true;
                for git in gits {
                    let arg_create_dir: String = format!("mkdir -p {}", git.path);
                    let arg_git: String = format!("git clone {}", git.url);
                    let mut create_dir: bool = false;
                    if !Path::new(&git.path).is_dir() {
                        create_dir = true;
                    }
                    if git.root && is_user_root() {
                        if create_dir {
                            result = result && execute_status(&arg_create_dir, "/");
                        }
                        result = result && execute_status(&arg_git, &git.path);
                    } else if !git.root && !is_user_root() {
                        if create_dir {
                            result = result && execute_status(&arg_create_dir, "/");
                        }
                        result = result && execute_status(&arg_git, &git.path);
                    }
                }
                result
            }
            None => true,
        };
        let add_unzip: bool = match self.diff.add.unzip {
            Some(ref unzips) => {
                printmsg("Adding", "Unzips", &unzips);
                let mut result: bool = true;
                for unzip in unzips {
                    let arg_create_dir: String = format!("mkdir -p {}", unzip.path);
                    let arg_unzip: String = format!("unzip {}.zip", unzip.path);
                    let mut create_dir: bool = false;
                    if !Path::new(&unzip.path).is_dir() {
                        create_dir = true;
                    }
                    if unzip.root && is_user_root() {
                        if create_dir {
                            result = result && execute_status(&arg_create_dir, "/");
                        }
                        result = result && execute_status(&arg_unzip, &unzip.path);
                    } else if !unzip.root && !is_user_root() {
                        if create_dir {
                            result = result && execute_status(&arg_create_dir, "/");
                        }
                        result = result && execute_status(&arg_unzip, &unzip.path);
                    }
                }
                result
            }
            None => true,
        };
        add_curl && add_git && add_unzip
    }
}

impl Add for MonitorDiff {
    fn add(&self) -> bool {
        if self.diff.add.monitors != None || self.diff.remove.monitors != None {
            match self.config.monitors {
                Some(ref monitors) => {
                    printmsg("Adding", "Monitors", &monitors);
                    let mut result: bool = true;
                    fs::create_dir_all(
                        Path::new(HYPR_MONITOR_CONF_PATH)
                            .parent()
                            .expect("Error (Expect): Failed retrieving Directory of HYPR_MONITOR_CONF_PATH"),
                    )
                    .expect("Error (Expect): Failed creating parent directories for HYPR_MONITOR_CONF_PATH");
                    fs::File::create(HYPR_MONITOR_CONF_PATH)
                        .expect("Error (Expect): Failed creating HYPR_MONITOR_CONF_PATH");
                    for monitor in monitors {
                        let mut append_monitor_str: String = format!(
                            "monitor={}, {}@{}, {}, {}\n",
                            monitor.connection,
                            monitor.resolution,
                            monitor.refreshrate,
                            monitor.position,
                            monitor.scale
                        );
                        for workspace in monitor.workspaces.clone() {
                            let workspace_str = format!(
                                "workspace={}, monitor:{}\n",
                                workspace, monitor.connection
                            );
                            append_monitor_str.push_str(&workspace_str);
                        }
                        result = result
                            && prepend_to_file(
                                Path::new(HYPR_MONITOR_CONF_PATH),
                                &append_monitor_str,
                            )
                    }
                    result
                }
                None => true,
            }
        } else {
            true
        }
    }
}

impl Add for FilesDiff {
    fn add(&self) -> bool {
        match self.diff.add.files {
            Some(ref files) => {
                printmsg("Adding", "Files", &files);
                let mut result: bool = true;
                for file in files.clone() {
                    let mut createdir: bool = false;
                    let arg_create_dir: String = format!("mkdir -p {}", file.path);
                    let file_path: String = format!("{}/{}", file.path, file.file_name);
                    if !Path::new(&file.path).is_dir() {
                        createdir = true;
                    }

                    if file.root && is_user_root() {
                        if createdir {
                            result = result && execute_status(&arg_create_dir, "/");
                        }
                        result = result && write_to_file(Path::new(&file_path), &file.write);
                    } else if !file.root && !is_user_root() {
                        if createdir {
                            result = result && execute_status(&arg_create_dir, "/");
                        }
                        result = result && write_to_file(Path::new(&file_path), &file.write);
                    }
                }
                result
            }
            None => true,
        }
    }
}
