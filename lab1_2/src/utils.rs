pub mod offsets;
pub mod error;

use std::ffi::CString;
fn str_to_cstring(s: &str) -> CString {
    let mut v = Vec::<u8>::new();
    if let Err(e) = v.try_reserve_exact(s.len() + 1) {
        crate::exit_errx!("Vec::try_reserve_exact(): {e}");
    }

    v.extend_from_slice(s.as_bytes());
    unsafe { CString::from_vec_unchecked(v) }
}