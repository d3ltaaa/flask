use crate::{
    data_types::PackagesDiff,
    function::Add,
    helper::execute_status,
    structure::{Packages, Partitioning},
};

pub fn install_important_packages(important_packages: Vec<String>) {
    let packages: PackagesDiff = PackagesDiff {
        config: Packages {
            pacman_packages: Some(important_packages.clone()),
            aur_packages: None,
            manual_install_packages: None,
        },
        system: Packages {
            pacman_packages: None,
            aur_packages: None,
            manual_install_packages: None,
        },
        diff: crate::data_types::Diff {
            add: Packages {
                pacman_packages: Some(important_packages),
                aur_packages: None,
                manual_install_packages: None,
            },
            remove: Packages {
                pacman_packages: None,
                aur_packages: None,
                manual_install_packages: None,
            },
        },
    };
    packages.add();
}

pub fn install_grub(partitioning: Partitioning) {
    match partitioning.disks.clone() {
        Some(disks) => {
            let mut boot_partition: Option<String> = None;
            for disk in disks {
                for partition in disk.partitions {
                    if partition.partition_type == "fat32" {
                        boot_partition = match partition.mount_point {
                            Some(mount_point) => Some(mount_point),
                            None => panic!(
                                "Error (panic): The fat32 partition does not have a mount point!"
                            ),
                        };
                    }
                }
            }
            match boot_partition {
                Some(partition) => {
                    let command: String = format!(
                        "grub-install --target=x86_64-efi --efi-directory={} --bootloader-id=GRUB",
                        partition
                    );
                    match execute_status(&command, "/") {
                        false => panic!("Error (panic): Failed to execute '{command}'"),
                        true => (),
                    }
                }
                None => panic!("Error (panic): No efi partition found"),
            }
        }
        None => panic!("Error (panic): No partitions found in config"),
    }
}
