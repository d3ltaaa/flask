use crate::helper::is_user_root;
use crate::structure::{Directories, Downloads, Fail2Ban, Files, Grub, Language, Mkinitcpio, Monitor, Packages, Pacman, Services, Shell, System, Time, Ufw, Users};
use crate::structure::Keyboard;
use crate::{HOSTNAME_PATH, HOSTS_PATH, LOCALE_CONF_PATH, LOCALE_GEN_PATH};
use std::fs;
use std::path::Path;
use crate::get_diff::GetDiff;
use crate::get_config::GetConfig;
use crate::get_system::GetSystem;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Diff<T> {
    pub add: T,
    pub remove: T,
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

pub trait New<T> {
    fn new() -> T;
}

pub trait Populate<T> {
    fn populate(&mut self, config: &T);
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

TypeDiff!(ShellDiff, Shell);

impl New<ShellDiff> for ShellDiff {
    fn new() -> ShellDiff {
        ShellDiff {
            config: Shell {
                default_shell: None,
            },
            system: Shell {
                default_shell: None,
            },
            diff: Diff {
                add: Shell {
                    default_shell: None,
                },
                remove: Shell {
                    default_shell: None,
                }
            }
        }
    }
}

impl Populate<Shell> for ShellDiff {
    fn populate(&mut self, config: &Shell) {
        self.get_config(config);
        self.get_system();
        self.get_diff();
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

TypeDiff!(PackagesDiff, Packages);

impl New<PackagesDiff> for PackagesDiff {
    fn new() -> PackagesDiff {
        PackagesDiff {
            config: Packages {
                pacman_packages: None,
                aur_packages: None,
                manual_install_packages: None,
            },
            system: Packages {
                pacman_packages: None,
                aur_packages: None,
                manual_install_packages: None,
            },
            diff: Diff {
                add: Packages {
                    pacman_packages: None,
                    aur_packages: None,
                    manual_install_packages: None,
                },
                remove: Packages {
                    pacman_packages: None,
                    aur_packages: None,
                    manual_install_packages: None,
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

TypeDiff!(GrubDiff, Grub);

impl New<GrubDiff> for GrubDiff {
    fn new() -> GrubDiff {
        GrubDiff {
             config: Grub {
                 grub_cmdline_linux_default: None,
                 grub_resume: None,
                 grub_crypt: None,
                 grub_timeout: None,
             },
             system: Grub {
                 grub_cmdline_linux_default: None,
                 grub_resume: None,
                 grub_crypt: None,
                 grub_timeout: None,
             },
             diff: Diff {
                 add: Grub {
                     grub_cmdline_linux_default: None,
                 grub_resume: None,
                 grub_crypt: None,
                 grub_timeout: None,
                 },
                 remove: Grub {
                     grub_cmdline_linux_default: None,
                 grub_resume: None,
                 grub_crypt: None,
                 grub_timeout: None,
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
