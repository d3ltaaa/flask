use crate::data_types::{
    DirectoriesDiff, DownloadsDiff, Fail2BanDiff, FilesDiff, GrubDiff, KeyboardDiff, LanguageDiff,
    MkinitcpioDiff, MonitorDiff, PackagesDiff, PacmanDiff, ServicesDiff, ShellDiff, SystemDiff,
    TimeDiff, UfwDiff, UserDiff,
};
use crate::helper::{execute_output, execute_status, is_user_root, read_in_variable};
use crate::structure::{
    CreateDirs, CurlDownload, Fail2Ban, GitDownload, Links, ManualInstallPackages, Monitor,
    MonitorStruct, ReownDirs, TextToFile, Unzip, User,
};
use crate::{
    FAIL2BAN_JAIL_LOCAL_PATH, GRUB_PATH, HOSTNAME_PATH, HYPR_MONITOR_CONF_PATH, LOCALE_CONF_PATH,
    LOCALE_GEN_PATH, MKINITCPIO_PATH, PACMAN_CONF_PATH,
};
use std::fs::{self};
use std::path::Path;
use std::u8;

macro_rules! SetNoneForVecIfNeededInSystem {
    ($self: ident, $field: ident, $vector: ident) => {
        if $vector.len() > 0 {
            if $vector.get(0) == Some(&"".to_string()) {
                $self.system.$field = None;
            } else {
                $self.system.$field = Some($vector);
            }
        } else {
            $self.system.$field = None;
        }
    };
}

pub trait GetSystem {
    fn get_system(&mut self);
}

impl GetSystem for KeyboardDiff {
    fn get_system(&mut self) {
        self.system.keyboard_tty = None;
        match fs::read_to_string(Path::new(MKINITCPIO_PATH)) {
            Ok(content_string) => {
                self.system.mkinitcpio = match read_in_variable(&content_string, "=", "KEYMAP") {
                    Some(keymap) => Some(keymap),
                    None => None,
                }
            }
            Err(_) => panic!("Error (panic): Failed to read mkinitcpio.conf"),
        }
    }
}

impl GetSystem for TimeDiff {
    fn get_system(&mut self) {
        match execute_output("timedatectl show", "/") {
            Ok(out_timedatectl) => {
                self.system.timezone = match read_in_variable(
                    String::from_utf8(out_timedatectl.stdout).unwrap().as_str(),
                    "=",
                    "Timezone",
                ) {
                    Some(var) => Some(var),
                    None => None,
                }
            }
            Err(_) => panic!("Error (panic): Failed to execute timedatectl show"),
        }
    }
}

impl GetSystem for LanguageDiff {
    fn get_system(&mut self) {
        match fs::read_to_string(Path::new(LOCALE_CONF_PATH)) {
            Ok(content_string) => {
                self.system.locale = match read_in_variable(content_string.as_str(), "=", "LANG") {
                    Some(locale) => Some(locale),
                    None => None,
                }
            }
            Err(_) => {
                println!("{LOCALE_CONF_PATH} not found");
                self.system.locale = None
            }
        }

        match fs::read_to_string(Path::new(LOCALE_GEN_PATH)) {
            Ok(content_string) => {
                self.system.character = match content_string
                    .trim()
                    .split(' ')
                    .collect::<Vec<&str>>()
                    .get(1)
                {
                    Some(var_character) => Some(var_character.to_string()),
                    None => None,
                }
            }
            Err(_) => {
                println!("{LOCALE_CONF_PATH} not found");
                self.system.locale = None
            }
        }
    }
}

impl GetSystem for SystemDiff {
    fn get_system(&mut self) {
        self.system.hostname = match fs::read_to_string(Path::new(HOSTNAME_PATH)) {
            Ok(var_hostname) => Some(var_hostname.trim().to_string()),
            Err(_) => None,
        };
    }
}

impl GetSystem for ShellDiff {
    fn get_system(&mut self) {
        let default_shell: String = match execute_output("echo $SHELL", "/") {
            Ok(out) => String::from_utf8(out.stdout)
                .expect("Error (expect): Failed to convert from utf8 to String")
                .trim()
                .to_string(),
            Err(err) => panic!("Error (panic): Failed to execute echo $SHELL - {err}"),
        };
        self.system.default_shell = Some(default_shell);
    }
}

impl GetSystem for UserDiff {
    fn get_system(&mut self) {
        // to get the users currently on the system, we can make use of the
        // getent command: $(getent passwd {1000..1401}
        // this searches /etc/passwd for all the userowned ids possible and puts out every user
        // it can find
        // this is the output:
        // falk:x:1000:1000::/home/falk:/usr/bin/zsh
        // joel:x:1001:1002::/home/joel:/usr/bin/bash
        // so it can be split by columns : and the first argument is the user name

        let out_get_user: String = match execute_output("getent passwd {1000..1401}", "/") {
            Ok(output) => String::from_utf8(output.stdout)
                .expect("Error (expect): Failed to convert from utf8 to String"),
            Err(_) => panic!("Error (panic): Failed to execute out_get_user"),
        };

        let mut user_groups_vec: Vec<User> = Vec::new(); // create a vector to store the users data
                                                         // type with a groups vector
        let mut user_list_vec: Vec<String> = Vec::new(); // create a vector to list the users

        for line in out_get_user.lines() {
            // retrieve user name from output
            let user_name: String = match line.split(':').collect::<Vec<&str>>().get(0) {
                Some(user) => user.to_string(),
                None => panic!("Error (panic): Error while reading username"),
            };

            // get the groups of that user
            let arg_get_groups: String = format!("groups {}", user_name);
            let out_get_groups: String = match execute_output(&arg_get_groups, "/") {
                Ok(output) => String::from_utf8(output.stdout)
                    .expect("Error (expect): Failed to convert utf8 to String"),
                Err(_) => panic!("Error (panic): Failed to execute arg_get_groups"),
            };

            let mut group_vec: Vec<String> = Vec::new(); // create a vector for the groups

            for group in out_get_groups.trim().split(' ').collect::<Vec<&str>>() {
                if group != user_name {
                    group_vec.push(group.to_string()); // add to group string
                }
            }

            user_groups_vec.push(User {
                name: user_name.clone(),
                groups: group_vec,
            }); // add users to user vector
            user_list_vec.push(user_name);
        }

        if user_groups_vec.len() == 0 {
            // if there is no element in the user vector, then it should be None
            self.system.user_groups = None;
        } else {
            self.system.user_groups = Some(user_groups_vec);
        }

        if user_list_vec.len() == 0 {
            self.system.user_list = None;
        } else {
            self.system.user_list = Some(user_list_vec);
        }
    }
}

impl GetSystem for PacmanDiff {
    fn get_system(&mut self) {
        match fs::read_to_string(Path::new(PACMAN_CONF_PATH)) {
            Ok(content_string) => {
                self.system.parallel =
                    match read_in_variable(&content_string, " = ", "ParallelDownloads") {
                        Some(parallel) => Some(
                            parallel
                                .parse::<u8>()
                                .expect("Error (expect): Failed to parse String to u8"),
                        ),
                        None => None,
                    }
            }
            Err(_) => panic!("Error (panic): Failed to read pacman.conf"),
        }
    }
}

impl GetSystem for ServicesDiff {
    fn get_system(&mut self) {
        if is_user_root() {
            // get services
            let out_get_services: String =
                match execute_output("systemctl list-unit-files --state=enabled", "/") {
                    Ok(output) => String::from_utf8(output.stdout)
                        .expect("Error (Expect): Failed to convert utf8 to String"),
                    Err(_) => panic!("Error (Panic): Failed to execute out_get_services!"),
                };
            let mut services_vec: Vec<String> = Vec::new();

            for line in out_get_services.lines() {
                if line.contains("enabled disabled") && line.contains(".service") {
                    match line.split(' ').collect::<Vec<&str>>().get(0) {
                        Some(service) => services_vec.push(service.to_string()),
                        None => (),
                    }
                }
            }
            if services_vec.len() > 0 {
                self.system.services = Some(services_vec);
            }
        } else {
            // get user services
            let out_get_user_services: String =
                match execute_output("systemctl --user list-unit-files --state=enabled", "/") {
                    Ok(output) => String::from_utf8(output.stdout)
                        .expect("Error (Expect): Failed to convert from utf8 to String"),
                    Err(_) => panic!("Error (panic): Failed to execute out_get_user_services"),
                };

            let mut user_services_vec: Vec<String> = Vec::new();

            for line in out_get_user_services.lines() {
                if line.contains("enabled") && line.contains(".service") {
                    match line.split(' ').collect::<Vec<&str>>().get(0) {
                        Some(service) => user_services_vec.push(service.to_string()),
                        None => (),
                    };
                }
            }

            if user_services_vec.len() > 0 {
                self.system.user_services = Some(user_services_vec);
            }
        }
    }
}

impl GetSystem for PackagesDiff {
    fn get_system(&mut self) {
        if is_user_root() {
            // get pacman_packages
            let out_get_pacman_packages: String = match execute_output("pacman -Qen", "/") {
                Ok(output) => String::from_utf8(output.stdout)
                    .expect("Error (expect): Failed to convert from utf8 to String"),
                Err(_) => panic!("Error (panic): Failed to execute out_get_pacman_packages"),
            };

            let mut arch_vec: Vec<String> = Vec::new();

            for line in out_get_pacman_packages.trim().lines() {
                match line.split(' ').collect::<Vec<&str>>().get(0) {
                    Some(pacman_package) => arch_vec.push(pacman_package.to_string()),
                    None => (),
                };
            }
            SetNoneForVecIfNeededInSystem!(self, pacman_packages, arch_vec);
        } else {
            // get aur_packages
            let out_get_aur_packages: String = match execute_output("pacman -Qem", "/") {
                Ok(output) => String::from_utf8(output.stdout)
                    .expect("Error (expect): Failed to convert from utf8 to String"),
                Err(_) => panic!("Error (panic): Failed to execute out_get_aur_packages"),
            };

            let mut aur_vec: Vec<String> = Vec::new();

            for line in out_get_aur_packages.trim().lines() {
                if !line.contains("paru") {
                    match line.split(' ').collect::<Vec<&str>>().get(0) {
                        Some(paru_package) => aur_vec.push(paru_package.to_string()),
                        None => (),
                    };
                }
            }
            SetNoneForVecIfNeededInSystem!(self, aur_packages, aur_vec);

            // get manual_install_packages
            let mut manual_vec: Vec<ManualInstallPackages> = Vec::new();
            match self.config.manual_install_packages.clone() {
                Some(packages) => {
                    for package in packages {
                        if package.root == false {
                            if execute_status(&package.check, "/") {
                                manual_vec.push(package);
                            }
                        } else {
                            manual_vec.push(package);
                        }
                    }
                    if manual_vec.len() > 0 {
                        self.system.manual_install_packages = Some(manual_vec);
                    }
                }
                None => self.system.manual_install_packages = None,
            }
        }
        // get manual_install_packages
        let mut manual_vec: Vec<ManualInstallPackages> = Vec::new();
        match self.config.manual_install_packages.clone() {
            Some(packages) => {
                for package in packages {
                    if is_user_root() == package.root {
                        if execute_status(&package.check, "/") {
                            manual_vec.push(package);
                        }
                    } else {
                        manual_vec.push(package);
                    }
                }
                if manual_vec.len() > 0 {
                    self.system.manual_install_packages = Some(manual_vec);
                }
            }
            None => self.system.manual_install_packages = None,
        }
    }
}

impl GetSystem for DirectoriesDiff {
    fn get_system(&mut self) {
        // since we cannot check all dirs we can only check wether or not every dir and link that
        // should be there is actuall there, therefore only the add part makes sense
        // reown_dirs

        match self.config.reown_dirs.clone() {
            Some(reown_dirs) => {
                if is_user_root() {
                    let mut reown_dirs_vec: Vec<ReownDirs> = Vec::new();
                    for reown_dir in reown_dirs {
                        if Path::new(&reown_dir.directory).is_dir() {
                            let arg_get_owner: String = format!("ls -ld {}", reown_dir.directory);
                            let out_get_owner: String = match execute_output(&arg_get_owner, "/") {
                                Ok(output) => String::from_utf8(output.stdout)
                                    .expect("Error (expect): Failed to convert utf8 to String"),
                                Err(_) => panic!("Error (panic): Failed to execute arg_get_owner"),
                            };
                            let owner_group: String =
                                match out_get_owner.split(' ').collect::<Vec<&str>>().get(3) {
                                    Some(owner) => owner.to_string(),
                                    None => panic!(
                                        "Error (panic): Failed to read owner from out_get_owner"
                                    ),
                                };
                            if owner_group == reown_dir.group {
                                reown_dirs_vec.push(reown_dir);
                            }
                        }
                    }
                    if reown_dirs_vec.len() > 0 {
                        self.system.reown_dirs = Some(reown_dirs_vec);
                    } else {
                        self.system.reown_dirs = None;
                    }
                }
            }

            None => self.system.reown_dirs = None,
        }

        match self.config.create_dirs.clone() {
            Some(create_dirs) => {
                // create_dirs
                let mut create_dirs_vec: Vec<CreateDirs> = Vec::new();
                for create_dir in create_dirs {
                    if create_dir.root == is_user_root() {
                        if Path::new(&create_dir.path).is_dir() {
                            create_dirs_vec.push(create_dir);
                        }
                    } else {
                        create_dirs_vec.push(create_dir);
                    }
                }
                if create_dirs_vec.len() > 0 {
                    self.system.create_dirs = Some(create_dirs_vec);
                }
            }
            None => self.system.create_dirs = None,
        }

        let mut links_vec: Vec<Links> = Vec::new();
        let is_user_root = is_user_root();
        for link in self.config.links.clone().unwrap() {
            if link.root == is_user_root {
                let mut file_vec: Vec<String> = Vec::new();
                let arg_get_links: String = format!("ls -A {}", link.origin);
                let out_get_links: String = match execute_output(&arg_get_links, "/") {
                    Ok(output) => String::from_utf8(output.stdout)
                        .expect("Error (expect): Failed to convert from utf8 to String"),
                    Err(_) => panic!("Error (panic): Failed to execute arg_get_links"),
                };
                for line in out_get_links.lines() {
                    file_vec.push(line.trim().to_string());
                }

                let mut all_links_are_ok: bool = true;
                for file in file_vec {
                    let origin_string: String = format!("{}/{}", link.origin, file);
                    let destination_string: String = format!("{}/{}", link.destination, file);

                    if Path::new(&link.origin).is_dir()
                        && Path::new(&destination_string).is_symlink()
                    {
                        let mut arg_get_link: String = String::new();
                        if Path::new(&destination_string).is_dir() {
                            arg_get_link = format!("ls -ldA1 {}", destination_string);
                        } else if Path::new(&destination_string).is_file() {
                            arg_get_link = format!("ls -lA1 {}", destination_string);
                        }

                        let out_get_link: String = match execute_output(&arg_get_link, "/") {
                            Ok(output) => String::from_utf8(output.stdout)
                                .expect("Error (expect): Failed to convert from utf8 to String"),
                            Err(_) => panic!("Error (panic): Failed to execute arg_get_link"),
                        };

                        let real_origin_string: String =
                            match out_get_link.split("->").collect::<Vec<&str>>().last() {
                                Some(origin) => origin.trim().to_string(),
                                None => panic!("Error (panic): Failed to read from out_get_link"),
                            };

                        if real_origin_string != origin_string {
                            all_links_are_ok = false;
                        }
                    } else {
                        all_links_are_ok = false;
                    }
                }

                if all_links_are_ok == true && Path::new(&link.origin).is_dir() {
                    links_vec.push(link);
                }
            } else {
                links_vec.push(link);
            }
        }

        if links_vec.len() > 0 {
            self.system.links = Some(links_vec);
        } else {
            self.system.links = None;
        }
    }
}

impl GetSystem for GrubDiff {
    fn get_system(&mut self) {
        let s: String = match fs::read_to_string(Path::new(GRUB_PATH)) {
            Ok(str) => str,
            Err(_) => panic!("Error (panic): Failed to read grub"),
        };
        // GRUB_CMDLINE_LINUX_DEFAULT
        let mut var_cmdline_linux_default: String =
            match read_in_variable(&s, "=", "GRUB_CMDLINE_LINUX_DEFAULT") {
                Some(var) => var,
                None => "  ".to_string(),
            };
        // remove "" from argument_string
        var_cmdline_linux_default.pop();
        if var_cmdline_linux_default.len() > 0 {
            var_cmdline_linux_default.remove(0);
        }

        let mut cmdline_default_vec: Vec<String> = Vec::new();
        for var in var_cmdline_linux_default.split(' ').collect::<Vec<&str>>() {
            cmdline_default_vec.push(var.to_string());
        }
        SetNoneForVecIfNeededInSystem!(self, grub_cmdline_linux_default, cmdline_default_vec);

        // GRUB_TIMEOUT
        let var_timeout: Option<i8> = match read_in_variable(&s, "=", "GRUB_TIMEOUT") {
            Some(var) => Some(
                var.parse::<i8>()
                    .expect("Error (expect): Failed to convert from String to i8"),
            ),
            None => match read_in_variable(&s, "=", "#GRUB_TIMEOUT") {
                Some(var) => Some(
                    var.parse::<i8>()
                        .expect("Error (expect): Failed to convert from String to i8"),
                ),
                None => None,
            },
        };
        self.system.grub_timeout = var_timeout;

        // GRUB_RESUME
        let var_cmdline_linux_with_newlines: String =
            match read_in_variable(&s, "=", "GRUB_CMDLINE_LINUX") {
                Some(cmdline_linux) => cmdline_linux
                    .trim()
                    .to_string()
                    .remove(0)
                    .to_string()
                    .remove(cmdline_linux.len())
                    .to_string()
                    .split(' ')
                    .collect::<Vec<&str>>()
                    .join("\n"),
                None => {
                    panic!("Error (panic): Failed to locate GRUB_CMDLINE_LINUX in {GRUB_PATH}!")
                }
            };

        self.system.grub_resume = read_in_variable(&var_cmdline_linux_with_newlines, "=", "resume");

        // GRUB_CRYPT
        self.system.grub_crypt =
            match read_in_variable(&var_cmdline_linux_with_newlines, "=", "cryptdevice") {
                Some(crypt_string) => match crypt_string.split_once(":") {
                    Some(splitted_crypt_string) => Some(splitted_crypt_string.1.to_string()),
                    None => None,
                },
                None => None,
            }
    }
}

impl GetSystem for MkinitcpioDiff {
    fn get_system(&mut self) {
        let s: String = match fs::read_to_string(Path::new(MKINITCPIO_PATH)) {
            Ok(str) => str,
            Err(_) => panic!("Error (panic): Failed to read mkinitcpio.conf"),
        };
        // MODULES
        let mut var_modules: String = match read_in_variable(&s, "=", "MODULES") {
            Some(var) => var,
            None => "  ".to_string(),
        };

        // remove () from argument_string
        var_modules.pop();
        if var_modules.len() > 0 {
            var_modules.remove(0);
        }

        let mut modules_vec: Vec<String> = Vec::new();
        for var in var_modules.split(' ').collect::<Vec<&str>>() {
            modules_vec.push(var.to_string());
        }
        SetNoneForVecIfNeededInSystem!(self, modules, modules_vec);

        // HOOKS
        let mut var_hooks: String = match read_in_variable(&s, "=", "HOOKS") {
            Some(var) => var,
            None => "  ".to_string(),
        };

        // remove () from argument_string
        var_hooks.pop();
        if var_hooks.len() > 0 {
            var_hooks.remove(0);
        }

        let mut hooks_vec: Vec<String> = Vec::new();
        for var in var_hooks.split(' ').collect::<Vec<&str>>() {
            hooks_vec.push(var.to_string());
        }
        SetNoneForVecIfNeededInSystem!(self, hooks, hooks_vec);
    }
}

impl GetSystem for DownloadsDiff {
    fn get_system(&mut self) {
        match self.config.git.clone() {
            Some(config_git_vec) => {
                let mut git_vec: Vec<GitDownload> = Vec::new();
                for git in config_git_vec {
                    if git.root == is_user_root() {
                        let git_dir_name: String =
                            match git.url.split('/').collect::<Vec<&str>>().last() {
                                Some(last) => match last.split_once('.') {
                                    Some(last_splitted) => last_splitted.0.to_string(),
                                    None => panic!("Error (panic): Failed to read path from git"),
                                },
                                None => panic!("Error (panic): Failed to read path from git"),
                            };
                        let git_path: String = format!("{}/{}", git.path, git_dir_name);
                        if Path::new(&git_path).is_dir() {
                            git_vec.push(git);
                        }
                    } else {
                        git_vec.push(git);
                    }
                }

                if git_vec.len() > 0 {
                    self.system.git = Some(git_vec);
                }
            }
            None => (),
        }

        match self.config.curl.clone() {
            Some(config_curl_vec) => {
                let mut curl_vec: Vec<CurlDownload> = Vec::new();
                for curl in config_curl_vec {
                    if is_user_root() == curl.root {
                        let path_string: String = format!("{}/{}", curl.path, curl.file_name);
                        let file_path: &Path = Path::new(&path_string);
                        if file_path.is_file() {
                            curl_vec.push(curl);
                        }
                    } else {
                        curl_vec.push(curl);
                    }
                }
                if curl_vec.len() > 0 {
                    self.system.curl = Some(curl_vec);
                }
            }
            None => (),
        };

        match self.config.unzip.clone() {
            Some(config_unzip_vec) => {
                let mut zip_vec: Vec<Unzip> = Vec::new();
                for zip in config_unzip_vec {
                    if is_user_root() == zip.root {
                        let zip_path: &Path = Path::new(&zip.path);
                        if zip_path.is_dir() {
                            zip_vec.push(zip);
                        }
                    } else {
                        zip_vec.push(zip);
                    }
                }
                if zip_vec.len() > 0 {
                    self.system.unzip = Some(zip_vec);
                }
            }
            None => (),
        };
    }
}

impl GetSystem for UfwDiff {
    fn get_system(&mut self) {
        let out_ufw_status: String = match execute_output("sudo ufw status verbose", "/") {
            Ok(output) => String::from_utf8(output.stdout)
                .expect("Error (expect): Failed to convert utf8 to String"),
            Err(_) => panic!("Error (panic): Failed to execute out_ufw_status"),
        };

        let val_default: String = match read_in_variable(&out_ufw_status, ":", "Default") {
            Some(default) => default.to_string(),
            None => String::new(),
        };

        let val_incoming: String = match val_default.split_once("(incoming)") {
            Some(incoming) => match incoming.0.trim().split(' ').collect::<Vec<&str>>().last() {
                Some(last) => last.to_string(),
                None => String::new(),
            },
            None => String::new(),
        };

        if val_incoming != "" {
            self.system.incoming = Some(val_incoming);
        }

        let val_outgoing: String = match val_default.split_once("(outgoing)") {
            Some(outgoing) => match outgoing.0.trim().split(' ').collect::<Vec<&str>>().last() {
                Some(last) => last.to_string(),
                None => String::new(),
            },
            None => String::new(),
        };

        if val_outgoing != "" {
            self.system.outgoing = Some(val_outgoing);
        }

        let out_rule: Vec<&str> = out_ufw_status.lines().collect::<Vec<&str>>();
        if out_rule.len() > 7 {
            let rule_output: String = out_rule[7..].join("\n");
            let mut rule_vec: Vec<String> = Vec::new();
            for line in rule_output.lines() {
                if !line.contains("(v6)") {
                    match line.split_once(" ") {
                        Some(line_splitted) => rule_vec.push(line_splitted.0.to_string()),
                        None => (),
                    };
                }
            }
            if rule_vec.len() > 0 {
                self.system.rules = Some(rule_vec);
            } else {
                self.system.rules = None;
            }
        } else {
            self.system.rules = None;
        }
    }
}

impl GetSystem for Fail2BanDiff {
    fn get_system(&mut self) {
        match fs::read_to_string(Path::new(FAIL2BAN_JAIL_LOCAL_PATH)) {
            Err(_) => {
                println!("Failed to read jail.local");
                self.system = Fail2Ban {
                    ignoreip: None,
                    bantime: None,
                    findtime: None,
                    maxretry: None,
                    services: None,
                }
            }
            Ok(content_string) => {
                let ignoreip = match read_in_variable(&content_string, " = ", "ignoreip") {
                    None => String::new(),
                    Some(ip) => ip,
                };
                let bantime: usize = match read_in_variable(&content_string, " = ", "bantime") {
                    None => 0,
                    Some(time) => time
                        .parse::<usize>()
                        .expect("Error (expect): Failed to convert String to usize"),
                };
                let findtime: usize = match read_in_variable(&content_string, " = ", "findtime") {
                    None => 0,
                    Some(time) => time
                        .parse::<usize>()
                        .expect("Error (expect): Failed to convert String to usize"),
                };
                let maxretry: usize = match read_in_variable(&content_string, " = ", "maxretry") {
                    None => 0,
                    Some(amount) => amount
                        .parse::<usize>()
                        .expect("Error (expect): Failed to convert String to usize"),
                };
                let mut services: Vec<String> = Vec::new();
                for line in content_string.lines() {
                    if line.contains("[") && line.contains("]") && !line.contains("[DEFAULT]") {
                        let mut service: String = line.to_string();
                        service.pop();
                        if service.len() > 0 {
                            service.remove(0);
                        }
                        services.push(service);
                    }
                }

                self.system.ignoreip = Some(ignoreip);
                self.system.maxretry = Some(maxretry);
                self.system.bantime = Some(bantime);
                self.system.findtime = Some(findtime);
                if services.len() > 0 {
                    self.system.services = Some(services);
                }
            }
        };
    }
}

impl GetSystem for MonitorDiff {
    fn get_system(&mut self) {
        match fs::read_to_string(Path::new(HYPR_MONITOR_CONF_PATH)) {
            Err(_) => {
                println!("Failed to read (hypr/)monitor.conf");
                self.system = Monitor { monitors: None }
            }
            Ok(content_string) => {
                let mut monitor_struct_vec: Vec<MonitorStruct> = Vec::new();

                for line in content_string.lines() {
                    if line.contains("monitor=") {
                        match line.split_once("=") {
                            Some(line_splitted) => {
                                let monitor_vec: String = line_splitted.1.to_string();
                                let monitor_vec: Vec<&str> = monitor_vec.split(", ").collect();
                                monitor_struct_vec.push(MonitorStruct {
                                    connection: match monitor_vec.get(0) {
                                        Some(connection) => connection.to_string(),
                                        None => String::new(),
                                    },
                                    resolution: match monitor_vec.get(1) {
                                        Some(resolution) => match resolution.split_once("@") {
                                            Some(resolution_splitted) => {
                                                resolution_splitted.0.to_string()
                                            }
                                            None => String::new(),
                                        },
                                        None => String::new(),
                                    },
                                    refreshrate: match monitor_vec.get(1) {
                                        Some(resolution) => match resolution.split_once("@") {
                                            Some(resolution_splitted) => {
                                                resolution_splitted.1.to_string()
                                            }
                                            None => String::new(),
                                        },
                                        None => String::new(),
                                    },
                                    position: match monitor_vec.get(2) {
                                        Some(position) => position.to_string(),
                                        None => String::new(),
                                    },
                                    scale: match monitor_vec.get(3) {
                                        Some(scale) => scale.parse::<f32>().expect(
                                            "Error (expect): Failed to convert String to f32",
                                        ),
                                        None => 1.0,
                                    },
                                    workspaces: Vec::new(),
                                });
                            }
                            None => (),
                        }
                    } else if line.contains("workspace=") {
                        match line.split_once("=") {
                            Some(line_splitted) => match line_splitted.1.split_once(", ") {
                                Some(workspace_splitted) => {
                                    let workspace: u8 = workspace_splitted
                                        .0
                                        .parse()
                                        .expect("Error (expect): Failed to convert String to u8");
                                    let connection: String =
                                        workspace_splitted.1.trim().to_string();
                                    for i in 0..monitor_struct_vec.len() {
                                        if format!("monitor:{}", monitor_struct_vec[i].connection)
                                            == connection
                                        {
                                            monitor_struct_vec[i].workspaces.push(workspace);
                                        }
                                    }
                                }
                                None => (),
                            },
                            None => (),
                        }
                    }
                }
                if monitor_struct_vec.len() > 0 {
                    self.system.monitors = Some(monitor_struct_vec);
                }
            }
        }
    }
}

impl GetSystem for FilesDiff {
    fn get_system(&mut self) {
        match self.config.files {
            Some(ref files) => {
                let mut file_vec: Vec<TextToFile> = Vec::new();
                for file in files {
                    if is_user_root() == file.root {
                        let str_file_path: String = format!("{}/{}", file.path, file.file_name);
                        if Path::new(&str_file_path).is_file() {
                            match fs::read_to_string(Path::new(&str_file_path)) {
                                Ok(file_content) => {
                                    if file_content.trim().to_string()
                                        == file.write.trim().to_string()
                                    {
                                        file_vec.push(file.clone());
                                    }
                                }
                                Err(_) => {
                                    panic!("Error (panic): Failed to read from file path")
                                }
                            }
                        }
                    } else {
                        file_vec.push(file.clone());
                    }
                }
                if file_vec.len() > 0 {
                    self.system.files = Some(file_vec);
                }
            }
            None => self.system.files = None,
        }
    }
}
