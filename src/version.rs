use crate::data_types::{
    DirectoriesDiff, DownloadsDiff, Fail2BanDiff, FilesDiff, GrubDiff, KeyboardDiff, LanguageDiff,
    MkinitcpioDiff, MonitorDiff, PackagesDiff, PacmanDiff, ServicesDiff, SystemDiff, TimeDiff,
    UfwDiff, UserDiff,
};
use crate::get_config::GetConfig;
use crate::get_diff::GetDiff;
use crate::get_system_from_other::GetSystemFromOther;
use std::{fs, path::Path, process::Output};

use chrono::NaiveDateTime;

use crate::{
    data_types::New, get_cargo_struct, helper::execute_output, CONFIG_DIR_PATH, CONFIG_PATH,
};

#[derive(Debug, PartialEq, Clone)]
pub struct AllVersions {
    pub versions: Option<Vec<Version>>,
    pub selected: Option<Version>,
    pub current: Option<Version>,
}

impl New<AllVersions> for AllVersions {
    fn new() -> AllVersions {
        AllVersions {
            versions: None,
            selected: None,
            current: None,
        }
    }
}

macro_rules! MakeDiffForType {
    ($cargo1_name:ident, $cargo2_name:ident, $element_name:ident, $Var_name: ident, $Type_name:ident) => {
        let mut $Var_name: $Type_name = $Type_name::new();
        $Var_name.get_config(&$cargo1_name.$element_name);
        $Var_name.get_system_from_other(&$cargo2_name.$element_name);
        $Var_name.get_diff();
        dbg!($Var_name.diff.clone());
    };
}

impl AllVersions {
    pub fn delete_version(&self, index: usize) {
        if index == 0 {
            println!("Cannot delete Version at index 0!");
            return;
        }

        match self.versions {
            Some(ref version_vec) => {
                let mut version_found: bool = false;
                for version in version_vec {
                    if version.index.unwrap() == index {
                        version_found = true;
                        let version_path_string: String = version.config_path.clone().unwrap();
                        match fs::remove_file(Path::new(&version_path_string)) {
                            Ok(_) => println!("Deleted: {}", &version_path_string),
                            Err(e) => {
                                panic!(
                                    "Error (panic): Failed deleting {}: {}",
                                    &version_path_string, e
                                )
                            }
                        }
                    }
                }
                if !version_found {
                    println!("No Version with index: {}!", index);
                }
            }
            None => panic!("Error (panic): No Versions available while deleting version"),
        }
    }

    pub fn delete_last_versions(&self, number: usize) {
        match self.versions {
            Some(ref version_vec) => {
                if number == 0 {
                    println!("Nothing deleted!");
                    return;
                } else if number >= version_vec.len() {
                    println!("You cannot delete the current config!");
                    return;
                } else {
                    let mut counter: usize = 0;
                    for version in version_vec.iter().rev().collect::<Vec<&Version>>() {
                        self.delete_version(version.index.unwrap());
                        counter += 1;
                        if counter == number {
                            break;
                        }
                    }
                }
            }
            None => panic!("Error (panic): No versions found while deleting last versions!"),
        }
    }

    pub fn commit(&self) {
        let mut current_highest_index: usize = 0;
        let mut current_latest_commit_version: Version = Version::new();
        for version in self.versions.clone().unwrap() {
            if version.index.unwrap() == 0 {
                current_latest_commit_version = version;
            } else if version.index.unwrap() > current_highest_index {
                current_highest_index = version.index.unwrap();
            }
        }

        let current_commited_config: String =
            fs::read_to_string(Path::new(CONFIG_PATH)).expect("Reading current config to String");
        let current_config: String = fs::read_to_string(Path::new(
            &current_latest_commit_version.config_path.clone().unwrap(),
        ))
        .expect("Reading current config to String");

        if current_commited_config != current_config {
            // get commit message from user
            let mut new_commit_message: String = String::new();
            loop {
                println!("Enter commit message: ");
                match std::io::stdin().read_line(&mut new_commit_message) {
                    Ok(_) => (),
                    Err(e) => panic!("{}", e),
                }
                if new_commit_message == "\n" {
                    println!("Type a commit message!");
                    new_commit_message = String::new();
                } else if new_commit_message.contains("__") {
                    println!("The message is not allowed to contain '__'");
                    new_commit_message = String::new();
                } else {
                    new_commit_message = new_commit_message.trim().to_string();
                    break;
                }
            }

            // get the curent time in the right format
            let datetime: NaiveDateTime = chrono::Utc::now().naive_utc();
            let datetime_string: String = datetime.format("%Y-%m-%d_%H-%M-%S").to_string();

            // move config into 0__commitmessage__datetime.toml to i__commitmessage__datetime.toml
            let old_current: String = current_latest_commit_version.config_path.clone().unwrap();
            let old_current_file_string: String = current_latest_commit_version
                .config_path
                .unwrap()
                .split_once("/0__")
                .unwrap()
                .1
                .to_string();

            let old_current_evolution: String = format!(
                "{}/{}__{}",
                CONFIG_DIR_PATH,
                current_highest_index + 1,
                old_current_file_string
            );
            let new_current: String = format!(
                "{}/0__{}__{}.toml",
                CONFIG_DIR_PATH, new_commit_message, datetime_string
            );

            // move config into 0__commitmessage__datetime.toml
            match fs::rename(Path::new(&old_current), Path::new(&old_current_evolution)) {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }
            // copy 0__commitmessage__datetime into config.toml
            match fs::copy(Path::new(&CONFIG_PATH), Path::new(&new_current)) {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }
        } else {
            println!("No changes to Commit!");
        }
    }

    pub fn get_versions(&mut self) {
        // list contents of .version
        let argument: String = format!("ls -A {}", CONFIG_DIR_PATH);
        let output: Output =
            execute_output(&argument, "/").expect("List contents of Config directory");
        let error_string: String = String::from_utf8(output.stderr).expect("Conversion");
        let output_string: String = String::from_utf8(output.stdout).expect("Conversion");

        if error_string != "" {
            panic!(
                "Error while listing the contents of {}: {:?}",
                CONFIG_DIR_PATH, error_string
            );
        } else if output_string == "" {
            panic!(
                "Error {} does not contain any versions: {:?}",
                CONFIG_DIR_PATH, output_string
            );
        } else {
            // get index, datetime and path
            for file in output_string.lines() {
                let name: String = match file.split("__").collect::<Vec<&str>>().get(1) {
                    Some(val) => val.parse().unwrap(),
                    None => panic!("Wrong file name, does not contain '__'!"),
                };
                let date_time_str: &str = match file.split("__").collect::<Vec<&str>>().get(2) {
                    Some(val) => val,
                    None => panic!("Wrong file name, at least one '__' missing!"),
                };
                let date_time: NaiveDateTime =
                    NaiveDateTime::parse_from_str(date_time_str, "%Y-%m-%d_%H-%M-%S.toml")
                        .expect("Time Conversion");
                let path: String = format!("{}/{}", CONFIG_DIR_PATH, file);

                let index: usize = match file.split_once("__") {
                    Some(val) => match val.0.parse() {
                        Ok(val) => {
                            if val == 0 {
                                self.current = Some(Version {
                                    name: Some(name.clone()),
                                    index: Some(0),
                                    last_use_date: Some(date_time),
                                    config_path: Some(path.clone()),
                                    selected: false,
                                })
                            }
                            val
                        }
                        Err(e) => panic!("{}", e),
                    },
                    None => panic!("Wrong file name, does not contain '__'!"),
                };

                // update self.versions
                if self.versions == None {
                    self.versions = Some(Vec::new());
                }
                let version_push: Version = Version {
                    name: Some(name),
                    index: Some(index),
                    selected: false,
                    last_use_date: Some(date_time),
                    config_path: Some(path),
                };
                match self.versions {
                    Some(ref mut version_vec) => {
                        if !version_vec.contains(&version_push) {
                            version_vec.push(version_push);
                        } else {
                            panic!("Detect the same Version twice!");
                        }
                    }
                    None => (),
                }
            }

            // check wether something is missing
            match self.current {
                Some(_) => (),
                None => panic!("No current config version in {CONFIG_DIR_PATH}"),
            }
            match self.versions {
                Some(_) => (),
                None => panic!("No versions found in {CONFIG_DIR_PATH}"),
            }
        }
    }

    pub fn list_versions(&self) {
        match self.versions {
            Some(ref version_vec) => {
                for version in version_vec {
                    let mut formatted: String = format!(
                        "{}: {} ({})",
                        version.index.unwrap(),
                        version.name.clone().unwrap(),
                        version.last_use_date.unwrap()
                    );
                    if version.selected {
                        formatted.push_str(" * selected");
                    }
                    println!("{formatted}");
                }
            }
            None => panic!("No Versions found while listing!"),
        }
    }

    pub fn align_version_indexes(&mut self) {
        match self.versions {
            Some(ref mut version_vec) => {
                let mut counter: usize = 0;
                version_vec
                    .sort_by(|a: &Version, b: &Version| a.index.unwrap().cmp(&b.index.unwrap()));
                for version in version_vec {
                    if version.index.unwrap() != counter {
                        version.index = Some(counter);
                    }
                    counter += 1;
                }
            }
            None => panic!("No Versions found while aligning"),
        }
    }

    pub fn update_indexes_to_path(&mut self) {
        match self.versions {
            Some(ref mut version_vec) => {
                for version in version_vec {
                    let path: (usize, String) = match version
                        .config_path
                        .clone()
                        .unwrap()
                        .split('/')
                        .collect::<Vec<&str>>()
                        .last()
                        .unwrap()
                        .split_once("__")
                    {
                        Some(val) => {
                            let path_index = match val.0.parse::<usize>() {
                                Ok(val) => val,
                                Err(_) => panic!("Wrong index in path: {:?}", val),
                            };
                            (path_index, val.1.to_string())
                        }
                        None => panic!("Wrong file name, does not contain '__'!"),
                    };
                    if path.0 != version.index.unwrap() {
                        let formatted_path: String =
                            format!("{}/{}__{}", CONFIG_DIR_PATH, version.index.unwrap(), path.1);
                        version.config_path = Some(formatted_path);
                    }
                }
            }
            None => panic!("No Versions found while updating indexes to paths"),
        }
    }

    pub fn update_paths(&self) {
        match self.versions {
            Some(ref version_vec) => {
                let mut old_versions: AllVersions = AllVersions::new();
                old_versions.get_versions();
                old_versions.align_version_indexes();

                match old_versions.versions {
                    Some(ref old_version_vec) => {
                        if old_version_vec.len() == version_vec.len() {
                            for i in 0..version_vec.len() {
                                // println!("");
                                // println!(
                                // "DEBUG: move ...\n{} to ...\n{}",
                                //     old_version_vec[i].config_path.clone().unwrap(),
                                //     version_vec[i].config_path.clone().unwrap()
                                // );

                                match fs::rename(
                                    Path::new(&old_version_vec[i].config_path.clone().unwrap()),
                                    Path::new(&version_vec[i].config_path.clone().unwrap()),
                                ) {
                                    Ok(_) => (),
                                    Err(e) => panic!("{}", e),
                                }
                            }
                        } else {
                            panic!("Version_vec lenghts are not the same!");
                        }
                    }
                    None => panic!("No Versions found while Updating Paths!"),
                }
            }
            None => panic!("No Versions found while Updating Paths!"),
        }
    }

    pub fn rollback(&self, index: usize) {
        if index == 0 {
            println!("Cannot rollback to index 0: Skipping!");
            return;
        }

        let mut current_highest_index: usize = 0;
        let mut current_latest_commit_version: Version = Version::new();
        let mut index_vec: Vec<usize> = Vec::new();
        let mut rollback_from: Version = Version::new();
        for version in self.versions.clone().unwrap() {
            if version.index.unwrap() == 0 {
                current_latest_commit_version = version.clone();
            } else if version.index.unwrap() > current_highest_index {
                current_highest_index = version.index.unwrap();
            }
            if version.index.unwrap() == index {
                rollback_from = version.clone();
                println!(
                    "Rolling back to: [{}: {} ({})]",
                    version.index.unwrap(),
                    version.name.unwrap(),
                    version.last_use_date.unwrap()
                );
            }
            index_vec.push(version.index.unwrap());
        }

        if !index_vec.contains(&index) {
            println!("Index {} not valid! Valid: {:?}", index, index_vec);
            return;
        }

        let current_commited_config: String =
            fs::read_to_string(Path::new(CONFIG_PATH)).expect("Reading current config to String");
        let current_config: String = fs::read_to_string(Path::new(
            &current_latest_commit_version.config_path.clone().unwrap(),
        ))
        .expect("Reading current config to String");

        if current_commited_config == current_config {
            // get the index of the rollback
            // move 0__commitmessage__datetime.toml to i__commitmessage__datetime
            let old_current_string: String = current_latest_commit_version
                .config_path
                .clone()
                .unwrap()
                .split_once("/0__")
                .unwrap()
                .1
                .to_string();

            let old_current_evolution: String = format!(
                "{}/{}__{}",
                CONFIG_DIR_PATH,
                current_highest_index + 1,
                old_current_string
            );

            // println!(
            // "DEBUG: move ...\n{} to ...\n{}",
            //     current_latest_commit_version.config_path.clone().unwrap(),
            //     old_current_evolution
            // );

            match fs::rename(
                Path::new(&current_latest_commit_version.config_path.unwrap()),
                &old_current_evolution,
            ) {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }

            let rollback_string: String = rollback_from
                .config_path
                .clone()
                .unwrap()
                .split_once("__")
                .unwrap()
                .1
                .to_string();

            let rollback_to: String = format!("{}/0__{}", CONFIG_DIR_PATH, rollback_string);

            // println!(
            // "DEBUG: move\n{} to ...\n{}",
            //     rollback_from.config_path.clone().unwrap(),
            //     rollback_to
            // );

            match fs::rename(Path::new(&rollback_from.config_path.unwrap()), &rollback_to) {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }

            // println!("DEBUG: copy\n{} to ...\n{}", rollback_to, CONFIG_PATH);

            match fs::copy(Path::new(&rollback_to), &CONFIG_PATH) {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }
        } else {
            println!("Uncommited changes in config!");
        }
    }

    pub fn to_latest(&self) -> usize {
        match self.versions {
            Some(ref version_vec) => {
                let mut latest_index: usize = 0;
                for version in version_vec {
                    let mut current_latest: Option<NaiveDateTime> = None;
                    for version_find in version_vec {
                        if version_find.index.unwrap() == latest_index {
                            current_latest = Some(version_find.last_use_date.unwrap());
                        }
                    }
                    let test_against: NaiveDateTime = version.last_use_date.unwrap();
                    // println!("focused: {test_against}");
                    match current_latest {
                        Some(val) => {
                            if val < test_against {
                                // println!("{current_latest} < {test_against}");
                                latest_index = version.index.unwrap();
                            }
                        }
                        None => (),
                    }
                }
                // println!("Latest index: {latest_index}");
                latest_index
            }
            None => panic!("No Versions found while rolling back to latest!"),
        }
    }

    pub fn print_diff(&mut self, index1: usize, index2: usize, diff_to_current: bool) {
        // get path to config to compare
        let mut index1_path: String = String::new();
        let mut index2_path: String = String::new();
        let mut index1_found: bool = false;
        let mut index2_found: bool = false;
        for version in self.versions.clone().unwrap() {
            if version.index.unwrap() == index1 && !diff_to_current {
                index1_path = version.config_path.clone().unwrap();
                index1_found = true;
            }
            if version.index.unwrap() == index2 {
                index2_path = version.config_path.clone().unwrap();
                index2_found = true;
            }
        }

        if diff_to_current {
            index1_path = String::from(CONFIG_PATH);
            index1_found = true;
        } else if !index1_found {
            println!("First index not valid: {}", index1);
        }
        if !index2_found {
            println!("Second index not valid: {}", index2);
        }
        if !index1_found || !index2_found {
            std::process::exit(1);
        }

        // read in config to compare
        let index1_toml = get_cargo_struct(Path::new(&index1_path));
        let index2_toml = get_cargo_struct(Path::new(&index2_path));

        MakeDiffForType!(
            index1_toml,
            index2_toml,
            keyboard,
            keyboard_diff,
            KeyboardDiff
        );
        MakeDiffForType!(index1_toml, index2_toml, time, time_diff, TimeDiff);
        MakeDiffForType!(
            index1_toml,
            index2_toml,
            language,
            language_diff,
            LanguageDiff
        );
        MakeDiffForType!(index1_toml, index2_toml, system, system_diff, SystemDiff);
        MakeDiffForType!(index1_toml, index2_toml, users, user_diff, UserDiff);
        MakeDiffForType!(index1_toml, index2_toml, pacman, pacman_diff, PacmanDiff);
        MakeDiffForType!(
            index1_toml,
            index2_toml,
            services,
            services_diff,
            ServicesDiff
        );
        MakeDiffForType!(
            index1_toml,
            index2_toml,
            packages,
            packages_diff,
            PackagesDiff
        );
        MakeDiffForType!(
            index1_toml,
            index2_toml,
            directories,
            directories_diff,
            DirectoriesDiff
        );
        MakeDiffForType!(index1_toml, index2_toml, grub, grub_diff, GrubDiff);
        MakeDiffForType!(
            index1_toml,
            index2_toml,
            mkinitcpio,
            mkinitcpio_diff,
            MkinitcpioDiff
        );
        MakeDiffForType!(index1_toml, index2_toml, ufw, ufw_diff, UfwDiff);
        MakeDiffForType!(
            index1_toml,
            index2_toml,
            fail2ban,
            fail2ban_diff,
            Fail2BanDiff
        );
        MakeDiffForType!(
            index1_toml,
            index2_toml,
            downloads,
            downloads_diff,
            DownloadsDiff
        );
        MakeDiffForType!(index1_toml, index2_toml, monitor, monitor_diff, MonitorDiff);
        MakeDiffForType!(index1_toml, index2_toml, files, file_diff, FilesDiff);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Version {
    pub name: Option<String>,
    pub index: Option<usize>,
    pub selected: bool,
    pub last_use_date: Option<NaiveDateTime>,
    pub config_path: Option<String>,
}

impl Version {
    pub fn new() -> Version {
        Version {
            name: None,
            index: None,
            selected: false,
            last_use_date: None,
            config_path: None,
        }
    }
}
