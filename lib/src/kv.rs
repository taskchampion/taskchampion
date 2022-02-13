use crate::traits::*;
use crate::types::*;

/// TCKV contains a key/value pair that is part of a task.
///
/// Neither key nor value are ever NULL.  They remain owned by the TCKV and
/// will be freed when it is freed with tc_kv_list_free.
#[repr(C)]
pub struct TCKV {
    pub key: *mut TCString<'static>,
    pub value: *mut TCString<'static>,
}

impl PassByValue for TCKV {
    type RustType = (TCString<'static>, TCString<'static>);

    unsafe fn from_ctype(self) -> Self::RustType {
        // SAFETY:
        //  - self is owned, so we can take ownership of this TCString
        //  - self.key is a valid, non-null TCString (see type docstring)
        let key = unsafe { TCString::take_from_ptr_arg(self.key) };
        // SAFETY: (same)
        let value = unsafe { TCString::take_from_ptr_arg(self.value) };
        (key, value)
    }

    fn as_ctype((key, value): Self::RustType) -> Self {
        TCKV {
            // SAFETY: caller assumes ownership of this value
            key: unsafe { key.return_ptr() },
            // SAFETY: caller assumes ownership of this value
            value: unsafe { value.return_ptr() },
        }
    }
}

/// TCKVList represents a list of key/value pairs.
///
/// The content of this struct must be treated as read-only.
#[repr(C)]
pub struct TCKVList {
    /// number of key/value pairs in items
    len: libc::size_t,

    /// total size of items (internal use only)
    _capacity: libc::size_t,

    /// array of TCKV's. these remain owned by the TCKVList instance and will be freed by
    /// tc_kv_list_free.  This pointer is never NULL for a valid TCKVList.
    items: *const TCKV,
}

impl CList for TCKVList {
    type Element = TCKV;

    unsafe fn from_raw_parts(items: *const Self::Element, len: usize, cap: usize) -> Self {
        TCKVList {
            len,
            _capacity: cap,
            items,
        }
    }

    fn into_raw_parts(self) -> (*const Self::Element, usize, usize) {
        (self.items, self.len, self._capacity)
    }
}

/// Free a TCKVList instance.  The instance, and all TCKVs it contains, must not be used after
/// this call.
///
/// When this call returns, the `items` pointer will be NULL, signalling an invalid TCKVList.
#[no_mangle]
pub unsafe extern "C" fn tc_kv_list_free(tckvs: *mut TCKVList) {
    // SAFETY:
    //  - tckvs is not NULL and points to a valid TCKVList (caller is not allowed to
    //    modify the list)
    //  - caller promises not to use the value after return
    unsafe { drop_value_list(tckvs) }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_list_has_non_null_pointer() {
        let tckvs = TCKVList::return_val(Vec::new());
        assert!(!tckvs.items.is_null());
        assert_eq!(tckvs.len, 0);
        assert_eq!(tckvs._capacity, 0);
    }

    #[test]
    fn free_sets_null_pointer() {
        let mut tckvs = TCKVList::return_val(Vec::new());
        // SAFETY: testing expected behavior
        unsafe { tc_kv_list_free(&mut tckvs) };
        assert!(tckvs.items.is_null());
        assert_eq!(tckvs.len, 0);
        assert_eq!(tckvs._capacity, 0);
    }
}