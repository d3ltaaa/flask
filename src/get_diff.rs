use crate::data_types::{
    DirectoriesDiff, DownloadsDiff, Fail2BanDiff, FilesDiff, GrubDiff, KeyboardDiff, LanguageDiff,
    MkinitcpioDiff, MonitorDiff, PackagesDiff, PacmanDiff, ServicesDiff, ShellDiff, SystemDiff,
    TimeDiff, UfwDiff, UserDiff,
};
use crate::structure::{
    CreateDirs, CurlDownload, GitDownload, Links, ManualInstallPackages, MonitorStruct, ReownDirs,
    TextToFile, Unzip, User,
};

pub trait GetDiff {
    fn get_diff(&mut self);
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
                if !$self.system.$field.clone().unwrap().contains(&arg) {
                    add_arg_vec.push(arg);
                }
            }
            if add_arg_vec.len() > 0 {
                $self.diff.add.$field = Some(add_arg_vec);
            }

            let mut remove_arg_vec: $data_type = Vec::new();
            for arg in $self.system.$field.clone().unwrap() {
                if !$self.config.$field.clone().unwrap().contains(&arg) {
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

impl GetDiff for KeyboardDiff {
    fn get_diff(&mut self) {
        CalcDiffVar!(self, mkinitcpio);

        // cant be build therefore it is just none
        self.diff.add.keyboard_tty = None;
        self.diff.remove.keyboard_tty = None;
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

impl GetDiff for LanguageDiff {
    fn get_diff(&mut self) {
        CalcDiffVar!(self, locale);
        CalcDiffVar!(self, character);
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

impl GetDiff for ShellDiff {
    fn get_diff(&mut self) {
        if self.config.default_shell != self.system.default_shell {
            self.diff.add.default_shell = self.config.default_shell.clone();
            self.diff.remove.default_shell = self.system.default_shell.clone();
        } else {
            self.diff.add.default_shell = None;
            self.diff.remove.default_shell = None;
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
        } else if self.config.user_list != None && self.system.user_list == None {
            self.diff.add.user_list = self.config.user_list.clone();
            self.diff.remove.user_list = None;
        } else if self.config.user_list == None && self.system.user_list != None {
            self.diff.add.user_list = None;
            self.diff.remove.user_list = self.system.user_list.clone();
        } else {
            // user_list: add
            let mut add_user_list_vec: Vec<String> = Vec::new();
            for user in self.config.user_list.clone().unwrap() {
                if !self.system.user_list.clone().unwrap().contains(&user) {
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
                if !self.config.user_list.clone().unwrap().contains(&user) {
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
                    let config_user: User = self
                        .config
                        .user_groups
                        .clone()
                        .unwrap()
                        .get(config_user_counter)
                        .unwrap()
                        .to_owned();
                    let system_user: User = self
                        .system
                        .user_groups
                        .clone()
                        .unwrap()
                        .get(system_user_counter)
                        .unwrap()
                        .to_owned();
                    if config_user.name == system_user.name {
                        // user is in config and in system
                        config_user_in_system = true;
                        let mut groups_vec: Vec<String> = Vec::new();
                        // go through the config's user's groups and look if they are in system as
                        // well
                        for group in config_user.groups.clone() {
                            if !system_user.groups.contains(&group) {
                                // if they are not found in the system version, add them to the
                                // groups_vec
                                groups_vec.push(group);
                            }
                        }
                        if groups_vec.len() > 0 {
                            add_groups_vec.push(User {
                                name: config_user.name,
                                groups: groups_vec,
                            });
                        }
                    }
                }

                // if the entire user is not found in system, than add him
                if !config_user_in_system {
                    add_groups_vec.push(
                        self.config
                            .user_groups
                            .clone()
                            .unwrap()
                            .get(config_user_counter)
                            .unwrap()
                            .to_owned(),
                    );
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
                    let config_user: User = self
                        .config
                        .user_groups
                        .clone()
                        .unwrap()
                        .get(config_user_counter)
                        .unwrap()
                        .to_owned();
                    let system_user: User = self
                        .system
                        .user_groups
                        .clone()
                        .unwrap()
                        .get(system_user_counter)
                        .unwrap()
                        .to_owned();
                    if system_user.name == config_user.name {
                        // user is in config and in system
                        // system_user_in_config = true;
                        let mut groups_vec: Vec<String> = Vec::new();
                        // go through the system's user's groups and look if they are in config as
                        // well
                        for group in system_user.groups.clone() {
                            if !config_user.groups.contains(&group) {
                                // if they are not in the config version, add them to the groups
                                // vec to be removed later
                                groups_vec.push(group);
                            }
                        }

                        if groups_vec.len() > 0 {
                            remove_groups_vec.push(User {
                                name: system_user.name,
                                groups: groups_vec,
                            });
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

impl GetDiff for PacmanDiff {
    fn get_diff(&mut self) {
        CalcDiffVar!(self, parallel);
    }
}

impl GetDiff for ServicesDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, services, Vec<String>);
        CalcDiffVec!(self, user_services, Vec<String>);
    }
}

impl GetDiff for PackagesDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, pacman_packages, Vec<String>);
        CalcDiffVec!(self, aur_packages, Vec<String>);
        CalcDiffVec!(self, manual_install_packages, Vec<ManualInstallPackages>);
    }
}

impl GetDiff for DirectoriesDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, reown_dirs, Vec<ReownDirs>);
        CalcDiffVec!(self, create_dirs, Vec<CreateDirs>);
        CalcDiffVec!(self, links, Vec<Links>);
    }
}

impl GetDiff for GrubDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, grub_cmdline_linux_default, Vec<String>);
    }
}

impl GetDiff for MkinitcpioDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, modules, Vec<String>);
        CalcDiffVec!(self, hooks, Vec<String>);
    }
}

impl GetDiff for DownloadsDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, git, Vec<GitDownload>);
        CalcDiffVec!(self, curl, Vec<CurlDownload>);
        CalcDiffVec!(self, unzip, Vec<Unzip>);
    }
}

impl GetDiff for UfwDiff {
    fn get_diff(&mut self) {
        CalcDiffVar!(self, incoming);
        CalcDiffVar!(self, outgoing);
        CalcDiffVec!(self, rules, Vec<String>);
    }
}

impl GetDiff for Fail2BanDiff {
    fn get_diff(&mut self) {
        CalcDiffVar!(self, ignoreip);
        CalcDiffVar!(self, maxretry);
        CalcDiffVar!(self, findtime);
        CalcDiffVar!(self, bantime);
        CalcDiffVec!(self, services, Vec<String>);
    }
}

impl GetDiff for MonitorDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, monitors, Vec<MonitorStruct>);
    }
}

impl GetDiff for FilesDiff {
    fn get_diff(&mut self) {
        CalcDiffVec!(self, files, Vec<TextToFile>);
    }
}
