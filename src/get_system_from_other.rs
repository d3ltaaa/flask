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

// trait declaration
pub trait GetSystemFromOther<T> {
    fn get_system_from_other(&mut self, other_config: &T);
}

// macros
macro_rules! ReadOtherToSystem {
    ($self: ident, $field: ident, $other: ident) => {
        $self.system.$field = $other.$field.clone();
    };
}

macro_rules! ReadOtherVecToSystem {
    ($self: ident, $field: ident, $other: ident) => {
        match $other.$field.clone() {
            Some(var_vec) => {
                if var_vec.len() == 0 {
                    $self.system.$field = None;
                } else {
                    $self.system.$field = Some(var_vec.clone());
                }
            }
            None => $self.system.$field = None,
        }
    };
}

// implementations
impl GetSystemFromOther<Keyboard> for KeyboardDiff {
    fn get_system_from_other(&mut self, other_config: &Keyboard) {
        ReadOtherToSystem!(self, keyboard_tty, other_config);
        ReadOtherToSystem!(self, mkinitcpio, other_config);
    }
}

impl GetSystemFromOther<Time> for TimeDiff {
    fn get_system_from_other(&mut self, other_config: &Time) {
        ReadOtherToSystem!(self, timezone, other_config);
    }
}

impl GetSystemFromOther<Language> for LanguageDiff {
    fn get_system_from_other(&mut self, other_config: &Language) {
        ReadOtherToSystem!(self, locale, other_config);
        ReadOtherToSystem!(self, character, other_config);
    }
}

impl GetSystemFromOther<System> for SystemDiff {
    fn get_system_from_other(&mut self, other_config: &System) {
        ReadOtherToSystem!(self, hostname, other_config);
    }
}

impl GetSystemFromOther<Users> for UserDiff {
    fn get_system_from_other(&mut self, other_config: &Users) {
        ReadOtherVecToSystem!(self, user_list, other_config);
        ReadOtherVecToSystem!(self, user_groups, other_config);
    }
}

impl GetSystemFromOther<Pacman> for PacmanDiff {
    fn get_system_from_other(&mut self, other_config: &Pacman) {
        ReadOtherToSystem!(self, parallel, other_config);
    }
}

impl GetSystemFromOther<Services> for ServicesDiff {
    fn get_system_from_other(&mut self, other_config: &Services) {
        if is_user_root() {
            ReadOtherVecToSystem!(self, services, other_config);
            self.system.user_services = None;
        } else {
            ReadOtherVecToSystem!(self, user_services, other_config);
            self.system.services = None;
        }
    }
}

impl GetSystemFromOther<Packages> for PackagesDiff {
    fn get_system_from_other(&mut self, other_config: &Packages) {
        if is_user_root() {
            ReadOtherVecToSystem!(self, pacman_packages, other_config);
            self.system.aur_packages = None;
        } else {
            ReadOtherVecToSystem!(self, aur_packages, other_config);
            self.system.pacman_packages = None;
        }
        ReadOtherVecToSystem!(self, manual_install_packages, other_config);
    }
}

impl GetSystemFromOther<Directories> for DirectoriesDiff {
    fn get_system_from_other(&mut self, other_config: &Directories) {
        if is_user_root() {
            ReadOtherVecToSystem!(self, reown_dirs, other_config);
        } else {
            self.system.reown_dirs = None;
        }
        ReadOtherVecToSystem!(self, links, other_config);
        ReadOtherVecToSystem!(self, create_dirs, other_config);
    }
}

impl GetSystemFromOther<Grub> for GrubDiff {
    fn get_system_from_other(&mut self, other_config: &Grub) {
        ReadOtherVecToSystem!(self, grub_cmdline_linux_default, other_config);
    }
}

impl GetSystemFromOther<Mkinitcpio> for MkinitcpioDiff {
    fn get_system_from_other(&mut self, other_config: &Mkinitcpio) {
        ReadOtherVecToSystem!(self, modules, other_config);
        ReadOtherVecToSystem!(self, hooks, other_config);
    }
}

impl GetSystemFromOther<Downloads> for DownloadsDiff {
    fn get_system_from_other(&mut self, other_config: &Downloads) {
        ReadOtherVecToSystem!(self, git, other_config);
        ReadOtherVecToSystem!(self, curl, other_config);
        ReadOtherVecToSystem!(self, unzip, other_config);
    }
}

impl GetSystemFromOther<Ufw> for UfwDiff {
    fn get_system_from_other(&mut self, other_config: &Ufw) {
        ReadOtherToSystem!(self, incoming, other_config);
        ReadOtherToSystem!(self, outgoing, other_config);
        ReadOtherVecToSystem!(self, rules, other_config);
    }
}

impl GetSystemFromOther<Fail2Ban> for Fail2BanDiff {
    fn get_system_from_other(&mut self, other_config: &Fail2Ban) {
        ReadOtherToSystem!(self, ignoreip, other_config);
        ReadOtherToSystem!(self, bantime, other_config);
        ReadOtherToSystem!(self, findtime, other_config);
        ReadOtherToSystem!(self, maxretry, other_config);
        ReadOtherVecToSystem!(self, services, other_config);
    }
}

impl GetSystemFromOther<Monitor> for MonitorDiff {
    fn get_system_from_other(&mut self, other_config: &Monitor) {
        ReadOtherVecToSystem!(self, monitors, other_config);
    }
}

impl GetSystemFromOther<Files> for FilesDiff {
    fn get_system_from_other(&mut self, other_config: &Files) {
        ReadOtherVecToSystem!(self, files, other_config);
    }
}
