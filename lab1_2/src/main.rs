mod utils;
mod dir_entry;

use crate::dir_entry::DirEntry;

use color_eyre::eyre::Result;
use std::{env, ffi::OsString, cell::RefCell, rc::Rc};

fn main() -> Result<()> {
    color_eyre::install()?;

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <path> -L -S", args[0]);
        std::process::exit(1);
    }

    let root_dir = args[1].clone();
    let mut opt_index = 2;
    while opt_index < args.len() {
        let opt = &args[opt_index];
        match opt.as_str() {
            "-S" => {
                unsafe { dir_entry::FOLLOW_SIMLINK = true; }
            }
            "-L" => {
                unsafe { dir_entry::STRICT_LOOP = false; }
            }
            _ => {
                exit_err!("Wrong option: {}", opt);
            }
        }
        opt_index+=1;
    }

    unsafe {
        dir_entry::OPEN_MODE = (libc::O_RDONLY | libc::O_DIRECTORY) as libc::mode_t;
        if !dir_entry::FOLLOW_SIMLINK { dir_entry::OPEN_MODE |= libc::O_NOFOLLOW as libc::mode_t; }

        if !dir_entry::FOLLOW_SIMLINK && !dir_entry::STRICT_LOOP { exit_errx!("the -L flag should be used with -S"); }
    }

    let root = DirEntry::new(OsString::from(root_dir), None);
    let rc_root = Rc::new(RefCell::new(root));
    dir_entry::walk(Rc::clone(&rc_root))?;
    dir_entry::show(Rc::clone(&rc_root), 0);
    Ok(())
}