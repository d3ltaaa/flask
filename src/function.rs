use core::panic;
use std::{fs, path::Path, process::Output};

use crate::{
    data_types::{
        DirectoriesDiff, DownloadsDiff, Fail2BanDiff, FilesDiff, GrubDiff, KeyboardDiff,
        LanguageDiff, MkinitcpioDiff, MonitorDiff, PackagesDiff, PacmanDiff, ServicesDiff,
        SystemDiff, TimeDiff, UfwDiff, UserDiff,
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
        if self.diff.add.character != None || self.diff.add.locale != None {
            let character: String = self.config.character.clone().unwrap();
            let locale: String = self.config.locale.clone().unwrap();
            let msg_string: String = format!("{character} + {locale}");
            printmsg("Adding", "Character + Locale", msg_string);
            let write: String = format!("LANG={}\n", locale);
            let add_character: bool = helper::write_to_file(Path::new(LOCALE_CONF_PATH), &write);
            let write: String = format!("{} {}\n", locale, character);
            let add_locale: bool = helper::write_to_file(Path::new(LOCALE_GEN_PATH), &write);
            let generate_locale: bool = execute_status("locale-gen", "/");
            add_character && add_locale && generate_locale
        } else {
            true
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
                                Ok(output) => String::from_utf8(output.stdout)
                                    .expect("Error: Conversion from utf8 to String failed"),
                                Err(_) => panic!(
                                    "Error: Failed to execute arg_check_for_group: {}",
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
        add_pacman && add_aur
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
                        // get the contents of origin
                        let arg_get_origin_content: String = format!("ls -A {}", link.origin);
                        let out_get_origin_content: String =
                            match execute_output(&arg_get_origin_content, "/") {
                                Ok(output) => String::from_utf8(output.stdout)
                                    .expect("Error: Conversion from utf8 to String failed"),
                                Err(_) => panic!(
                                    "Error: Failed to execute arg_get_origin_content: {}",
                                    arg_get_origin_content
                                ),
                            };

                        // go through the files in origin //TODO
                        for line in out_get_origin_content.lines() {
                            dbg!(line);
                            let argument: String =
                                format!("ln -sf {}/{} {}", link.origin, line, link.destination);
                            let argument_create: String = format!("mkdir -p {}", link.destination);
                            // check if destination dir has to be created
                            let mut create_destination_dir: bool = false;
                            if !Path::new(&link.destination).is_dir() {
                                create_destination_dir = true;
                            }
                            if !Path::new(&link.destination).is_symlink() {
                                if link.root && is_user_root() {
                                    // create destination dir if needed
                                    if create_destination_dir {
                                        println!("{argument_create}");
                                        result = result && execute_status(&argument_create, "/");
                                    }
                                    // actually link
                                    result = result && execute_status(&argument, "/");
                                    dbg!(&argument);
                                } else if !link.root && !is_user_root() {
                                    // create destination dir if needed
                                    if create_destination_dir {
                                        println!("{argument_create}");
                                        result = result && execute_status(&argument_create, "/");
                                    }
                                    // actually link
                                    result = result && execute_status(&argument, "/");
                                    dbg!(&argument);
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
        if self.diff.add.grub_cmdline_linux_default != None
            || self.diff.remove.grub_cmdline_linux_default != None
        {
            println!(
                "Modify Grub: Add({:?}) Remove({:?})",
                self.diff.add.grub_cmdline_linux_default,
                self.diff.remove.grub_cmdline_linux_default
            );

            let mut argument: String = String::from("GRUB_CMDLINE_LINUX_DEFAULT=\"");

            match self.config.grub_cmdline_linux_default.clone() {
                Some(grub_cmdline_linux_default) => {
                    for arg in grub_cmdline_linux_default.clone() {
                        argument.push_str(&arg);
                        argument.push(' ');
                    }
                    if grub_cmdline_linux_default.len() > 0 {
                        argument.pop();
                    }
                }
                None => (),
            }

            argument.push('\"');
            replace_line(
                Path::new(GRUB_PATH),
                "GRUB_CMDLINE_LINUX_DEFAULT=",
                &argument,
            )
        } else {
            true
        }
    }
}

impl Add for MkinitcpioDiff {
    fn add(&self) -> bool {
        let mut add_hooks: bool = true;
        if self.diff.add.hooks != None || self.diff.remove.hooks != None {
            println!(
                "Modify Hooks: Add({:?}) Remove({:?})",
                self.diff.add.hooks, self.diff.remove.hooks
            );

            let mut argument: String = String::from("HOOKS=(");
            for arg in self.config.hooks.clone().unwrap() {
                argument.push_str(&arg);
                argument.push(' ');
            }
            if self.config.hooks.clone().unwrap().len() > 0 {
                argument.pop();
            }
            argument.push(')');
            add_hooks = add_hooks && replace_line(Path::new(MKINITCPIO_PATH), "HOOKS=(", &argument)
        };

        let mut add_modules: bool = true;
        if self.diff.add.modules != None || self.diff.remove.modules != None {
            println!(
                "Modify Modules: Add({:?}) Remove({:?})",
                self.diff.add.modules, self.diff.remove.modules
            );

            let mut argument: String = String::from("MODULES=(");
            match self.config.modules.clone() {
                Some(ref modules) => {
                    for arg in modules {
                        argument.push_str(&arg);
                        argument.push(' ');
                    }
                    if modules.len() > 0 {
                        argument.pop();
                    }
                }
                None => (),
            }
            argument.push(')');
            add_modules =
                add_modules && replace_line(Path::new(MKINITCPIO_PATH), "MODULES=(", &argument);
        }
        add_hooks && add_modules
    }
}

impl Add for UfwDiff {
    fn add(&self) -> bool {
        let add_incoming: bool = match self.diff.add.incoming {
            Some(ref val) => {
                println!("Adding Ufw-Incoming: {}", val);
                let argument: String = format!("ufw default {} incoming", val);
                execute_status(&argument, "/")
            }
            None => true,
        };
        let add_outgoing: bool = match self.diff.add.outgoing {
            Some(ref val) => {
                println!("Adding Ufw-Incoming: {}", val);
                let argument: String = format!("ufw default {} outgoing", val);
                execute_status(&argument, "/")
            }
            None => true,
        };
        let add_rule: bool = match self.diff.add.rules {
            Some(ref val) => {
                println!("Adding Ufw-Rule: {:?}", val);
                let mut result: bool = true;
                for rule in val {
                    let argument: String = format!("ufw allow {}", rule);
                    result = result && execute_status(&argument, "/");
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
            Some(ref val) => {
                println!("Removing Ufw-Rule: {:?}", val);
                let mut result: bool = true;
                for rule in val {
                    let argument: String = format!("ufw delete allow {}", rule);
                    result = result && execute_status(&argument, "/");
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
            Some(ref val) => {
                println!("Adding Ignoreip: {}", val);
                let argument: String = format!("ignoreip = {}", val);
                replace_line(Path::new(FAIL2BAN_JAIL_LOCAL_PATH), "ignoreip =", &argument)
            }
            None => true,
        };
        let add_bantime: bool = match self.diff.add.bantime {
            Some(ref val) => {
                println!("Adding Bantime: {}", val);
                let argument: String = format!("bantime = {}", val);
                replace_line(Path::new(FAIL2BAN_JAIL_LOCAL_PATH), "bantime =", &argument)
            }
            None => true,
        };
        let add_findtime: bool = match self.diff.add.findtime {
            Some(ref val) => {
                println!("Adding Findtime: {}", val);
                let argument: String = format!("findtime = {}", val);
                replace_line(Path::new(FAIL2BAN_JAIL_LOCAL_PATH), "findtime =", &argument)
            }
            None => true,
        };
        let add_maxretry: bool = match self.diff.add.maxretry {
            Some(ref val) => {
                println!("Adding Maxentry: {}", val);
                let argument: String = format!("maxretry = {}", val);
                replace_line(Path::new(FAIL2BAN_JAIL_LOCAL_PATH), "bantim =", &argument)
            }
            None => true,
        };
        let add_services: bool = match self.diff.add.services {
            Some(ref val) => {
                println!("Adding F2B-Services: {:?}", val);
                let mut result: bool = true;
                for service in val {
                    let argument: String = format!("\n[{}]\nenabled = true", service);
                    result =
                        result && append_to_file(Path::new(FAIL2BAN_JAIL_LOCAL_PATH), &argument);
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
            Some(ref val) => {
                println!("Remove F2B-Services: {:?}", val);
                let mut result: bool = true;
                for service in val {
                    let argument: String = format!("\n\n[{}]\nenabled = true", service);
                    result =
                        result && remove_from_file(Path::new(FAIL2BAN_JAIL_LOCAL_PATH), &argument);
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
            Some(ref val) => {
                println!("Adding Curl: {:?}", val);
                let mut result: bool = true;
                for curl in val {
                    let argument_create: String = format!("mkdir -p {}", curl.path);
                    let argument: String =
                        format!("curl -L {} > {}/{}", curl.url, curl.path, curl.file_name);
                    let mut create_dir: bool = false;
                    if !Path::new(&curl.path).is_dir() {
                        create_dir = true;
                    }
                    if curl.root && is_user_root() {
                        if create_dir {
                            result = result && execute_status(&argument_create, "/");
                        }
                        result = result && execute_status(&argument, "/");
                    } else if !curl.root && !is_user_root() {
                        if create_dir {
                            result = result && execute_status(&argument_create, "/");
                        }
                        result = result && execute_status(&argument, "/");
                    }
                }
                result
            }
            None => true,
        };
        let add_git: bool = match self.diff.add.git {
            Some(ref val) => {
                println!("Adding Git: {:?}", val);
                let mut result: bool = true;
                for git in val {
                    let argument_create: String = format!("mkdir -p {}", git.path);
                    let argument: String = format!("git clone {}", git.url);
                    let mut create_dir: bool = false;
                    if !Path::new(&git.path).is_dir() {
                        create_dir = true;
                    }
                    if git.root && is_user_root() {
                        if create_dir {
                            result = result && execute_status(&argument_create, "/");
                        }
                        result = result && execute_status(&argument, &git.path);
                    } else if !git.root && !is_user_root() {
                        if create_dir {
                            result = result && execute_status(&argument_create, "/");
                        }
                        result = result && execute_status(&argument, &git.path);
                    }
                }
                result
            }
            None => true,
        };
        let add_unzip: bool = match self.diff.add.unzip {
            Some(ref val) => {
                println!("Adding Unzip: {:?}", val);
                let mut result: bool = true;
                for unzip in val {
                    let argument_create: String = format!("mkdir -p {}", unzip.path);
                    let argument: String = format!("unzip {}.zip", unzip.path);
                    let mut create_dir: bool = false;
                    if !Path::new(&unzip.path).is_dir() {
                        create_dir = true;
                    }
                    if unzip.root && is_user_root() {
                        if create_dir {
                            result = result && execute_status(&argument_create, "/");
                        }
                        result = result && execute_status(&argument, &unzip.path);
                    } else if !unzip.root && !is_user_root() {
                        if create_dir {
                            result = result && execute_status(&argument_create, "/");
                        }
                        result = result && execute_status(&argument, &unzip.path);
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
        match self.diff.add.monitors {
            Some(ref val) => {
                println!("Adding Monitor(s): {:?}", val);
                let mut result: bool = true;
                // let dir_path: String = HYPR_MONITOR_CONF_PATH
                //     .split('/')
                //     .rev()
                //     .collect::<Vec<&str>>()[1..]
                //     .join("/");
                if !Path::new(HYPR_MONITOR_CONF_PATH).exists() {
                    fs::File::create(HYPR_MONITOR_CONF_PATH)
                        .expect("Creating HYPR_MONITOR_CONF_PATH");
                }
                for monitor in val {
                    let monitor_string: String = format!(
                        "monitor={}, {}@{}, {}, {}",
                        monitor.connection,
                        monitor.resolution,
                        monitor.refreshrate,
                        monitor.position,
                        monitor.scale
                    );
                    result = result
                        && prepend_to_file(Path::new(HYPR_MONITOR_CONF_PATH), &monitor_string)
                }
                result
            }
            None => true,
        }
    }
}

impl Remove for MonitorDiff {
    fn remove(&self) -> bool {
        match self.diff.remove.monitors {
            Some(ref val) => {
                println!("Removing Monitor(s): {:?}", val);
                let mut result: bool = true;
                for monitor in val {
                    let monitor_string: String = format!(
                        "monitor={}, {}@{}, {}, {}\n",
                        monitor.connection,
                        monitor.resolution,
                        monitor.refreshrate,
                        monitor.position,
                        monitor.scale
                    );
                    result = result
                        && remove_from_file(
                            Path::new(HYPR_MONITOR_CONF_PATH),
                            monitor_string.trim(),
                        );
                }
                result
            }
            None => true,
        }
    }
}

impl Add for FilesDiff {
    fn add(&self) -> bool {
        match self.diff.add.files {
            Some(ref val) => {
                println!("Add File(s): {:?}", val);
                let mut result: bool = true;
                for file in val.clone() {
                    let mut createdir: bool = false;
                    let argument_create: String = format!("mkdir -p {}", file.path);
                    let file_path: String = format!("{}/{}", file.path, file.file_name);
                    if !Path::new(&file.path).is_dir() {
                        createdir = true;
                    }

                    if file.root && is_user_root() {
                        if createdir {
                            result = result && execute_status(&argument_create, "/");
                        }
                        result = result && write_to_file(Path::new(&file_path), &file.write);
                    } else if !file.root && !is_user_root() {
                        if createdir {
                            result = result && execute_status(&argument_create, "/");
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
