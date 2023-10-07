#[macro_export]
macro_rules! offset_of {
    ($type:ty, $field:ident) => {{
        use std::mem::{MaybeUninit};
        use std::ptr;
        let data = MaybeUninit::<$type>::uninit();

        ptr::addr_of!((*data.as_ptr()).$field).cast::<u8>()
            .offset_from(data.as_ptr().cast::<u8>()) as usize
    }};
}

#[macro_export]
macro_rules! field_ptr {
    ($ptr:expr, $type:ty, $field:ident) => {{
        use std::ptr;

        if true {
            $ptr.cast::<u8>().add(crate::offset_of!($type, $field)).cast()
        } else {
            #[allow(deref_nullptr)]
            { ptr::addr_of!((*ptr::null::<$type>()).$field) }
        }
    }};
}