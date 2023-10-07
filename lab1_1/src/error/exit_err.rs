#[macro_export]
macro_rules! exit_err {
    () => { compile_error!("Specify first argument"); };
    ($($arg:tt)*) => {{
        use std::io::{self, Write};
        let error = io::Error::last_os_error();
        _ = io::stdout().flush();
        eprint!("FAIL: ");
        eprint!($($arg)*);
        eprintln!(": {}", error);
        std::process::exit(1);
    }};
}

#[macro_export]
macro_rules! exit_errx {
    () => { compile_error!("Specify first argument"); };
    ($($arg:tt)*) => {{
        use std::io::{self, Write};
        _ = io::stdout().flush();
        eprint!("FAIL: ");
        eprint!($($arg)*);
        eprintln!();
    }};
}

#[macro_export]
macro_rules! warn_err {
    () => { compile_error!("Specify first argument"); };
    ($($arg:tt)*) => {{
        use std::io::{self, Write};
        let error = io::Error::last_os_error();
        _ = io::stdout().flush();
        eprint!("WARN: ");
        eprint!($($arg)*);
        eprintln!(": {}", error);
    }};
}

#[macro_export]
macro_rules! warn_errx {
    () => { compile_error!("Specify first argument"); };
    ($($arg:tt)*) => {{
        use std::io::{self, Write};
        _ = io::stdout().flush();
        eprint!("WARN: ");
        eprint!($($arg)*);
        eprintln!();
    }};
}