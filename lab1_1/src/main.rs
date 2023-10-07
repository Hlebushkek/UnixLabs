use std::ffi::CStr;
use std::env;
use std::fmt::{self, Display};

mod error;

struct PathList(Vec<String>);

impl PathList {
    fn new() -> Self {
        PathList(Vec::new())
    }
}

impl Display for PathList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "/")
        } else {
            for elem in self.0.iter().rev() {
                write!(f, "/{}", elem)?;
            }
            Ok(())
        }
    }
}

fn get_cwd() -> PathList {
    let mut path_list = PathList::new();

    let mut fd: i32;
    let mut cfd = libc::AT_FDCWD;
    let mut found_all = false;

    let mut ent: *mut libc::dirent;

    while !found_all {
        fd = unsafe { libc::openat(cfd, "..\0".as_ptr() as *const i8, libc::O_RDONLY) };
        if fd < 0 {
            exit_err!("openat()");
        }
        
        let dir = unsafe { libc::fdopendir(fd) };
        if dir.is_null() {
            exit_err!("fdopendir()");
        }

        let mut statbuf: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstatat(cfd, ".\0".as_ptr() as *const i8, &mut statbuf, 0) } < 0 {
            exit_err!("fstatat()");
        }
        
        let dev = statbuf.st_dev;
        let ino = statbuf.st_ino;

        loop {
            let errno;

            ent = unsafe { libc::readdir(dir) };
            if ent.is_null() {
                errno = unsafe { *libc::__error() };
                if errno != 0 {
                    exit_err!("readdir()");
                }
                break;
            }
            
            if unsafe { libc::fstatat(fd, (*ent).d_name.as_ptr(), &mut statbuf, libc::AT_SYMLINK_NOFOLLOW) } < 0 {
                errno = unsafe { *libc::__error() };
                if errno == libc::ENOENT {
                    continue;
                }
                exit_err!("fstatat()");
            }

            // if unsafe { /libc::S_ISDIR(statbuf.st_mode) } != 0
            if (statbuf.st_mode & libc::S_IFMT) == libc::S_IFDIR
                && statbuf.st_dev == dev
                && statbuf.st_ino == ino
            {
                let str = unsafe {
                    CStr::from_ptr((*ent).d_name.as_ptr() as *const libc::c_char).to_str().unwrap()
                };
                path_list.0.push(str.to_string());
                println!("{}", str.to_string());
                break;
            }
        }

        if unsafe { libc::closedir(dir) } < 0 {
            exit_err!("closedir()");
        }

        if ent.is_null() {
            exit_errx!("some component in CWD was removed.");
        }

        fd = cfd;

        if unsafe { ((*ent).d_name[0] == 46 && (*ent).d_name[1] == 0) ||
                    ((*ent).d_name[1] == 46 && (*ent).d_name[2] == 0) }
        {
            found_all = true;
            path_list.0.pop();
        } else {
            cfd = unsafe { libc::openat(cfd, "..\0".as_ptr() as *const i8, libc::O_RDONLY) };
            if cfd < 0 {
                exit_err!("openat()");
            }
        }

        if fd != libc::AT_FDCWD && unsafe { libc::close(fd) } < 0 {
            exit_err!("close()");
        }
    }

    return path_list
}

fn main() {
    let mut root_dir: Option<String> = None;
    let mut work_dir: Option<String> = None;

    let args: Vec<String> = env::args().collect();

    let mut opt_index = 1;
    println!("{:?}", args);
    while opt_index < args.len() {
        let opt = &args[opt_index];
        match opt.as_str() {
            "-r" => {
                root_dir = Some(args[opt_index + 1].clone());
                opt_index += 2;
            }
            "-w" => {
                work_dir = Some(args[opt_index + 1].clone());
                opt_index += 2;
            }
            _ => {
                exit_err!("Wrong option: {}", opt);
            }
        }
    }

    if let Some(root_dir) = root_dir {
        if unsafe { libc::chroot(root_dir.as_ptr() as *const i8) } < 0 {
            exit_err!("chroot()");
        }
    }

    if let Some(work_dir) = work_dir {
        if unsafe { libc::chdir(work_dir.as_ptr() as *const i8) } < 0 {
            exit_err!("chdir()");
        }
    }

    let list = get_cwd();
    println!("{}", list);
}