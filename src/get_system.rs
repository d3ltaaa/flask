use crate::data_types::{
    DirectoriesDiff, DownloadsDiff, Fail2BanDiff, FilesDiff, GrubDiff, KeyboardDiff, LanguageDiff,
    MkinitcpioDiff, MonitorDiff, PackagesDiff, PacmanDiff, ServicesDiff, SystemDiff, TimeDiff,
    UfwDiff, UserDiff,
};
use crate::helper::{execute_output, is_user_root, read_in_variable};
use crate::structure::{
    CreateDirs, CurlDownload, GitDownload, Links, MonitorStruct, ReownDirs, TextToFile, Unzip, User,
};
use crate::{
    FAIL2BAN_JAIL_LOCAL_PATH, GRUB_PATH, HOSTNAME_PATH, HYPR_MONITOR_CONF_PATH, LOCALE_CONF_PATH,
    LOCALE_GEN_PATH, MKINITCPIO_PATH, PACMAN_CONF_PATH,
};
use std::fs;
use std::path::Path;
use std::process::Output;
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

        // get mkinitcpio
        let content_mkinitcpio: String =
            fs::read_to_string(Path::new(MKINITCPIO_PATH)).expect("Read files content to string");
        self.system.mkinitcpio = match read_in_variable(&content_mkinitcpio, "=", "KEYMAP") {
            Some(var) => Some(var),
            None => None,
        };
    }
}

impl GetSystem for TimeDiff {
    fn get_system(&mut self) {
        let out_timedatectl: Output =
            execute_output("timedatectl show", "/").expect("Unable to execute command!");
        self.system.timezone = match read_in_variable(
            String::from_utf8(out_timedatectl.stdout).unwrap().as_str(),
            "=",
            "Timezone",
        ) {
            Some(var) => Some(var),
            None => None,
        };
    }
}

impl GetSystem for LanguageDiff {
    fn get_system(&mut self) {
        // get locale
        let content_locale_conf: String =
            fs::read_to_string(Path::new(LOCALE_CONF_PATH)).expect("Read files content to string");
        self.system.locale = match read_in_variable(content_locale_conf.as_str(), "=", "LANG") {
            Some(locale) => Some(locale),
            None => None,
        };

        // get locale + character
        let content_locale_gen: String =
            fs::read_to_string(Path::new(LOCALE_GEN_PATH)).expect("Read files content to string");

        self.system.character = match content_locale_gen
            .trim()
            .split(' ')
            .collect::<Vec<&str>>()
            .get(1)
        {
            Some(var_character) => Some(var_character.to_string()),
            None => None,
        };
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
                Err(_) => panic!("Error (expect): Failed to execute arg_get_groups"),
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
        let str_pacman = fs::read_to_string(Path::new(PACMAN_CONF_PATH))
            .expect("Error (expect): Failed to read content of pacman.conf");
        self.system.parallel = match read_in_variable(&str_pacman, " = ", "ParallelDownloads") {
            Some(parallel) => Some(
                parallel
                    .parse::<u8>()
                    .expect("Error (expect): Failed to parse String to u8"),
            ),
            None => None,
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
                        .expect("Error (Expect): Conversion from utf8 to String"),
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
                    if Path::new(&create_dir.path).is_dir() {
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
        for link in self.config.links.clone().unwrap() {
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

                if Path::new(&link.origin).is_dir() && Path::new(&destination_string).is_symlink() {
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
                    let path_string: String = format!("{}/{}", curl.path, curl.file_name);
                    let file_path: &Path = Path::new(&path_string);
                    if file_path.is_file() {
                        curl_vec.push(curl);
                    }
                }
                if self.config.curl.clone() != None {
                    if curl_vec.len() > 0 {
                        self.system.curl = Some(curl_vec);
                    }
                }
            }
            None => (),
        };

        match self.config.unzip.clone() {
            Some(config_unzip_vec) => {
                let mut zip_vec: Vec<Unzip> = Vec::new();
                for zip in config_unzip_vec {
                    let zip_path: &Path = Path::new(&zip.path);
                    if zip_path.is_dir() {
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
        let file_content_string: String = fs::read_to_string(Path::new(FAIL2BAN_JAIL_LOCAL_PATH))
            .expect("Retrieving file's content");
        let ignoreip: String = read_in_variable(&file_content_string, " = ", "ignoreip")
            .expect("Get ignoreip from jail.local");
        let bantime: usize = read_in_variable(&file_content_string, " = ", "bantime")
            .expect("Get bantime from jail.local")
            .parse::<usize>()
            .expect("Parse String to usize");
        let findtime: usize = read_in_variable(&file_content_string, " = ", "findtime")
            .expect("Get findtime from jail.local")
            .parse::<usize>()
            .expect("Parse String to usize");
        let maxretry: usize = read_in_variable(&file_content_string, " = ", "maxretry")
            .expect("Get maxretry from jail.local")
            .parse::<usize>()
            .expect("Parse String to usize");
        let mut services: Vec<String> = Vec::new();
        for line in file_content_string.lines() {
            if line.contains("[") && line.contains("]") && !line.contains("[DEFAULT]") {
                let mut service: String = line.to_string();
                service.pop();
                if service.len() > 0 {
                    service.remove(0);
                }
                services.push(service);
            }
        }

        if ignoreip != "" {
            self.system.ignoreip = Some(ignoreip);
        }
        self.system.maxretry = Some(maxretry);
        self.system.bantime = Some(bantime);
        self.system.findtime = Some(findtime);
        if services.len() > 0 {
            self.system.services = Some(services);
        }
    }
}

impl GetSystem for MonitorDiff {
    fn get_system(&mut self) {
        let file_content_string: String =
            match fs::read_to_string(Path::new(HYPR_MONITOR_CONF_PATH)) {
                Ok(val) => val,
                Err(_) => "".to_string(),
            };
        let mut monitor_string_vec: Vec<String> = Vec::new();

        let mut monitor_struct_vec: Vec<MonitorStruct> = Vec::new();

        for line in file_content_string.lines() {
            if line.contains("monitor=") {
                monitor_string_vec.push(line.split_once("=").unwrap().1.to_string());
            }
        }
        for monitor in monitor_string_vec {
            let monitor_vec: Vec<&str> = monitor.split(", ").collect();
            monitor_struct_vec.push(MonitorStruct {
                connection: monitor_vec.get(0).unwrap().to_string(),
                resolution: monitor_vec
                    .get(1)
                    .unwrap()
                    .split_once("@")
                    .unwrap()
                    .0
                    .to_string(),
                refreshrate: monitor_vec
                    .get(1)
                    .unwrap()
                    .split_once("@")
                    .unwrap()
                    .1
                    .to_string(),
                position: monitor_vec.get(2).unwrap().to_string(),
                scale: monitor_vec
                    .get(3)
                    .unwrap()
                    .parse::<f32>()
                    .expect("Conversion from String to f32"),
            })
        }

        if monitor_struct_vec.len() > 0 {
            self.system.monitors = Some(monitor_struct_vec);
        }
    }
}

impl GetSystem for FilesDiff {
    fn get_system(&mut self) {
        // FILES
        if self.config.files == None {
            // cant find files if i dont have the path to them
            self.system.files = None;
        } else {
            let mut file_vec: Vec<TextToFile> = Vec::new();
            for file in self.config.files.clone().unwrap() {
                let file_path_string: String = format!("{}/{}", file.path, file.file_name);
                let file_path: &Path = Path::new(&file_path_string);
                if file_path.is_file() {
                    let file_content: String = fs::read_to_string(file_path)
                        .expect("Able to read file's content to file!")
                        .trim()
                        .to_string();
                    if file_content == file.write {
                        file_vec.push(file);
                    }
                }
            }
            if file_vec.len() > 0 {
                self.system.files = Some(file_vec);
            }
        }
    }
}
