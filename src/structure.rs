use crate::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
#[allow(dead_code)]
pub struct CargoToml {
    pub keyboard: Keyboard,
    pub time: Time,
    pub language: Language,
    pub system: System,
    pub users: Users,
    pub pacman: Pacman,
    pub partitioning: Partitioning,
    pub lvm: Lvm,
    pub packages: Packages,
    pub services: Services,
    pub directories: Directories,
    pub grub: Grub,
    pub mkinitcpio: Mkinitcpio,
    pub ufw: Ufw,
    pub fail2ban: Fail2Ban,
    pub downloads: Downloads,
    pub monitor: Monitor,
    pub files: Files,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Keyboard {
    pub keyboard_tty: Option<String>,
    pub mkinitcpio: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Time {
    pub timezone: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Language {
    pub locale: Option<String>,
    pub character: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct System {
    pub hostname: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct User {
    pub name: String,
    pub groups: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Users {
    pub user_list: Option<Vec<String>>,
    pub user_groups: Option<Vec<User>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Pacman {
    pub parallel: Option<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Partitioning {
    pub dual: bool,
    pub disks: Vec<String>,
    pub partitions: Vec<String>,
    pub start: Vec<String>,
    pub end: Vec<String>,
    pub partition_types: Vec<String>,
    pub crypts: Vec<String>,
    pub file_system_type: Vec<String>,
    pub mount_points: Vec<String>,
    pub hierarchy: Vec<i8>,
    pub update: Vec<bool>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Lvm {
    pub volume_groups: Vec<String>,
    pub logical_volumes: Vec<String>,
    pub sizes: Vec<String>,
    pub file_system_type: Vec<String>,
    pub mount_points: Vec<String>,
    pub hierarchy: Vec<i8>,
    pub update: Vec<bool>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ManualInstallPackages {
    pub name: String,
    pub check: String,
    pub command: String,
    pub sudo: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Packages {
    pub pacman_packages: Option<Vec<String>>,
    pub aur_packages: Option<Vec<String>>,
    pub manual_install_packages: Option<Vec<ManualInstallPackages>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Services {
    pub user_services: Option<Vec<String>>,
    pub services: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ReownDirs {
    pub directory: String,
    pub group: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Links {
    pub origin: String,
    pub destination: String,
    pub root: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CreateDirs {
    pub path: String,
    pub root: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Directories {
    pub reown_dirs: Option<Vec<ReownDirs>>,
    pub links: Option<Vec<Links>>,
    pub create_dirs: Option<Vec<CreateDirs>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Grub {
    pub grub_cmdline_linux_default: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Mkinitcpio {
    pub modules: Option<Vec<String>>,
    pub hooks: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Ufw {
    pub incoming: Option<String>,
    pub outgoing: Option<String>,
    pub rules: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Fail2Ban {
    pub ignoreip: Option<String>,
    pub bantime: Option<usize>,
    pub findtime: Option<usize>,
    pub maxretry: Option<usize>,
    pub services: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct GitDownload {
    pub url: String,
    pub path: String,
    pub root: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CurlDownload {
    pub url: String,
    pub path: String,
    pub file_name: String,
    pub root: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Unzip {
    pub path: String,
    pub root: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Downloads {
    pub git: Option<Vec<GitDownload>>,
    pub curl: Option<Vec<CurlDownload>>,
    pub unzip: Option<Vec<Unzip>>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct MonitorStruct {
    pub connection: String,
    pub resolution: String,
    pub refreshrate: String,
    pub position: String,
    pub scale: f32,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Monitor {
    pub monitors: Option<Vec<MonitorStruct>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TextToFile {
    pub write: String,
    pub path: String,
    pub file_name: String,
    pub root: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Files {
    pub files: Option<Vec<TextToFile>>,
}
