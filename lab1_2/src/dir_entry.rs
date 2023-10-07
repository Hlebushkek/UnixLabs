use std::collections::LinkedList;
use std::ffi::{CString, OsString, CStr};
use std::io::{self};
use std::clone::Clone;
use std::os::unix::prelude::OsStrExt;
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;

use crate::exit_err;

pub(crate) static mut STRICT_LOOP: bool = true;
pub(crate) static mut FOLLOW_SIMLINK: bool = false;
pub(crate) static mut OPEN_MODE: libc::mode_t = 0;

#[derive(Debug, Clone)]
pub struct DirEntry {
    name: OsString,
    parent: Option<Rc<RefCell<DirEntry>>>,
    entries: LinkedList<Rc<RefCell<DirEntry>>>,
    dev: libc::dev_t,
    ino: libc::ino_t,
    loop_flag: bool,
    cycle_flag: bool
}

impl DirEntry {
    pub fn new(name: OsString, parent: Option<Rc<RefCell<DirEntry>>>) -> Self {
        DirEntry {
            name,
            parent,
            entries: LinkedList::new(),
            dev: 0,
            ino: 0,
            loop_flag: false,
            cycle_flag: false,
        }
    }
}

fn fd(root_entry: Rc<RefCell<DirEntry>>) -> io::Result<libc::c_int> {
    let mut path = LinkedList::new();
    let (mut fd1, mut fd2);

    path.push_front(Rc::clone(&root_entry));
    let mut entry = (root_entry).borrow().parent.clone();
    loop {
        match entry {
            Some(strong_entry) => {
                path.push_front(Rc::clone(&strong_entry));
                entry = strong_entry.borrow().parent.clone();
            }
            None => { break; }
        }
    }

    fd1 = libc::AT_FDCWD;

    for entry in path.iter() {
        let entry_name = CString::new(entry.borrow().name.clone().to_string_lossy().as_bytes()).unwrap();
        let flags = (if fd1 == libc::AT_FDCWD { libc::O_NOFOLLOW } else { 0 }) as libc::mode_t;
        fd2 = unsafe { libc::openat(fd1, entry_name.as_ptr(), (OPEN_MODE & !flags) as i32) };
        
        if fd2 < 0 {
            exit_err!("DirEntry::fd(): openat()");
        }
        if fd1 != libc::AT_FDCWD && unsafe { libc::close(fd1) < 0 } {
            exit_err!("DirEntry::fd(): close()");
        }
        fd1 = fd2;
    }

    Ok(fd1)
}

pub fn walk(root_entry: Rc<RefCell<DirEntry>>) -> io::Result<()> {
    let mut statbuf: libc::stat = unsafe { std::mem::zeroed() };
    let mut fd1 = fd(Rc::clone(&root_entry))?;

    let mut entry = root_entry.borrow().parent.clone();
    loop {
        match entry {
            Some(strong_entry) => {
                if root_entry.borrow().dev == strong_entry.borrow().dev &&
                   root_entry.borrow().ino == strong_entry.borrow().ino {
                    root_entry.borrow_mut().loop_flag = true;
                    return Ok(());
                }
                entry = strong_entry.borrow().parent.clone();
            }
            None => { break; }
        }
    }
    
    if root_entry.borrow().parent.is_none() {
        if unsafe { libc::fstat(fd1, &mut statbuf) } < 0 {
            exit_err!("DirEntry::walk(): fstat()");
        }
        root_entry.borrow_mut().dev = statbuf.st_dev;
        root_entry.borrow_mut().ino = statbuf.st_ino;
    }

    let dir = unsafe { libc::fdopendir(fd1) };
    if dir.is_null() {
        exit_err!("DirEntry::walk(): fdopendir()");
    }
    
    let mut errno = 0;
    let mut ent_ptr: *mut libc::dirent = ptr::null_mut();
    loop {
        ent_ptr = unsafe { libc::readdir(dir) };
        if ent_ptr.is_null() {
            errno = unsafe { *libc::__error() };
            if errno != 0 {
                exit_err!("DirEntry::walk(): readdir()");
            }
            break;
        }
        
        // let entry_name = unsafe { CStr::from_ptr((*ent_ptr).d_name.as_ptr() as *const libc::c_char).to_str().unwrap() };
        let entry_name = unsafe { CStr::from_ptr(crate::field_ptr!(ent_ptr, libc::dirent, d_name).cast()).to_str().unwrap() };
        
        if entry_name.chars().nth(0) != Some('.') || (entry_name.chars().nth(1) != None && 
          (entry_name.chars().nth(1) != Some('.') ||  entry_name.chars().nth(2) != None)) {
            let entry =  DirEntry::new(OsString::from(entry_name), Some(Rc::clone(&root_entry)));
            root_entry.borrow_mut().entries.push_back(Rc::new(RefCell::new(entry)));
        }
    }

    if unsafe { libc::closedir(dir) } < 0 {
        exit_err!("walk(): closedir()");
    }

    fd1 = -1;
    for entry in root_entry.borrow().entries.iter() {
        if fd1 < 0 { fd1 = fd(Rc::clone(&root_entry))?; }

        let entry_name = CString::new(entry.borrow().name.as_bytes()).unwrap();
        let flag = if unsafe { FOLLOW_SIMLINK } { 0 } else { libc::AT_SYMLINK_NOFOLLOW };
        if unsafe { libc::fstatat(fd1,  entry_name.as_ptr(), &mut statbuf, flag) } < 0 {
            errno = unsafe { *libc::__error() };
            if errno == libc::ENOENT {
                continue;
            }
            if errno == libc::ELOOP {
                root_entry.borrow_mut().cycle_flag = true;
                continue;
            }
            exit_err!("DirEntry::walk(): fstatat() 1");
        }

        let is_dir = (statbuf.st_mode & libc::S_IFMT) == libc::S_IFDIR;
        if unsafe { is_dir && FOLLOW_SIMLINK && !STRICT_LOOP &&
            libc::fstatat(fd1, entry_name.as_ptr(), &mut statbuf, libc::AT_SYMLINK_NOFOLLOW) < 0 } {
            exit_err!("DirEntry::walk(): fstatat() 2");
        }

        entry.borrow_mut().dev = statbuf.st_dev;
        entry.borrow_mut().ino = statbuf.st_ino;
        if is_dir {
            if unsafe { libc::close(fd1) < 0 } {
                exit_err!("DirEntry::walk(): close()");
            }

            fd1 = -1;
            walk(Rc::clone(entry))?;
        }
    }

    if fd1 >= 0 && unsafe { libc::close(fd1) < 0 } {
        exit_err!("DirEntry::walk(): close()");
    }

    Ok(())
}

pub fn show(root_entry: Rc<RefCell<DirEntry>>, level: usize) {
    for entry in root_entry.borrow().entries.iter() {
        print!("{:width$}", "", width = level);
        println!(
            "{}{}{}",
            entry.borrow().name.to_string_lossy(),
            if entry.borrow().loop_flag { " (loop)" } else { "" },
            if entry.borrow().cycle_flag { " (cycle)" } else { "" }
        );
        
        show(Rc::clone(entry), level + 2)
    }
}