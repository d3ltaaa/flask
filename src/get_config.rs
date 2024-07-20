use crate::data_types::{
    DirectoriesDiff, DownloadsDiff, Fail2BanDiff, FilesDiff, GrubDiff, KeyboardDiff, LanguageDiff,
    MkinitcpioDiff, MonitorDiff, PackagesDiff, PacmanDiff, ServicesDiff, ShellDiff, SystemDiff,
    TimeDiff, UfwDiff, UserDiff,
};
use crate::helper::is_user_root;
use crate::structure::{
    Directories, Downloads, Fail2Ban, Files, Grub, Keyboard, Language, Mkinitcpio, Monitor,
    Packages, Pacman, Partitioning, Services, Shell, System, Time, Ufw, Users,
};

macro_rules! SetNoneForVecIfNeededInConfig {
    ($self: ident, $field: ident, $toml_instance: ident) => {
        $self.config.$field = match $toml_instance.$field.clone() {
            Some(vector) => {
                if vector.len() == 0 {
                    None
                } else {
                    Some(vector)
                }
            }
            None => None,
        }
    };
}

pub trait GetConfig<T> {
    fn get_config(&mut self, config: &T);
}

impl GetConfig<Keyboard> for KeyboardDiff {
    fn get_config(&mut self, config: &Keyboard) {
        self.config.keyboard_tty = config.keyboard_tty.to_owned();

        match self.config.keyboard_tty.clone() {
            Some(keyboard_tty) => {
                if keyboard_tty == "" {
                    panic!("Config-Error: Keyboard_tty must be specified!");
                }
            }
            None => panic!("Config-Error: Keyboard_tty must be specified!"),
        }

        self.config.mkinitcpio = config.mkinitcpio.to_owned();

        match self.config.mkinitcpio.clone() {
            Some(mkinitcpio) => {
                if mkinitcpio == "" {
                    panic!("Config-Error: Mkinitcpio must be specified!");
                }
            }
            None => panic!("Config-Error: Mkinitcpio must be specified!"),
        }
    }
}

impl GetConfig<Time> for TimeDiff {
    fn get_config(&mut self, time: &Time) {
        self.config.timezone = time.timezone.to_owned();
        match self.config.timezone.clone() {
            Some(timezone) => {
                if timezone == "" {
                    panic!("Config-Error: Timezone must be specified!");
                }
            }
            None => {
                panic!("Config-Error: Timezone must be specified!");
            }
        }
    }
}

impl GetConfig<System> for SystemDiff {
    fn get_config(&mut self, system: &System) {
        self.config.hostname = system.hostname.to_owned();
        match self.config.hostname.clone() {
            Some(hostname) => {
                if hostname.contains(" ") {
                    panic!("Config-Error: Hostname is not allowed to contain ' '!");
                } else if hostname == "" {
                    panic!("Config-Error: Hostname must be specified!");
                }
            }
            None => panic!("Config-Error: Hostname must be specified!"),
        }
    }
}

impl GetConfig<Shell> for ShellDiff {
    fn get_config(&mut self, shell: &Shell) {
        self.config.default_shell = shell.default_shell.to_owned();
    }
}

impl GetConfig<Users> for UserDiff {
    fn get_config(&mut self, users: &Users) {
        self.config.user_list = users.user_list.clone();
        self.config.user_groups = users.user_groups.clone();
        match self.config.user_list.clone() {
            Some(user_list_vec) => {
                if user_list_vec.len() == 0 {
                    panic!("Config-Error: No user specified in config!");
                } else {
                    match self.config.user_groups.clone() {
                        Some(user_groups_vec) => {
                            if user_groups_vec.len() != user_list_vec.len() {
                                panic!("Config-Error: User_list and User_groups do not have the same length! Some groups are not specified for User(s)!")
                            }
                        }
                        None => panic!("Config-Error: No user groups specified at all!"),
                    }
                }
            }
            None => panic!("Config-Error: No user or groups specified at all!"),
        }
    }
}

impl GetConfig<Pacman> for PacmanDiff {
    fn get_config(&mut self, pacman: &Pacman) {
        self.config.parallel = pacman.parallel;
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

impl GetConfig<Packages> for PackagesDiff {
    fn get_config(&mut self, packages: &Packages) {
        if is_user_root() {
            SetNoneForVecIfNeededInConfig!(self, pacman_packages, packages);
        } else {
            SetNoneForVecIfNeededInConfig!(self, aur_packages, packages);
        }
        SetNoneForVecIfNeededInConfig!(self, manual_install_packages, packages);
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

impl GetConfig<Grub> for GrubDiff {
    fn get_config(&mut self, grub: &Grub) {
        SetNoneForVecIfNeededInConfig!(self, grub_cmdline_linux_default, grub);
    }
}

impl GetConfig<Mkinitcpio> for MkinitcpioDiff {
    fn get_config(&mut self, mkinitcpio: &Mkinitcpio) {
        SetNoneForVecIfNeededInConfig!(self, modules, mkinitcpio);
        SetNoneForVecIfNeededInConfig!(self, hooks, mkinitcpio);
    }
}

impl GetConfig<Downloads> for DownloadsDiff {
    fn get_config(&mut self, downloads: &Downloads) {
        SetNoneForVecIfNeededInConfig!(self, curl, downloads);
        SetNoneForVecIfNeededInConfig!(self, git, downloads);
        SetNoneForVecIfNeededInConfig!(self, unzip, downloads);
    }
}

impl GetConfig<Ufw> for UfwDiff {
    fn get_config(&mut self, ufw: &Ufw) {
        self.config.incoming = ufw.incoming.to_owned();
        self.config.outgoing = ufw.outgoing.to_owned();
        SetNoneForVecIfNeededInConfig!(self, rules, ufw);
    }
}

impl GetConfig<Fail2Ban> for Fail2BanDiff {
    fn get_config(&mut self, fail2ban: &Fail2Ban) {
        self.config.ignoreip = fail2ban.ignoreip.to_owned();
        self.config.bantime = fail2ban.bantime;
        self.config.findtime = fail2ban.findtime;
        self.config.maxretry = fail2ban.maxretry;
        SetNoneForVecIfNeededInConfig!(self, services, fail2ban);
    }
}

impl GetConfig<Monitor> for MonitorDiff {
    fn get_config(&mut self, monitor: &Monitor) {
        SetNoneForVecIfNeededInConfig!(self, monitors, monitor);
    }
}

impl GetConfig<Files> for FilesDiff {
    fn get_config(&mut self, config: &Files) {
        SetNoneForVecIfNeededInConfig!(self, files, config);
    }
}

impl GetConfig<Language> for LanguageDiff {
    fn get_config(&mut self, language: &Language) {
        self.config.locale = language.locale.clone();
        self.config.character = language.character.clone();

        match self.config.locale.clone() {
            Some(locale) => {
                if locale == "" {
                    panic!("Config-Error: Locale must be specified!");
                }
            }
            None => panic!("Config-Error: Locale must be specified!"),
        }

        match self.config.character.clone() {
            Some(character) => {
                if character == "" {
                    panic!("Config-Error: Locale must be specified!");
                }
            }
            None => panic!("Config-Error: Locale must be specified!"),
        }
    }
}

impl GetConfig<Partitioning> for PartitioningDiff {}
