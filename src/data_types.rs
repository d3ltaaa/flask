use crate::helper::{execute_output, is_user_root, read_in_variable};
use crate::structure::{CreateDirs, CurlDownload, Directories, Downloads, Fail2Ban, Files, GitDownload, Grub, Language, Links, Mkinitcpio, Monitor, MonitorStruct, Packages, Pacman, ReownDirs, Services, System, TextToFile, Time, Ufw, Unzip, User, Users};
use crate::structure::Keyboard;
use crate::{FAIL2BAN_JAIL_LOCAL_PATH, GRUB_PATH, HOSTNAME_PATH, HOSTS_PATH, HYPR_MONITOR_CONF_PATH, LOCALE_CONF_PATH, LOCALE_GEN_PATH, MKINITCPIO_PATH, PACMAN_CONF_PATH};
use std::fs;
use std::path::Path;
use std::process::Output;
use std::u8;

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub enum InstallOrUpdate {
//     Install,
//     Update,
// }

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Diff<T> {
    pub add: T,
    pub remove: T,
}

macro_rules! SetNoneForVecIfNeededInConfig {
    ($self: ident, $field: ident, $toml_instance: ident) => {
        $self.config.$field = $toml_instance.$field.clone();
        match $self.config.$field.clone() {
            Some(vector) => {
                if vector.len() == 0 {
                    $self.config.$field = None;
                }
            }
            None => (),
        }
    };
}

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

macro_rules! TypeDiff {
    ($name:ident, $type_name:ty) => {
        #[derive(Debug, PartialEq, Clone)]
        pub struct $name {
            pub config: $type_name,
            pub system: $type_name,
            pub diff: Diff<$type_name>,
        }
    };
}

macro_rules! CalcDiffVec {
    ($self: ident, $field: ident, $data_type: ty) => {
            if $self.config.$field == None && $self.system.$field == None {
                $self.diff.add.$field = None;
                $self.diff.remove.$field = None;
            } else if $self.config.$field == None && $self.system.$field != None {
                $self.diff.add.$field = None;
                $self.diff.remove.$field = $self.system.$field.clone();
            } else if $self.config.$field != None && $self.system.$field == None {
                $self.diff.add.$field = $self.config.$field.clone();
                $self.diff.remove.$field = None;
            } else {
                let mut add_arg_vec: $data_type = Vec::new();
                for arg in $self.config.$field.clone().unwrap() {
                    if ! $self.system.$field.clone().unwrap().contains(&arg) {
                        add_arg_vec.push(arg);
                    }
                }
                if add_arg_vec.len() > 0 {
                    $self.diff.add.$field = Some(add_arg_vec);
                }

                let mut remove_arg_vec: $data_type = Vec::new();
                for arg in $self.system.$field.clone().unwrap() {
                    if ! $self.config.$field.clone().unwrap().contains(&arg) {
                        remove_arg_vec.push(arg);
                    }
                }
                if remove_arg_vec.len() > 0 {
                    $self.diff.remove.$field = Some(remove_arg_vec);
                }
            }
    };
}

macro_rules! CalcDiffVar {
    ($self: ident, $field: ident) => {
        if $self.config.$field == $self.system.$field {
            $self.diff.add.$field = None;
            $self.diff.remove.$field = None;
        } else {
            $self.diff.add.$field = $self.config.$field.clone();
            $self.diff.remove.$field = $self.system.$field.clone();
        }
    };
}


pub trait New<T> {
    fn new() -> T;
}

pub trait Populate<T> {
    fn populate(&mut self, config: &T);
}

pub trait GetConfig<T> {
    fn get_config(&mut self, config: &T);
}

pub trait GetSystem {
    fn get_system(&mut self);
}

pub trait GetDiff {
    fn get_diff(&mut self);
}

TypeDiff!(KeyboardDiff, Keyboard);

impl New<KeyboardDiff> for KeyboardDiff {
    fn new() -> KeyboardDiff {
        KeyboardDiff {
            config: Keyboard {
                mkinitcpio: None,
                keyboard_tty: None,
            },
            system: Keyboard {
                mkinitcpio: None,
                keyboard_tty: None,
            },
            diff: Diff {
                add: Keyboard {
                    mkinitcpio: None,
                    keyboard_tty: None,
                },
                remove: Keyboard {
                    mkinitcpio: None,
                    keyboard_tty: None,
                },
            },
        }
    }
}

impl Populate<Keyboard> for KeyboardDiff {
    fn populate(&mut self, config: &Keyboard) {
        if is_user_root() {
            self.get_config(config);
            self.get_system();
            self.get_diff();
        }
    }
}

impl GetConfig<Keyboard> for KeyboardDiff {
    fn get_config(&mut self, config: &Keyboard) {
        // get keyboard_tty
        self.config.keyboard_tty = config.keyboard_tty.to_owned();

        match self.config.keyboard_tty.clone() {
            Some(keyboard_tty) => {
                if keyboard_tty == "" {
                    panic!("Keyboard_tty must be specified!");
                }
            }
            None => panic!("Keyboard_tty must be specified!"),
        }

        // get mkinitcpio
        self.config.mkinitcpio = config.mkinitcpio.to_owned();

        match self.config.mkinitcpio.clone() {
            Some(mkinitcpio) => {
                if mkinitcpio == "" {
                    panic!("Mkinitcpio must be specified!");
                }
            }
            None => panic!("Mkinitcpio must be specified!"),
        }
    }
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

impl GetDiff for KeyboardDiff {
    fn get_diff(&mut self) {
        CalcDiffVar!(self, mkinitcpio);

        // cant be build therefore it is just none
        self.diff.add.keyboard_tty = None;
        self.diff.remove.keyboard_tty = None;
    }
}

TypeDiff!(TimeDiff, Time);

impl New<TimeDiff> for TimeDiff {
    fn new() -> TimeDiff {
        TimeDiff {
            config: Time { timezone: None },
            system: Time { timezone: None },
            diff: Diff {
                add: Time { timezone: None },
                remove: Time { timezone: None },
            },
        }
    }
}

impl Populate<Time> for TimeDiff {
    fn populate(&mut self, time: &Time) {
        if is_user_root() {
            self.get_config(time);
            self.get_system();
            self.get_diff();
        }
    }
}

impl GetConfig<Time> for TimeDiff {
    fn get_config(&mut self, time: &Time) {
        // get time zone
        self.config.timezone = time.timezone.clone();
        match self.config.timezone.clone() {
            Some(timezone) => {
                if timezone == "" {
                    panic!("Timezone must be specified!");
                }
            }
            None => {
                panic!("Timezone must be specified!");
            }
        }
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

impl GetDiff for TimeDiff {
    fn get_diff(&mut self) {
        if self.config.timezone == self.system.timezone {
            self.diff.add.timezone = None;
            self.diff.remove.timezone = None;
        } else if self.config.timezone != self.system.timezone {
            self.diff.add.timezone = self.config.timezone.clone();
            self.diff.remove.timezone = self.system.timezone.clone();
        }
    }
}

TypeDiff!(LanguageDiff, Language);

impl New<LanguageDiff> for LanguageDiff {
    fn new() -> LanguageDiff {
        LanguageDiff {
            config: Language {
                locale: None,
                character: None,
            },
            system: Language {
                locale: None,
                character: None,
            },
            diff: Diff {
                add: Language {
                    locale: None,
                    character: None,
                },
                remove: Language {
                    locale: None,
                    character: None,
                },
            },
        }
    }
}

impl Populate<Language> for LanguageDiff {
    fn populate(&mut self, language: &Language) {
        if is_user_root() {
            self.get_config(language);
            self.get_system();
            self.get_diff();
            self.check();
        }
    }
}

impl GetConfig<Language> for LanguageDiff {
    fn get_config(&mut self, language: &Language) {
        self.config.locale = language.locale.clone();
        self.config.character = language.character.clone();

        match self.config.locale.clone() {
            Some(locale) => {
                if locale == "" {
                    panic!("Locale must be specified!");
                }
            },
            None => panic!("Locale must be specified!")
        }

        match self.config.character.clone() {
            Some(character) => {
                if character == "" {
                    panic!("Locale must be specified!");
                }
            },
            None => panic!("Locale must be specified!")
        }
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
            .split(' ')
            .collect::<Vec<&str>>()
            .get(1)
            .expect("Get element from Vector")
            .to_string();
        self.system.character = Some(character);
    }
}

impl GetDiff for LanguageDiff {
    fn get_diff(&mut self) {
        CalcDiffVar!(self, locale);
        CalcDiffVar!(self, character);
    }
}

impl LanguageDiff {
    fn check(&self) {
        let contents: String =
            fs::read_to_string(Path::new(LOCALE_GEN_PATH)).expect("Read files content to string");

        // check if locales are the same
        let locale: String = contents
            .split(' ')
            .collect::<Vec<&str>>()
            .get(0)
            .expect("Get element from Vector")
            .to_string();

        if Some(locale) != self.system.locale {
            println!(
                "Error: Locales are not the same in {} {}",
                LOCALE_GEN_PATH, LOCALE_CONF_PATH
            );
        }
    }
}

TypeDiff!(SystemDiff, System);

impl New<SystemDiff> for SystemDiff {
    fn new() -> SystemDiff {
        SystemDiff {
            config: System { hostname: None },
            system: System { hostname: None },
            diff: Diff {
                add: System { hostname: None },
                remove: System { hostname: None },
            },
        }
    }
}

impl Populate<System> for SystemDiff {
    fn populate(&mut self, system: &System) {
        if is_user_root() {
            self.get_config(system);
            self.get_system();
            self.get_diff();
            self.check();
        }
    }
}

impl GetConfig<System> for SystemDiff {
    fn get_config(&mut self, system: &System) {
        self.config.hostname = system.hostname.clone();
        match self.config.hostname.clone() {
            Some(hostname) => {
                if hostname.contains(" ") {
                    panic!("Hostname is not allowed to contain ' '!");
                } else if hostname == "" {
                    panic!("Hostname must be specified!");
                }
            },
            None => panic!("Hostname must be specified!"),
        }
    }
}

impl GetSystem for SystemDiff {
    fn get_system(&mut self) {
        let content: String = fs::read_to_string(Path::new(HOSTNAME_PATH))
            .expect("Reading from /etc/hostname succeded");
        self.system.hostname = Some(content.trim().to_string());
    }
}

impl GetDiff for SystemDiff {
    fn get_diff(&mut self) {
        if self.config.hostname != self.system.hostname {
            self.diff.add.hostname = self.config.hostname.clone();
            self.diff.remove.hostname = self.system.hostname.clone();
        } else {
            self.diff.add.hostname = None;
            self.diff.remove.hostname = None;
        }
    }
}

impl SystemDiff {
    fn check(&self) {
        let supposed: String = format!(
            "127.0.0.1       localhost\n::1             localhost\n127.0.1.1       {}.localdomain {}", 
            self.config.hostname.clone().unwrap(), 
            self.config.hostname.clone().unwrap()
            );

        let real: String = fs::read_to_string(Path::new(HOSTS_PATH)).expect("Reading from Hostpath succeded").trim().to_string();

        if real != supposed {
            println!(
                "Error: Hostnames are not the same in {} {}",
                HOSTNAME_PATH, HOSTS_PATH
            );
        }
    }
}

TypeDiff!(UserDiff, Users);

impl New<UserDiff> for UserDiff {
    fn new() -> UserDiff {
        UserDiff {
            config: Users {
                user_list: None,
                user_groups: None
            },
            system: Users {
                user_list: None,
                user_groups: None
            },
            diff: Diff {
                add: Users {
                    user_list: None,
                    user_groups: None
                },
                remove: Users {
                    user_list: None,
                    user_groups: None
                },
            },
        }
    }
}

impl Populate<Users> for UserDiff {
    fn populate(&mut self, users: &Users) {
        if is_user_root() {
            self.get_config(users);
            self.get_system();
            self.get_diff();
        }
    }
}

impl GetConfig<Users> for UserDiff {
    fn get_config(&mut self, users: &Users) {
        self.config.user_list = users.user_list.clone();
        self.config.user_groups = users.user_groups.clone();
        match self.config.user_list.clone() {
            Some(user_list_vec) => {
                if user_list_vec.len() == 0 {
                    panic!("No user specified in config!"); 
                } else {
                    match self.config.user_groups.clone() {
                        Some(user_groups_vec) => {
                            if user_groups_vec.len() != user_list_vec.len() {
                                panic!("User_list and User_groups do not have the same length! Some groups are not specified for User(s)!")
                            }
                        }
                        None => panic!("No user groups specified at all!"),
                    }
                }
            },
            None => (),
        }
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
        
        let output: Output = execute_output("getent passwd {1000..1401}", "/").expect("Able to retrieve output from getent");
        let output_string: String = String::from_utf8(output.stdout).expect("Converting utf8 to String");
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
            let group_output: Output = execute_output(&argument, "/").expect("Able to retrieve output from groups command");
            let group_output_string: String = String::from_utf8(group_output.stdout).expect("Converting utf8 to String");
            let mut group_vec: Vec<String> = Vec::new(); // create a vector for the groups

            for group in group_output_string.trim().split(' ').collect::<Vec<&str>>() {
                if group != user_name {
                    group_vec.push(group.to_string()); // add to group string
                }
            }

            user_groups_vec.push(User {name: user_name.clone(), groups: group_vec}); // add users to user vector
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

impl GetDiff for UserDiff {
    fn get_diff(&mut self) {
        // diff names
        // rule out Nones in any of the following variables
        if self.config.user_list == None && self.system.user_list == None {
            self.diff.add.user_list = None;
            self.diff.remove.user_list = None;
        }
        else if self.config.user_list != None && self.system.user_list == None {
            self.diff.add.user_list = self.config.user_list.clone();
            self.diff.remove.user_list = None;
        }
        else if self.config.user_list == None && self.system.user_list != None {
            self.diff.add.user_list = None;
            self.diff.remove.user_list = self.system.user_list.clone();
        }
        else {
            // user_list: add
            let mut add_user_list_vec: Vec<String> = Vec::new();
            for user in self.config.user_list.clone().unwrap() {
                if ! self.system.user_list.clone().unwrap().contains(&user) {
                    add_user_list_vec.push(user);
                }
            }
            if add_user_list_vec.len() == 0 {
                self.diff.add.user_list = None;
            } else {
                self.diff.add.user_list = Some(add_user_list_vec);
            }

            // user_list: remove
            let mut remove_user_list_vec: Vec<String> = Vec::new();
            for user in self.system.user_list.clone().unwrap() {
                if ! self.config.user_list.clone().unwrap().contains(&user) {
                    remove_user_list_vec.push(user);
                }
            }
            if remove_user_list_vec.len() == 0 {
                self.diff.remove.user_list = None;
            } else {
                self.diff.remove.user_list = Some(remove_user_list_vec);
            }

        }

        if self.config.user_groups == None && self.system.user_groups == None {
            self.diff.add.user_groups = None;
            self.diff.remove.user_groups = None;
        } else if self.config.user_groups == None && self.system.user_groups != None {
            self.diff.add.user_groups = None;
            self.diff.remove.user_groups = self.system.user_groups.clone();
        } else if self.config.user_groups != None && self.system.user_groups == None {
            self.diff.add.user_groups = self.config.user_groups.clone();
            self.diff.remove.user_groups = None;
        } else {
            // user_groups: add
            let mut add_groups_vec: Vec<User> = Vec::new();
            // go though the users in config
            for config_user_counter in 0..self.config.user_groups.clone().unwrap().len() {
                let mut config_user_in_system: bool = false;
                // go through the users in system
                for system_user_counter in 0..self.system.user_groups.clone().unwrap().len() {
                    let config_user: User = self.config.user_groups.clone().unwrap().get(config_user_counter).unwrap().to_owned();
                    let system_user: User = self.system.user_groups.clone().unwrap().get(system_user_counter).unwrap().to_owned();
                    if config_user.name == system_user.name {
                        // user is in config and in system
                        config_user_in_system = true;
                        let mut groups_vec: Vec<String> = Vec::new();
                        // go through the config's user's groups and look if they are in system as
                        // well
                        for group in config_user.groups.clone() {
                            if ! system_user.groups.contains(&group) {
                                // if they are not found in the system version, add them to the
                                // groups_vec
                                groups_vec.push(group);
                            }
                        }
                        if groups_vec.len() > 0 {
                            add_groups_vec.push(User { name: config_user.name, groups: groups_vec });
                        }
                    }
                }

                // if the entire user is not found in system, than add him
                if ! config_user_in_system {
                    add_groups_vec.push(self.config.user_groups.clone().unwrap().get(config_user_counter).unwrap().to_owned());
                }
            }
            if add_groups_vec.len() > 0 {
                self.diff.add.user_groups = Some(add_groups_vec);
            }

            // user_groups: remove
            let mut remove_groups_vec: Vec<User> = Vec::new();
            // go through the users in system
            for system_user_counter in 0..self.system.user_groups.clone().unwrap().len() {
                // let mut system_user_in_config: bool = false;
                // go through the users in config
                for config_user_counter in 0..self.config.user_groups.clone().unwrap().len() {
                    let config_user: User = self.config.user_groups.clone().unwrap().get(config_user_counter).unwrap().to_owned();
                    let system_user: User = self.system.user_groups.clone().unwrap().get(system_user_counter).unwrap().to_owned();
                    if system_user.name == config_user.name {
                        // user is in config and in system
                        // system_user_in_config = true;
                        let mut groups_vec: Vec<String> = Vec::new();
                        // go through the system's user's groups and look if they are in config as
                        // well
                        for group in system_user.groups.clone() {
                            if ! config_user.groups.contains(&group) {
                                // if they are not in the config version, add them to the groups
                                // vec to be removed later
                                groups_vec.push(group);
                            }
                        }

                        if groups_vec.len() > 0 {
                            remove_groups_vec.push(User { name: system_user.name, groups: groups_vec });
                        }
                    }
                }

                // if the entire user is not found in config, than remove him (he should already be
                // removed by diff.remove.user_list)
                // if ! system_user_in_config {
                //     remove_groups_vec.push(self.system.user_groups.clone().unwrap().get(system_user_counter).unwrap().to_owned());
                // }
            }
            if remove_groups_vec.len() > 0 {
                self.diff.remove.user_groups = Some(remove_groups_vec);
            }
        }
    }
}

TypeDiff!(PacmanDiff, Pacman);

impl New<PacmanDiff> for PacmanDiff {
    fn new() -> PacmanDiff {
        PacmanDiff {
            config: Pacman {
                parallel: None
            },
            system: Pacman {
                parallel: None
            },
            diff: Diff {
                add: Pacman {
                    parallel: None
                },
                remove: Pacman {
                    parallel: None
                },
            }
        }
    }
}

impl Populate<Pacman> for PacmanDiff {
    fn populate(&mut self, pacman: &Pacman) {
        if is_user_root() {
            self.get_config(pacman);
            self.get_system();
            self.get_diff();
        }
    }
}

impl GetConfig<Pacman> for PacmanDiff {
    fn get_config(&mut self, pacman: &Pacman) {
        self.config.parallel = pacman.parallel;
    }
}

impl GetSystem for PacmanDiff {
    fn get_system(&mut self) {
        let s = fs::read_to_string(Path::new(PACMAN_CONF_PATH)).expect("Reading files content");
        let parallel: u8 = read_in_variable(&s, " = ", "ParallelDownloads").expect("Reading variable").parse::<u8>().expect("Parse String to u8");
        self.system.parallel = Some(parallel);
    }
}

impl GetDiff for PacmanDiff {
    fn get_diff(&mut self) {
        CalcDiffVar!(self, parallel);
    }
}

TypeDiff!(ServicesDiff, Services);

impl New<ServicesDiff> for ServicesDiff {
    fn new() -> ServicesDiff {
        ServicesDiff {
            config: Services {
                user_services: None,
                services: None
            },
            system: Services {
                user_services: None,
                services: None
            },
            diff: Diff {
                add: Services {
                    user_services: None,
                    services: None
                },
                remove: Services {
                    user_services: None,
                    services: None
                }
            }
        }
    }
}

impl Populate<Services> for ServicesDiff {
    fn populate(&mut self, services: &Services) {
        self.get_config(services);
        self.get_system();
        self.get_diff();
    }
}

impl GetConfig<Services> for ServicesDiff {
    fn get_config(&mut self, services: &Services) {
        if is_user_root() {
            self.config.services = services.services.clone();
        } else {
            self.config.user_services = services.user_services.clone();
        }
    }
}

impl GetSystem for ServicesDiff {
    fn get_system(&mut self) {
        if is_user_root() {
            // get services
            let output: Output = execute_output("systemctl list-unit-files --state=enabled", "/").expect("Running systemctl command");
            let output_string: String = String::from_utf8(output.stdout).expect("Converting from utf8 to String");
            let mut services_vec: Vec<String> = Vec::new();
    
            for line in output_string.lines() {
                if line.contains("enabled disabled") && line.contains(".service") {
                    services_vec.push(line.split(' ').collect::<Vec<&str>>().get(0).expect("Retrieving first element").to_string());
                }
            }
    
            if services_vec.len() > 0 {
                self.system.services = Some(services_vec);
            }
        } else {
            // get user services
            let output: Output = execute_output("systemctl --user list-unit-files --state=enabled", "/").expect("Running systemctl command");
            let output_string: String = String::from_utf8(output.stdout).expect("Converting from utf8 to String");
            let mut user_services_vec: Vec<String> = Vec::new();
    
            for line in output_string.lines() {
                if line.contains("enabled") && line.contains(".service") {
                    user_services_vec.push(line.split(' ').collect::<Vec<&str>>().get(0).expect("Retrieving first element").to_string());
                }
            }
    
            if user_services_vec.len() > 0 {
                self.system.user_services= Some(user_services_vec);
            }
        }
    }
}

impl GetDiff for ServicesDiff {
    fn get_diff(&mut self) {
        // services
        CalcDiffVec!(self, services, Vec<String>);

        // user_services
        CalcDiffVec!(self, user_services, Vec<String>);
    }
}

TypeDiff!(PackagesDiff, Packages);

impl New<PackagesDiff> for PackagesDiff {
    fn new() -> PackagesDiff {
        PackagesDiff {
            config: Packages {
                pacman_packages: None,
                aur_packages: None,
                manual_install_packages: None,
                build_packages: None,
            },
            system: Packages {
                pacman_packages: None,
                aur_packages: None,
                manual_install_packages: None,
                build_packages: None,
            },
            diff: Diff {
                add: Packages {
                    pacman_packages: None,
                    aur_packages: None,
                    manual_install_packages: None,
                    build_packages: None,
                },
                remove: Packages {
                    pacman_packages: None,
                    aur_packages: None,
                    manual_install_packages: None,
                    build_packages: None,
                }
            }
        }
    }
}

impl Populate<Packages> for PackagesDiff {
    fn populate(&mut self, packages: &Packages) {
        self.get_config(packages);
        self.get_system();
        self.get_diff();
    }
}

impl GetConfig<Packages> for PackagesDiff {
    fn get_config(&mut self, packages: &Packages) {
        if is_user_root() {
            SetNoneForVecIfNeededInConfig!(self, pacman_packages, packages);
            SetNoneForVecIfNeededInConfig!(self, manual_install_packages, packages);
            SetNoneForVecIfNeededInConfig!(self, build_packages, packages);
        } else {
            SetNoneForVecIfNeededInConfig!(self, aur_packages, packages);
        }
    }
}

impl GetSystem for PackagesDiff {
    fn get_system(&mut self) {
        if is_user_root() {
            // get pacman_packages
            let output: Output = execute_output("pacman -Qen", "/").expect("Running pacman -Qen");
            let output_string: String = String::from_utf8(output.stdout).expect("Converting from utf8 to String");
    
            let mut arch_vec: Vec<String> = Vec::new();
    
            for line in output_string.trim().lines() {
                arch_vec.push(line.split(' ').collect::<Vec<&str>>().get(0).unwrap().to_string());
            }
            SetNoneForVecIfNeededInSystem!(self, pacman_packages, arch_vec);
        } else {
            // get aur_packages 
            let output: Output = execute_output("pacman -Qem", "/").expect("Running pacman -Qem");
            let output_string: String = String::from_utf8(output.stdout).expect("Converting from utf8 to String");
    
            let mut aur_vec: Vec<String> = Vec::new();
    
            for line in output_string.trim().lines() {
                if ! line.contains("paru") {
                    aur_vec.push(line.split(' ').collect::<Vec<&str>>().get(0).unwrap().to_string());
                }
            }
            SetNoneForVecIfNeededInSystem!(self, aur_packages, aur_vec);
        }
    }
}
impl GetDiff for PackagesDiff {
    fn get_diff(&mut self) {
        // pacman packages
        CalcDiffVec!(self, pacman_packages, Vec<String>);
        // aur packages
        CalcDiffVec!(self, aur_packages, Vec<String>);
    }
}

TypeDiff!(DirectoriesDiff, Directories);

impl New<DirectoriesDiff> for DirectoriesDiff {
    fn new() -> DirectoriesDiff {
        DirectoriesDiff {
            config: Directories {
                reown_dirs: None,
                links: None,
                create_dirs: None,
            },
            system: Directories {
                reown_dirs: None,
                links: None,
                create_dirs: None,
            },
            diff: Diff {
                add: Directories {
                    reown_dirs: None,
                    links: None,
                    create_dirs: None,
                },
                remove: Directories {
                    reown_dirs: None,
                    links: None,
                    create_dirs: None,
                }
            }
        }
    }
}

impl Populate<Directories> for DirectoriesDiff {
    fn populate(&mut self, directories: &Directories) {
        self.get_config(directories);
        self.get_system();
        self.get_diff();
    }
}

impl GetConfig<Directories> for DirectoriesDiff {
    fn get_config(&mut self, directories: &Directories) {
        if is_user_root() {
            SetNoneForVecIfNeededInConfig!(self, reown_dirs, directories);
        }
        SetNoneForVecIfNeededInConfig!(self, create_dirs, directories);
        SetNoneForVecIfNeededInConfig!(self, links, directories);
    }
}

impl GetSystem for DirectoriesDiff {
    fn get_system(&mut self) {
        if is_user_root() {
            // since we cannot check all dirs we can only check wether or not every dir and link that
            // should be there is actuall there, therefore only the add part makes sense
            // reown_dirs
            let mut reown_dirs_vec: Vec<ReownDirs> = Vec::new();
            for reown_dir in self.config.reown_dirs.clone().unwrap() {
                if Path::new(&reown_dir.directory).is_dir() {
                    let argument: String = format!("ls -ld {}", reown_dir.directory);
                    let output: Output = execute_output(&argument, "/").expect("ls -ld succeded");
                    let output_string: String = String::from_utf8(output.stdout).expect("Conversion from utf8 to String");
                    let owner_group: String = output_string.split(' ').collect::<Vec<&str>>().get(3).expect("get(3)").to_string();
                    if owner_group == reown_dir.group {
                        reown_dirs_vec.push(reown_dir);
                    }
                }
            }
            if reown_dirs_vec.len() > 0 {
                self.system.reown_dirs = Some(reown_dirs_vec);
            }
        }

        // create_dirs
        let mut create_dirs_vec: Vec<CreateDirs> = Vec::new();
        for create_dir in self.config.create_dirs.clone().unwrap() {
            if Path::new(&create_dir.path).is_dir() {
                create_dirs_vec.push(create_dir);
            }
        }
        if create_dirs_vec.len() > 0 {
            self.system.create_dirs = Some(create_dirs_vec);
        }
    
        //links
        let mut links_vec: Vec<Links> = Vec::new();
        for link in self.config.links.clone().unwrap() {
            let mut file_vec: Vec<String> = Vec::new();
            let argument: String = format!("ls -A {}", link.origin);
            let output: Output = execute_output(&argument, "/").expect("ls -A link.get(0)");
            let output_string: String = String::from_utf8(output.stdout).expect("Converting from utf8 to String");
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
                    let output_string: String = String::from_utf8(output.stdout).expect("Conversion from utf8 to String");
    
                    let real_origin_string: String = output_string.split("->").collect::<Vec<&str>>().last().unwrap().trim().to_string();
                    // println!("f: {}\n  o: {}\n  r: {}", destination_string, origin_string, real_origin_string);

                    if real_origin_string != origin_string {
                        all_links_are_ok = false;
                    }
                }
                else {
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

impl GetDiff for DirectoriesDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, reown_dirs, Vec<ReownDirs>);

        CalcDiffVec!(self, create_dirs, Vec<CreateDirs>);

        CalcDiffVec!(self, links, Vec<Links>);
    }
}

TypeDiff!(GrubDiff, Grub);

impl New<GrubDiff> for GrubDiff {
    fn new() -> GrubDiff {
        GrubDiff {
             config: Grub {
                 grub_cmdline_linux_default: None,
             },
             system: Grub {
                 grub_cmdline_linux_default: None,
             },
             diff: Diff {
                 add: Grub {
                     grub_cmdline_linux_default: None,
                 },
                 remove: Grub {
                     grub_cmdline_linux_default: None,
                 }
             }
        }
    }
}

impl Populate<Grub> for GrubDiff {
    fn populate(&mut self, grub: &Grub) {
        if is_user_root() {
            self.get_config(grub);
            self.get_system();
            self.get_diff();
        }
    }
}

impl GetConfig<Grub> for GrubDiff {
    fn get_config(&mut self, grub: &Grub) {
        SetNoneForVecIfNeededInConfig!(self, grub_cmdline_linux_default, grub);
    }
}

impl GetSystem for GrubDiff {
    fn get_system(&mut self) {
        let s: String = fs::read_to_string(Path::new(GRUB_PATH)).expect("Reading files content to string");
        let mut argument_string: String = read_in_variable(&s, "=", "GRUB_CMDLINE_LINUX_DEFAULT").expect("Reading variable from File");
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

impl GetDiff for GrubDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, grub_cmdline_linux_default, Vec<String>);
    }

}

TypeDiff!(MkinitcpioDiff, Mkinitcpio);

impl New<MkinitcpioDiff> for MkinitcpioDiff {
    fn new() -> MkinitcpioDiff {
        MkinitcpioDiff {
            config: Mkinitcpio {
                modules: None,
                hooks: None,
            },
            system: Mkinitcpio {
                modules: None,
                hooks: None,
            },
            diff: Diff {
                add: Mkinitcpio {
                    modules: None,
                    hooks: None,
                },
                remove: Mkinitcpio {
                    modules: None,
                    hooks: None,
                }
            }
        }
    }
}

impl Populate<Mkinitcpio> for MkinitcpioDiff {
    fn populate(&mut self, mkinitcpio: &Mkinitcpio) {
        if is_user_root() {
            self.get_config(mkinitcpio);
            self.get_system();
            self.get_diff();
        }
    }
}

impl GetConfig<Mkinitcpio> for MkinitcpioDiff {
    fn get_config(&mut self, mkinitcpio: &Mkinitcpio) {
        SetNoneForVecIfNeededInConfig!(self, modules, mkinitcpio);
        SetNoneForVecIfNeededInConfig!(self, hooks, mkinitcpio);
    }
}


impl GetSystem for MkinitcpioDiff {
    fn get_system(&mut self) {
        let s: String = fs::read_to_string(Path::new(MKINITCPIO_PATH)).expect("Reading files content to string");

        // MODULES
        let mut argument_string: String = read_in_variable(&s, "=", "MODULES"). expect("Reading variable from File");
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
        let mut argument_string: String = read_in_variable(&s, "=", "HOOKS"). expect("Reading variable from File");
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

impl GetDiff for MkinitcpioDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, modules, Vec<String>);
        CalcDiffVec!(self, hooks, Vec<String>);
    }
}

TypeDiff!(DownloadsDiff, Downloads); 

impl New<DownloadsDiff> for DownloadsDiff {
    fn new() -> DownloadsDiff {
        DownloadsDiff {
            config: Downloads {
                git: None,
                curl: None,
                unzip: None,
            },
            system: Downloads {
                git: None,
                curl: None,
                unzip: None,
            },
            diff: Diff {
                add: Downloads {
                    git: None,
                    curl: None,
                    unzip: None,
                },
                remove: Downloads {
                    git: None,
                    curl: None,
                    unzip: None,
                }
            }
        }
    }
}

impl Populate<Downloads> for DownloadsDiff {
    fn populate(&mut self, downloads: &Downloads) {
        self.get_config(downloads);
        self.get_system();
        self.get_diff();
    }
}

impl GetConfig<Downloads> for DownloadsDiff {
    fn get_config(&mut self, downloads: &Downloads) {
        SetNoneForVecIfNeededInConfig!(self, curl, downloads);
        SetNoneForVecIfNeededInConfig!(self, git, downloads);
        SetNoneForVecIfNeededInConfig!(self, unzip, downloads);
    }
}

impl GetSystem for DownloadsDiff {
    fn get_system(&mut self) {

        if self.config.git.clone() != None {
            let mut git_vec: Vec<GitDownload> = Vec::new();
            for git in self.config.git.clone().unwrap() {
                let git_dir_name: String = git.url.split('/').collect::<Vec<&str>>().last().unwrap().split_once('.').unwrap().0.to_string();
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

impl GetDiff for DownloadsDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, git, Vec<GitDownload>);
        CalcDiffVec!(self, curl, Vec<CurlDownload>);
        CalcDiffVec!(self, unzip, Vec<Unzip>);
    }
}


TypeDiff!(UfwDiff, Ufw);

impl New<UfwDiff> for UfwDiff {
    fn new() -> UfwDiff {
        UfwDiff {
            config: Ufw {
                incoming: None,
                outgoing: None,
                rules: None,
            }, 
            system: Ufw {
                incoming: None,
                outgoing: None,
                rules: None,
            },
            diff: Diff {
                add: Ufw {
                    incoming: None,
                    outgoing: None,
                    rules: None,
                },
                remove: Ufw {
                    incoming: None,
                    outgoing: None,
                    rules: None,
                }
            }
        }
    }
}

impl Populate<Ufw> for UfwDiff {
    fn populate(&mut self, ufw: &Ufw) {
        if is_user_root() {
            self.get_config(ufw);
            self.get_system();
            self.get_diff();
        }
    }
}

impl GetConfig<Ufw> for UfwDiff {
    fn get_config(&mut self, ufw: &Ufw) {
        self.config.incoming= ufw.incoming.clone();
        self.config.outgoing= ufw.outgoing.clone();
        self.config.rules = ufw.rules.clone();
    }
}

impl GetSystem for UfwDiff {
    fn get_system(&mut self) {
        let output: Output = execute_output("sudo ufw status verbose", "/").expect("Ufw command execution");
        let output_string: String = String::from_utf8(output.stdout).expect("Conversion from utf8 to String");

        let default_output: String = read_in_variable(&output_string, ":", "Default").expect("Reading variable").to_string();

        let incoming_val: String = default_output.split_once("(incoming)").unwrap().0.trim().split(' ').collect::<Vec<&str>>().last().unwrap().to_string();
        if incoming_val != "" {
            self.system.incoming = Some(incoming_val);
        }

        let outgoing_val: String = default_output.split_once("(outgoing)").unwrap().0.trim().split(' ').collect::<Vec<&str>>().last().unwrap().to_string();
        if outgoing_val != "" {
            self.system.outgoing = Some(outgoing_val);
        }

        let rule_output: Vec<&str> = output_string.lines().collect::<Vec<&str>>();
        if rule_output.len() > 7 {
            let rule_output: String = rule_output[7..].join("\n");
            let mut rule_vec: Vec<String> = Vec::new();
            for line in rule_output.lines() {
                if ! line.contains("(v6)") {
                    rule_vec.push(line.split_once(" ").unwrap().0.to_string());
                }
            }
            if rule_vec.len() > 0 {
                self.system.rules = Some(rule_vec);
            }
        }
    }
}

impl UfwDiff {
    fn get_diff(&mut self) {
        CalcDiffVar!(self, incoming);
        CalcDiffVar!(self, outgoing);
        CalcDiffVec!(self, rules, Vec<String>);
    }
}

TypeDiff!(Fail2BanDiff, Fail2Ban);

impl New<Fail2BanDiff> for Fail2BanDiff {
    fn new() -> Fail2BanDiff {
        Fail2BanDiff {
            config: Fail2Ban {
                ignoreip: None,
                bantime: None,
                findtime: None,
                maxretry: None,
                services: None,
            },
            system: Fail2Ban {
                ignoreip: None,
                bantime: None,
                findtime: None,
                maxretry: None,
                services: None,
            },
            diff: Diff {
                add: Fail2Ban {
                    ignoreip: None,
                    bantime: None,
                    findtime: None,
                    maxretry: None,
                    services: None,
                },
                remove: Fail2Ban {
                    ignoreip: None,
                    bantime: None,
                    findtime: None,
                    maxretry: None,
                    services: None,
                }
            }
        }
    }
}

impl Populate<Fail2Ban> for Fail2BanDiff {
    fn populate(&mut self, fail2ban: &Fail2Ban) {
        if is_user_root() {
            self.get_config(fail2ban);
            self.get_system();
            self.get_diff();
        }
    }
}

impl GetConfig<Fail2Ban> for Fail2BanDiff {
    fn get_config(&mut self, fail2ban: &Fail2Ban) {
        self.config.ignoreip = fail2ban.ignoreip.clone();
        self.config.bantime = fail2ban.bantime;
        self.config.findtime = fail2ban.findtime;
        self.config.maxretry = fail2ban.maxretry;
        self.config.services = fail2ban.services.clone();
    }
}

impl GetSystem for Fail2BanDiff {
    fn get_system(&mut self) {
        let file_content_string: String = fs::read_to_string(Path::new(FAIL2BAN_JAIL_LOCAL_PATH)).expect("Retrieving file's content");
        let ignoreip: String = read_in_variable(&file_content_string, " = ", "ignoreip").expect("Get ignoreip from jail.local");
        let bantime: usize = read_in_variable(&file_content_string, " = ", "bantime").expect("Get bantime from jail.local").parse::<usize>().expect("Parse String to usize");
        let findtime: usize = read_in_variable(&file_content_string, " = ", "findtime").expect("Get findtime from jail.local").parse::<usize>().expect("Parse String to usize");
        let maxretry: usize = read_in_variable(&file_content_string, " = ", "maxretry").expect("Get maxretry from jail.local").parse::<usize>().expect("Parse String to usize");
        let mut services: Vec<String> = Vec::new();
        for line in file_content_string.lines() {
            if line.contains("[") && line.contains("]") && ! line.contains("[DEFAULT]") {
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

impl Fail2BanDiff {
    fn get_diff(&mut self) {
        CalcDiffVar!(self, ignoreip);
        CalcDiffVar!(self, maxretry);
        CalcDiffVar!(self, findtime);
        CalcDiffVar!(self, bantime);
        CalcDiffVec!(self, services, Vec<String>);
    }
}

TypeDiff!(MonitorDiff, Monitor);

impl New<MonitorDiff> for MonitorDiff {
    fn new() -> MonitorDiff {
        MonitorDiff {
            config: Monitor {
                monitors: None
            },
            system: Monitor {
                monitors: None
            },
            diff: Diff {
                add: Monitor {
                    monitors: None
                },
                remove: Monitor {
                    monitors: None
                }
            }
        }
    }
}

impl Populate<Monitor> for MonitorDiff {
    fn populate(&mut self, monitor: &Monitor) {
        if !is_user_root() {
            self.get_config(monitor);
            self.get_system();
            self.get_diff();
        }
    }
}

impl GetConfig<Monitor> for MonitorDiff {
    fn get_config(&mut self, monitor: &Monitor) {
        self.config.monitors = monitor.monitors.clone();
    }
}

impl GetSystem for MonitorDiff {
    fn get_system(&mut self) {
        let file_content_string: String = match fs::read_to_string(Path::new(HYPR_MONITOR_CONF_PATH)) {
            Ok(val) => val,
            Err(_) => {
                "".to_string()
            },
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
            monitor_struct_vec.push(
                MonitorStruct {
                    connection: monitor_vec.get(0).unwrap().to_string(),
                    resolution: monitor_vec.get(1).unwrap().split_once("@").unwrap().0.to_string(),
                    refreshrate: monitor_vec.get(1).unwrap().split_once("@").unwrap().1.to_string(),
                    position: monitor_vec.get(2).unwrap().to_string(),
                    scale: monitor_vec.get(3).unwrap().parse::<f32>().expect("Conversion from String to f32"),
                }
                )
        }

        if monitor_struct_vec.len() > 0 {
            self.system.monitors = Some(monitor_struct_vec);
        }

    }
}

impl GetDiff for MonitorDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, monitors, Vec<MonitorStruct>);
    }
}

TypeDiff!(FilesDiff, Files);

impl New<FilesDiff> for FilesDiff {
    fn new() -> FilesDiff {
        FilesDiff {
            config: Files {
                files: None
            },
            system: Files {
                files: None
            },
            diff: Diff {
                add: Files {
                    files: None
                },
                remove: Files {
                    files: None
                }
            }
        }
    }
}

impl Populate<Files> for FilesDiff {
    fn populate(&mut self, config: &Files) {
        self.get_config(config);
        self.get_system();
        self.get_diff();
    }
}

impl GetConfig<Files> for FilesDiff {
    fn get_config(&mut self, config: &Files) {
        SetNoneForVecIfNeededInConfig!(self, files, config);
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
                    let file_content: String = fs::read_to_string(file_path).expect("Able to read file's content to file!").trim().to_string();
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

impl GetDiff for FilesDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, files, Vec<TextToFile>);
    }
}
