use std::fs::create_dir_all;

use crate::{
    helper::{execute_output, execute_status},
    structure::Partitioning,
};

#[allow(dead_code, unused_variables)]
impl Partitioning {
    pub fn install_or_update(self, install: bool) {
        match (self.dual, self.disks, self.volume_groups) {
            (Some(dual), Some(ref disks), Some(ref volume_groups)) => {
                for disk in disks.clone() {
                    if check_for_lvm() {
                        remove_lvm_structure()
                    }
                    set_label(format!("/dev/{}", disk.name), String::from("gpt"));
                    // whole disk is reset now

                    for partition in disk.partitions {
                        // create paritions
                        create_partition(
                            format!("/dev/{}", disk.name),
                            partition.partition_type,
                            partition.start,
                            partition.end,
                        );

                        // encrypt if needed, then open
                        match partition.crypt.clone() {
                            Some(crypt_name) => {
                                encrypt_partition(
                                    format!("/dev/{}", partition.name),
                                    crypt_name.clone(),
                                );
                                open_encrypted_partition(
                                    format!("/dev/{}", partition.name),
                                    crypt_name,
                                )
                            }
                            None => (),
                        };

                        // create filesystem if needed
                        match (partition.file_system_type, partition.update, install) {
                            (Some(file_system), true, _) => {
                                create_filesystem(format!("/dev/{}", partition.name), file_system)
                            }
                            (Some(file_system), _, true) => {
                                create_filesystem(format!("/dev/{}", partition.name), file_system)
                            }
                            _ => (),
                        }

                        // mount if root
                        match partition.mount_point {
                            Some(mount_point) => {
                                if mount_point == "/" {
                                    mount_partition(
                                        format!("/dev/{}", partition.name),
                                        String::from("/mnt"),
                                    );
                                }
                            }
                            _ => (),
                        }

                        // create physical volume, wipefs and add to volume_group
                        match (install, partition.volume_group) {
                            (true, Some(volume_group)) => {
                                let device_path: String;
                                match partition.crypt {
                                    Some(crypt_name) => {
                                        device_path = format!("/dev/mapper/{}", crypt_name);
                                    }
                                    None => device_path = format!("/dev/{}", partition.name),
                                }
                                // create physical volume and wipe existing content
                                create_physical_volume_and_wipefs(device_path.clone());

                                if check_for_existing_volume_group(volume_group.clone()) == false {
                                    create_new_volume_group(device_path, volume_group)
                                } else {
                                    extend_existing_volume_group(device_path, volume_group)
                                }
                            }
                            _ => (),
                        }
                    }
                }

                for volume_group in volume_groups.clone() {
                    for logical_volume in volume_group.logical_volumes {
                        create_logical_volume(
                            volume_group.name.clone(),
                            logical_volume.size,
                            logical_volume.name.clone(),
                        );

                        // create filesystem if needed
                        match (
                            logical_volume.file_system_type,
                            logical_volume.update,
                            install,
                        ) {
                            (Some(file_system), true, _) => create_filesystem(
                                format!(
                                    "/dev/mapper/{}-{}",
                                    volume_group.name, logical_volume.name
                                ),
                                file_system,
                            ),
                            (Some(file_system), _, true) => create_filesystem(
                                format!(
                                    "/dev/mapper/{}-{}",
                                    volume_group.name, logical_volume.name
                                ),
                                file_system,
                            ),
                            _ => (),
                        }

                        // mount if root
                        match logical_volume.mount_point {
                            Some(mount_point) => {
                                if mount_point == "/" {
                                    mount_partition(
                                        format!(
                                            "/dev/mapper/{}-{}",
                                            volume_group.name.clone(),
                                            logical_volume.name
                                        ),
                                        String::from("/mnt"),
                                    );
                                }
                            }
                            _ => (),
                        }
                    }

                    for disk in disks {
                        for partition in disk.partitions.clone() {
                            match partition.mount_point {
                                Some(mount_point) => {
                                    if check_if_directory_exists(format!("/mnt/{}", mount_point)) {
                                        match create_dir_all(&format!("/mnt/{}", mount_point)) {
                                            Ok(out) => println!("{:?}", out),
                                            Err(e) => panic!("Error (panic): Error while creating directory {} - {}", format!("/mnt/{}", mount_point), e),
                                        };
                                    }
                                    mount_partition(
                                        format!("/dev/{}", partition.name),
                                        format!("/mnt/{}", mount_point),
                                    );
                                }
                                None => (),
                            }
                        }
                    }

                    for volume_group in volume_groups {
                        for logical_volume in volume_group.logical_volumes.clone() {
                            match logical_volume.mount_point {
                                Some(mount_point) => {
                                    if check_if_directory_exists(format!("/mnt/{}", mount_point)) {
                                        match create_dir_all(&format!("/mnt/{}", mount_point)) {
                                            Ok(out) => println!("{:?}", out),
                                            Err(e) => panic!("Error (panic): Error while creating directory {} - {}", format!("/mnt/{}", mount_point), e),
                                        };
                                    }
                                    mount_partition(
                                        format!(
                                            "/dev/mapper/{}-{}",
                                            volume_group.name, logical_volume.name
                                        ),
                                        format!("/mnt/{}", mount_point),
                                    );
                                }
                                None => (),
                            }
                        }
                    }
                }
            }
            _ => panic!(
                "Error (panic): Partitioning is not read correctly or the config has errors!"
            ),
        }
    }
}

#[allow(dead_code, unused_variables)]
fn check_for_lvm() -> bool {
    execute_status("vgs", "/")
}

#[allow(dead_code, unused_variables)]
fn remove_lvm_structure() {
    match execute_output("vgchange -a n", "/") {
        Ok(out) => println!(
            "{}",
            String::from_utf8(out.stdout)
                .expect("Error (expect): Failed to convert from utf8 to String")
        ),
        Err(e) => panic!(
            "Error (panic): Error while disabling volume groups (vgchange -a n) - {}",
            e
        ),
    }
}

#[allow(dead_code, unused_variables)]
fn set_label(path_to_disk: String, label: String) {}

#[allow(dead_code, unused_variables)]
fn create_partition(path_to_disk: String, partition_type: String, start: String, end: String) {}

#[allow(dead_code, unused_variables)]
fn encrypt_partition(path_to_par: String, crypt_name: String) {}

#[allow(dead_code, unused_variables)]
fn open_encrypted_partition(path_to_par: String, crypt_name: String) {}

#[allow(dead_code, unused_variables)]
fn create_filesystem(path_to_par: String, file_system: String) {}

#[allow(dead_code, unused_variables)]
fn mount_partition(path_to_par: String, mount_point: String) {}

#[allow(dead_code, unused_variables)]
fn add_partition_to_volume_group(path_to_par: String, volume_group: String) {}

#[allow(dead_code, unused_variables)]
fn create_physical_volume_and_wipefs(path_to_par: String) {}

#[allow(dead_code, unused_variables)]
fn check_for_existing_volume_group(volume_group: String) -> bool {
    true
}

#[allow(dead_code, unused_variables)]
fn create_new_volume_group(path_to_par: String, volume_group: String) {}

#[allow(dead_code, unused_variables)]
fn extend_existing_volume_group(path_to_par: String, volume_group: String) {}

#[allow(dead_code, unused_variables)]
fn create_logical_volume(volume_group: String, volume_size: String, volume_name: String) {}

#[allow(dead_code, unused_variables)]
fn check_if_directory_exists(path_to_dir: String) -> bool {
    true
}
