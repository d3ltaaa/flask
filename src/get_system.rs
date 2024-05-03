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
        // write to self
        self.system.keyboard_tty = None;

        // get mkinitcpio
        let mkinitcpio_path: &Path = Path::new(MKINITCPIO_PATH);
        let contents: String =
            fs::read_to_string(mkinitcpio_path).expect("Read files content to string");
        self.system.mkinitcpio = Some(read_in_variable(&contents, "=", "KEYMAP").unwrap());
    }
}

impl GetSystem for TimeDiff {
    fn get_system(&mut self) {
        // get timezone from timedatectl output
        let output: Output =
            execute_output("timedatectl show", "/").expect("Unable to execute command!");
        let timezone: String = read_in_variable(
            String::from_utf8(output.stdout).unwrap().as_str(),
            "=",
            "Timezone",
        )
        .expect("Retrieve variable from String");
        self.system.timezone = Some(timezone);
    }
}

impl GetSystem for LanguageDiff {
    fn get_system(&mut self) {
        // get locale
        let contents: String =
            fs::read_to_string(Path::new(LOCALE_CONF_PATH)).expect("Read files content to string");
        let locale: String =
            read_in_variable(contents.as_str(), "=", "LANG").expect("Retrieve variable from file");
        self.system.locale = Some(locale);

        // get locale + character
        let contents: String =
            fs::read_to_string(Path::new(LOCALE_GEN_PATH)).expect("Read files content to string");

        let character: String = contents
            .trim()
            .split(' ')
            .collect::<Vec<&str>>()
            .get(1)
            .expect("Get element from Vector")
            .to_string();
        self.system.character = Some(character);
    }
}

impl GetSystem for SystemDiff {
    fn get_system(&mut self) {
        let content: String = fs::read_to_string(Path::new(HOSTNAME_PATH))
            .expect("Reading from /etc/hostname succeded");
        self.system.hostname = Some(content.trim().to_string());
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

        let output: Output = execute_output("getent passwd {1000..1401}", "/")
            .expect("Able to retrieve output from getent");
        let output_string: String =
            String::from_utf8(output.stdout).expect("Converting utf8 to String");
        let mut user_groups_vec: Vec<User> = Vec::new(); // create a vector to store the users data
                                                         // type with a groups vector
        let mut user_list_vec: Vec<String> = Vec::new(); // create a vector to list the users

        for line in output_string.lines() {
            // retrieve user name from output
            let user_name: String = line
                .split(':')
                .collect::<Vec<&str>>()
                .get(0)
                .expect("Retrieve first element of Vector")
                .to_string();

            // get the groups of that user
            let argument: String = format!("groups {}", user_name);
            let group_output: Output = execute_output(&argument, "/")
                .expect("Able to retrieve output from groups command");
            let group_output_string: String =
                String::from_utf8(group_output.stdout).expect("Converting utf8 to String");
            let mut group_vec: Vec<String> = Vec::new(); // create a vector for the groups

            for group in group_output_string.trim().split(' ').collect::<Vec<&str>>() {
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
        let s = fs::read_to_string(Path::new(PACMAN_CONF_PATH)).expect("Reading files content");
        let parallel: u8 = read_in_variable(&s, " = ", "ParallelDownloads")
            .expect("Reading variable")
            .parse::<u8>()
            .expect("Parse String to u8");
        self.system.parallel = Some(parallel);
    }
}

impl GetSystem for ServicesDiff {
    fn get_system(&mut self) {
        if is_user_root() {
            // get services
            let output: Output = execute_output("systemctl list-unit-files --state=enabled", "/")
                .expect("Running systemctl command");
            let output_string: String =
                String::from_utf8(output.stdout).expect("Converting from utf8 to String");
            let mut services_vec: Vec<String> = Vec::new();

            for line in output_string.lines() {
                if line.contains("enabled disabled") && line.contains(".service") {
                    services_vec.push(
                        line.split(' ')
                            .collect::<Vec<&str>>()
                            .get(0)
                            .expect("Retrieving first element")
                            .to_string(),
                    );
                }
            }

            if services_vec.len() > 0 {
                self.system.services = Some(services_vec);
            }
        } else {
            // get user services
            let output: Output =
                execute_output("systemctl --user list-unit-files --state=enabled", "/")
                    .expect("Running systemctl command");
            let output_string: String =
                String::from_utf8(output.stdout).expect("Converting from utf8 to String");
            let mut user_services_vec: Vec<String> = Vec::new();

            for line in output_string.lines() {
                if line.contains("enabled") && line.contains(".service") {
                    user_services_vec.push(
                        line.split(' ')
                            .collect::<Vec<&str>>()
                            .get(0)
                            .expect("Retrieving first element")
                            .to_string(),
                    );
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
            let output: Output = execute_output("pacman -Qen", "/").expect("Running pacman -Qen");
            let output_string: String =
                String::from_utf8(output.stdout).expect("Converting from utf8 to String");

            let mut arch_vec: Vec<String> = Vec::new();

            for line in output_string.trim().lines() {
                arch_vec.push(
                    line.split(' ')
                        .collect::<Vec<&str>>()
                        .get(0)
                        .unwrap()
                        .to_string(),
                );
            }
            SetNoneForVecIfNeededInSystem!(self, pacman_packages, arch_vec);
        } else {
            // get aur_packages
            let output: Output = execute_output("pacman -Qem", "/").expect("Running pacman -Qem");
            let output_string: String =
                String::from_utf8(output.stdout).expect("Converting from utf8 to String");

            let mut aur_vec: Vec<String> = Vec::new();

            for line in output_string.trim().lines() {
                if !line.contains("paru") {
                    aur_vec.push(
                        line.split(' ')
                            .collect::<Vec<&str>>()
                            .get(0)
                            .unwrap()
                            .to_string(),
                    );
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
                            let argument: String = format!("ls -ld {}", reown_dir.directory);
                            let output: Output =
                                execute_output(&argument, "/").expect("ls -ld succeded");
                            let output_string: String = String::from_utf8(output.stdout)
                                .expect("Conversion from utf8 to String");
                            let owner_group: String = output_string
                                .split(' ')
                                .collect::<Vec<&str>>()
                                .get(3)
                                .expect("get(3)")
                                .to_string();
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
        //links
        let mut links_vec: Vec<Links> = Vec::new();
        for link in self.config.links.clone().unwrap() {
            let mut file_vec: Vec<String> = Vec::new();
            let argument: String = format!("ls -A {}", link.origin);
            let output: Output = execute_output(&argument, "/").expect("ls -A link.get(0)");
            let output_string: String =
                String::from_utf8(output.stdout).expect("Converting from utf8 to String");
            for line in output_string.lines() {
                file_vec.push(line.trim().to_string());
            }

            let mut all_links_are_ok: bool = true;
            for file in file_vec {
                let origin_string: String = format!("{}/{}", link.origin, file);
                let destination_string: String = format!("{}/{}", link.destination, file);

                if Path::new(&link.origin).is_dir() && Path::new(&destination_string).is_symlink() {
                    let mut argument: String = String::new();
                    if Path::new(&destination_string).is_dir() {
                        argument = format!("ls -ldA1 {}", destination_string);
                    } else if Path::new(&destination_string).is_file() {
                        argument = format!("ls -lA1 {}", destination_string);
                    }

                    let output: Output = execute_output(&argument, "/").expect(&argument);
                    let output_string: String =
                        String::from_utf8(output.stdout).expect("Conversion from utf8 to String");

                    let real_origin_string: String = output_string
                        .split("->")
                        .collect::<Vec<&str>>()
                        .last()
                        .unwrap()
                        .trim()
                        .to_string();
                    // println!("f: {}\n  o: {}\n  r: {}", destination_string, origin_string, real_origin_string);

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
        }
    }
}

impl GetSystem for GrubDiff {
    fn get_system(&mut self) {
        let s: String =
            fs::read_to_string(Path::new(GRUB_PATH)).expect("Reading files content to string");
        let mut argument_string: String = read_in_variable(&s, "=", "GRUB_CMDLINE_LINUX_DEFAULT")
            .expect("Reading variable from File");
        // remove "" from argument_string
        argument_string.pop();
        if argument_string.len() > 0 {
            argument_string.remove(0);
        }

        let mut argument_vec: Vec<String> = Vec::new();
        for var in argument_string.split(' ').collect::<Vec<&str>>() {
            argument_vec.push(var.to_string());
        }
        println!("DEBUG: {:?}", argument_vec);
        SetNoneForVecIfNeededInSystem!(self, grub_cmdline_linux_default, argument_vec);
    }
}

impl GetSystem for MkinitcpioDiff {
    fn get_system(&mut self) {
        let s: String = fs::read_to_string(Path::new(MKINITCPIO_PATH))
            .expect("Reading files content to string");

        // MODULES
        let mut argument_string: String =
            read_in_variable(&s, "=", "MODULES").expect("Reading variable from File");
        // remove () from argument_string
        argument_string.pop();
        if argument_string.len() > 0 {
            argument_string.remove(0);
        }

        let mut argument_vec: Vec<String> = Vec::new();
        for var in argument_string.split(' ').collect::<Vec<&str>>() {
            argument_vec.push(var.to_string());
        }
        SetNoneForVecIfNeededInSystem!(self, modules, argument_vec);

        // HOOKS
        let mut argument_string: String =
            read_in_variable(&s, "=", "HOOKS").expect("Reading variable from File");
        // remove () from argument_string
        argument_string.pop();
        if argument_string.len() > 0 {
            argument_string.remove(0);
        }

        let mut argument_vec: Vec<String> = Vec::new();
        for var in argument_string.split(' ').collect::<Vec<&str>>() {
            argument_vec.push(var.to_string());
        }
        SetNoneForVecIfNeededInSystem!(self, hooks, argument_vec);
    }
}

impl GetSystem for DownloadsDiff {
    fn get_system(&mut self) {
        if self.config.git.clone() != None {
            let mut git_vec: Vec<GitDownload> = Vec::new();
            for git in self.config.git.clone().unwrap() {
                let git_dir_name: String = git
                    .url
                    .split('/')
                    .collect::<Vec<&str>>()
                    .last()
                    .unwrap()
                    .split_once('.')
                    .unwrap()
                    .0
                    .to_string();
                let git_path: String = format!("{}/{}", git.path, git_dir_name);
                if Path::new(&git_path).is_dir() {
                    git_vec.push(git);
                }
            }
            if git_vec.len() > 0 {
                self.system.git = Some(git_vec);
            }
        }

        if self.config.curl.clone() != None {
            let mut curl_vec: Vec<CurlDownload> = Vec::new();
            for curl in self.config.curl.clone().unwrap() {
                let path_string: String = format!("{}/{}", curl.path, curl.file_name);
                let file_path: &Path = Path::new(&path_string);
                if file_path.is_file() {
                    curl_vec.push(curl);
                }
            }
            if curl_vec.len() > 0 {
                self.system.curl = Some(curl_vec);
            }
        }

        if self.config.unzip.clone() != None {
            let mut zip_vec: Vec<Unzip> = Vec::new();
            for zip in self.config.unzip.clone().unwrap() {
                let zip_path: &Path = Path::new(&zip.path);
                if zip_path.is_dir() {
                    zip_vec.push(zip);
                }
            }
            if zip_vec.len() > 0 {
                self.system.unzip = Some(zip_vec);
            }
        }
    }
}

impl GetSystem for UfwDiff {
    fn get_system(&mut self) {
        let output: Output =
            execute_output("sudo ufw status verbose", "/").expect("Ufw command execution");
        let output_string: String =
            String::from_utf8(output.stdout).expect("Conversion from utf8 to String");

        let default_output: String = read_in_variable(&output_string, ":", "Default")
            .expect("Reading variable")
            .to_string();

        let incoming_val: String = default_output
            .split_once("(incoming)")
            .unwrap()
            .0
            .trim()
            .split(' ')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        if incoming_val != "" {
            self.system.incoming = Some(incoming_val);
        }

        let outgoing_val: String = default_output
            .split_once("(outgoing)")
            .unwrap()
            .0
            .trim()
            .split(' ')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .to_string();
        if outgoing_val != "" {
            self.system.outgoing = Some(outgoing_val);
        }

        let rule_output: Vec<&str> = output_string.lines().collect::<Vec<&str>>();
        if rule_output.len() > 7 {
            let rule_output: String = rule_output[7..].join("\n");
            let mut rule_vec: Vec<String> = Vec::new();
            for line in rule_output.lines() {
                if !line.contains("(v6)") {
                    rule_vec.push(line.split_once(" ").unwrap().0.to_string());
                }
            }
            if rule_vec.len() > 0 {
                self.system.rules = Some(rule_vec);
            }
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
