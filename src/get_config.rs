use crate::data_types::{
    DirectoriesDiff, DownloadsDiff, Fail2BanDiff, FilesDiff, GrubDiff, KeyboardDiff, LanguageDiff,
    MkinitcpioDiff, MonitorDiff, PackagesDiff, PacmanDiff, ServicesDiff, SystemDiff, TimeDiff,
    UfwDiff, UserDiff,
};
use crate::helper::is_user_root;
use crate::structure::{
    Directories, Downloads, Fail2Ban, Files, Grub, Keyboard, Language, Mkinitcpio, Monitor,
    Packages, Pacman, Services, System, Time, Ufw, Users,
};

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

pub trait GetConfig<T> {
    fn get_config(&mut self, config: &T);
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
            }
            None => panic!("Hostname must be specified!"),
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
            }
            None => (),
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
            SetNoneForVecIfNeededInConfig!(self, manual_install_packages, packages);
            SetNoneForVecIfNeededInConfig!(self, build_packages, packages);
        } else {
            SetNoneForVecIfNeededInConfig!(self, aur_packages, packages);
        }
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
        self.config.incoming = ufw.incoming.clone();
        self.config.outgoing = ufw.outgoing.clone();
        self.config.rules = ufw.rules.clone();
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

impl GetConfig<Monitor> for MonitorDiff {
    fn get_config(&mut self, monitor: &Monitor) {
        self.config.monitors = monitor.monitors.clone();
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
                    panic!("Locale must be specified!");
                }
            }
            None => panic!("Locale must be specified!"),
        }

        match self.config.character.clone() {
            Some(character) => {
                if character == "" {
                    panic!("Locale must be specified!");
                }
            }
            None => panic!("Locale must be specified!"),
        }
    }
}
