use nix::libc::geteuid;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Output};

// pub fn notify(text: &str) {
//     println!("--------------| {text}");
// }

pub fn execute_status(cmd_str: &str, dir: &str) -> bool {
    match Command::new("bash")
        .args(["-c", cmd_str])
        .current_dir(&dir)
        .status()
    {
        Ok(o) => o,
        Err(_) => return false,
    }
    .success()
}

pub fn execute_output(cmd_str: &str, dir: &str) -> io::Result<Output> {
    Command::new("bash")
        .args(["-c", cmd_str])
        .current_dir(&dir)
        .output()
}

pub fn replace_line(path: &Path, old_str: &str, new_str: &str) -> bool {
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(path) {
        Err(why) => panic!("Error (panic): Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => {
            panic!("Error (panic): Failed to copy content to string: {}", why);
        }
        Ok(_) => (),
    };

    // create buffer in which line after line is read except for when it contains old_str
    let mut buf = String::new();

    for line in s.lines() {
        if line.starts_with(old_str) {
            buf.push_str(new_str);
            buf.push('\n');
        } else {
            buf.push_str(line);
            buf.push('\n');
        }
    }

    // create a new file in the same place and push buffer to it
    match File::create(path) {
        Ok(mut handle_create_path) => match handle_create_path.write_all(buf.as_bytes()) {
            Ok(handle_write_to_file) => println!("{:?}", handle_write_to_file),
            Err(why) => panic!("Error (panic): Unable to create {:?}: {}", path, why),
        },
        Err(why) => panic!("Error (panic): Unable to create {:?}: {}", path, why),
    };
    true
}

pub fn write_to_file(path: &Path, str: &str) -> bool {
    // create new file in path and push str to it
    match File::create(path) {
        Ok(mut handle_create_path) => match handle_create_path.write_all(str.as_bytes()) {
            Ok(handle_write_to_file) => println!("{:?}", handle_write_to_file),
            Err(why) => panic!("Error (panic): Unable to create {:?}: {}", path, why),
        },
        Err(why) => panic!("Error (panic): Unable to create {:?}: {}", path, why),
    };
    true
}

pub fn prepend_to_file(path: &Path, s: &str) -> bool {
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut file_content = String::new();
    match file.read_to_string(&mut file_content) {
        Err(why) => {
            println!("couldn't read {}: {}", display, why);
            return false;
        }
        Ok(_) => (),
    };

    let new_string: String = format!("{}\n{}", &s, &file_content);

    // create a new file in the same place and push string to it
    write_to_file(path, &new_string);

    true
}

pub fn read_in_variable(s: &str, divider: &str, name: &str) -> Option<String> {
    let mut collect: Option<(&str, &str)> = None;
    let contained: String = format!("{}{}", name, divider);
    for line in s.lines() {
        if line.starts_with(&contained) {
            collect = line.trim().split_once(divider);
        }
    }

    match collect {
        Some(string) => {
            let (_, val) = string;
            Some(val.trim().to_string())
        }
        None => None,
    }
}

pub fn append_to_file(path: &Path, s: &str) -> bool {
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut file_content = String::new();
    match file.read_to_string(&mut file_content) {
        Err(why) => {
            println!("couldn't read {}: {}", display, why);
            return false;
        }
        Ok(_) => (),
    };

    let new_string: String = format!("{}\n{}", &file_content, &s);

    // create a new file in the same place and push string to it
    write_to_file(path, &new_string);

    true
}

pub fn remove_from_file(path: &Path, s: &str) -> bool {
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut file_content = String::new();
    match file.read_to_string(&mut file_content) {
        Err(why) => {
            println!("couldn't read {}: {}", display, why);
            return false;
        }
        Ok(_) => (),
    };

    let new_string: String = file_content.replace(s, "").trim().to_string();

    // create a new file in the same place and push string to it
    write_to_file(path, &new_string);

    true
}

// pub fn get_original_user() -> String {
//     let output: Output = execute_output("echo $SUDO_USER", "/").expect("Retrieving SUDO_USER");
//     String::from_utf8(output.stdout)
//         .expect("Conversion")
//         .trim()
//         .to_string()
// }

pub fn is_user_root() -> bool {
    unsafe {
        match geteuid() {
            0 => true,
            _ => false,
        }
    }
}

pub fn printmsg<T>(op: &str, msg: &str, val: T)
where
    T: Debug,
{
    println!("|==============={}===============|", op);
    println!("{} => {:?}", msg, val);
}
