use crate::{
    data_types::PacmanDiff,
    function::Add,
    helper::execute_status,
    structure::{Keyboard, Pacman},
};

pub fn setup_environment_on_installation(keyboard: Keyboard, pacman: Pacman) {
    match (keyboard.keyboard_tty, pacman.parallel) {
        (Some(keyboard_tty), Some(parallel)) => {
            let command: String = format!("loadkeys {}", keyboard_tty);
            match execute_status(&command, "/") {
                true => (),
                false => panic!("Error (panic): Failed to execute '{command}'"),
            }
            let command: String = String::from("pacman -Sy");
            match execute_status(&command, "/") {
                true => (),
                false => panic!("Error (panic): Failed to execute '{command}'"),
            }
            let command: String = String::from("timedatectl set-ntp true");
            match execute_status(&command, "/") {
                true => (),
                false => panic!("Error (panic): Failed to execute '{command}'"),
            }
            let pacmandiff: PacmanDiff = PacmanDiff {
                config: Pacman {
                    parallel: Some(parallel),
                },
                system: Pacman { parallel: None },
                diff: crate::data_types::Diff {
                    add: Pacman {
                        parallel: Some(parallel),
                    },
                    remove: Pacman { parallel: None },
                },
            };
            pacmandiff.add();
        }
        _ => panic!("Error (panic): Failed to read keyboard, pacman and time from config!"),
    }
}
