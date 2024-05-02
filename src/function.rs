use std::{fs, path::Path, process::Output};

use crate::{
    data_types::{
        DirectoriesDiff, DownloadsDiff, Fail2BanDiff, FilesDiff, GrubDiff, KeyboardDiff,
        LanguageDiff, MkinitcpioDiff, MonitorDiff, PackagesDiff, PacmanDiff, ServicesDiff,
        SystemDiff, TimeDiff, UfwDiff, UserDiff,
    },
    helper::{
        self, append_to_file, execute_output, execute_status, is_user_root, prepend_to_file,
        remove_from_file, replace_line, write_to_file,
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
            Some(ref val) => {
                println!("Adding KEYMAP: {}", val);
                let replace_string: String = format!("KEYMAP={}", val);
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
            Some(ref val) => {
                println!("Adding Timezone: {}", val);
                let argument: String = format!("timedatectl set-timezone {}", val);
                println!("{argument}");
                execute_status(&argument, "/")
            }
            None => true,
        }
    }
}

impl Add for LanguageDiff {
    fn add(&self) -> bool {
        if self.diff.add.character != None || self.diff.add.locale != None {
            println!("Adding Locale: {}", self.config.locale.clone().unwrap());
            let write: String = format!("LANG={}\n", self.config.locale.clone().unwrap());
            let add_character: bool = helper::write_to_file(Path::new(LOCALE_CONF_PATH), &write);
            println!(
                "Adding Character: {}",
                self.config.character.clone().unwrap()
            );
            let write: String = format!(
                "{} {}\n",
                self.config.locale.clone().unwrap(),
                self.config.character.clone().unwrap()
            );
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
            Some(ref val) => {
                println!("Adding Hostname: {}", val);
                let hostname_str: String = val.to_string();
                let hostname: bool = helper::write_to_file(Path::new(HOSTNAME_PATH), &hostname_str);

                let string_to_write: String = format!(
                    "127.0.0.1       localhost\n::1             localhost\n127.0.1.1       {}.localdomain {}",
                    &hostname_str, &hostname_str
                );
                let hosts: bool = helper::write_to_file(Path::new(HOSTS_PATH), &string_to_write);
                hostname && hosts
            }
            None => true,
        }
    }
}

impl Add for UserDiff {
    fn add(&self) -> bool {
        let add_user: bool = match self.diff.add.user_list {
            Some(ref val) => {
                println!("Adding User(s): {:?}", val);
                let mut result: bool = true;
                for user in val {
                    let argument: String = format!("useradd {}", user);
                    result = result && execute_status(&argument, "/");
                }
                result
            }
            None => true,
        };
        let add_group: bool = match self.diff.add.user_groups {
            Some(ref val) => {
                println!("Adding Group(s): {:?}", val);
                let mut result: bool = true;
                for user_group_struct in val {
                    let mut argument: String = String::from("usermod -aG ");
                    for group in user_group_struct.groups.clone() {
                        argument.push_str(&group);
                        argument.push(',');

                        // check if group even exists yet
                        let check_argument: String = format!("getent group {}", group);
                        let output: Output = execute_output(&check_argument, "/")
                            .expect("Able to execute getent command");
                        let output_string: String = String::from_utf8(output.stdout)
                            .expect("Conversion from utf8 to String");
                        if output_string == "" {
                            let create_group_argument: String = format!("groupadd {}", group);
                            result = result && execute_status(&create_group_argument, "/");
                        }
                    }
                    argument.pop();
                    argument.push(' ');
                    argument.push_str(&user_group_struct.name);

                    result = result && execute_status(&argument, "/");
                }
                result
            }
            None => true,
        };
        add_user && add_group
    }
}

impl Remove for UserDiff {
    fn remove(&self) -> bool {
        let remove_user_group: bool = match self.diff.remove.user_groups {
            Some(ref val) => {
                println!("Removing Group(s): {:?}", val);
                let mut result: bool = true;
                for user_group_struct in val {
                    for group in user_group_struct.groups.clone() {
                        let argument: String =
                            format!("gpasswd -d {} {}", user_group_struct.name, group);
                        result = result && execute_status(&argument, "/");
                    }
                }
                result
            }
            None => true,
        };

        let remove_user: bool = match self.diff.remove.user_list {
            Some(ref val) => {
                println!("Removing User(s): {:?}", val);
                let mut result: bool = true;
                for user in val {
                    let argument: String = format!("userdel {}", user);
                    result = result && execute_status(&argument, "/");
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
            Some(ref val) => {
                println!("Adding Parallel: {}", val);
                let argument: String = format!("ParallelDownloads = {val}\n");
                replace_line(Path::new(PACMAN_CONF_PATH), "ParallelDownloads", &argument)
            }
            None => true,
        }
    }
}

impl Add for PackagesDiff {
    fn add(&self) -> bool {
        let add_pacman: bool = match self.diff.add.pacman_packages {
            Some(ref val) => {
                println!("Adding Arch Package(s): {:?}", val);
                let mut argument: String = String::from("pacman --noconfirm -S ");
                for package in val {
                    argument.push_str(&package);
                    argument.push(' ');
                }
                execute_status(&argument, "/")
            }
            None => true,
        };
        let add_aur: bool = match self.diff.add.aur_packages {
            Some(ref val) => {
                println!("Adding AUR Package(s): {:?}", val);
                let mut argument: String = String::from("paru --noconfirm -S ");
                for package in val {
                    argument.push_str(&package);
                    argument.push(' ');
                }
                execute_status(&argument, "/")
            }
            None => true,
        };
        add_pacman && add_aur
    }
}

impl Remove for PackagesDiff {
    fn remove(&self) -> bool {
        let remove_pacman: bool = match self.diff.remove.pacman_packages {
            Some(ref val) => {
                println!("Removing Arch Package(s): {:?}", val);
                let mut argument: String = String::from("pacman --noconfirm -Rns ");
                for package in val {
                    argument.push_str(&package);
                    argument.push(' ');
                }
                execute_status(&argument, "/")
            }
            None => true,
        };
        let remove_aur: bool = match self.diff.remove.aur_packages {
            Some(ref val) => {
                println!("Removing AUR Package(s): {:?}", val);
                let mut argument: String = String::from("paru --noconfirm -Rns ");
                for package in val {
                    argument.push_str(&package);
                    argument.push(' ');
                }
                execute_status(&argument, "/")
            }
            None => true,
        };
        remove_pacman && remove_aur
    }
}

impl Add for ServicesDiff {
    fn add(&self) -> bool {
        let add_services: bool = match self.diff.add.services {
            Some(ref val) => {
                println!("Add Service(s): {:?}", val);
                let mut result: bool = true;
                for service in val {
                    let argument: String = format!("systemctl enable {}", service);
                    result = result && execute_status(&argument, "/");
                }
                result
            }
            None => true,
        };
        let add_user_services: bool = match self.diff.add.user_services {
            Some(ref val) => {
                println!("Add User Service(s): {:?}", val);
                let mut result: bool = true;
                for service in val {
                    let argument: String = format!("systemctl --user enable {}", service,);
                    result = result && execute_status(&argument, "/");
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
        let add_services: bool = match self.diff.remove.services {
            Some(ref val) => {
                println!("Removing Service(s): {:?}", val);
                let mut result: bool = true;
                for service in val {
                    let argument: String = format!("systemctl disable {}", service);
                    result = result && execute_status(&argument, "/");
                }
                result
            }
            None => true,
        };
        let add_user_services: bool = match self.diff.remove.user_services {
            Some(ref val) => {
                println!("Removing User Service(s): {:?}", val);
                let mut result: bool = true;
                for service in val {
                    let argument: String = format!("systemctl --user disable {}", service);
                    result = result && execute_status(&argument, "/");
                }
                result
            }
            None => true,
        };

        add_services && add_user_services
    }
}

impl Add for DirectoriesDiff {
    fn add(&self) -> bool {
        let mut add_reown_dirs: bool = true;
        if is_user_root() {
            add_reown_dirs = match self.diff.add.reown_dirs {
                Some(ref val) => {
                    println!("Adding Group to Dir(s): {:?}", val);
                    let mut result: bool = true;
                    for dir in val {
                        if !Path::new(&dir.directory).exists() {
                            let argument: String = format!("mkdir -p {}", dir.directory);
                            result = result && execute_status(&argument, "/");
                        }
                        let argument: String = format!("chown :{} {}", dir.group, dir.directory);
                        result = result && execute_status(&argument, "/");
                    }
                    result
                }
                None => true,
            };
        }
        let add_create_dirs: bool = match self.diff.add.create_dirs {
            Some(ref val) => {
                println!("Adding Dir(s): {:?}", val);
                let mut result: bool = true;
                for dir in val {
                    let argument: String = format!("mkdir -p {}", dir.path);
                    if dir.root && is_user_root() {
                        result = result && execute_status(&argument, "/");
                    } else if !dir.root && !is_user_root() {
                        result = result && execute_status(&argument, "/");
                    }
                }
                result
            }
            None => true,
        };
        let add_links: bool = match self.diff.add.links {
            Some(ref val) => {
                println!("Adding Link(s): {:?}", val);
                let mut result: bool = true;
                for link in val {
                    if !Path::new(&link.origin).is_dir() {
                        println!("Origin ({}) does not exist. Skipping...", link.origin);
                    } else {
                        // get the contents of origin
                        let argument: String = format!("ls -A {}", link.origin);
                        let output: Output = execute_output(&argument, "/").expect("ls");
                        let output_string: String =
                            String::from_utf8(output.stdout).expect("Conversion String to Utf8");

                        // go through the files in origin
                        for line in output_string.lines() {
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
