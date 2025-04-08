use std::ffi::CString;

extern "C" {
    fn tzset();
}

pub fn set_timezone(tz_str: &str) {
    let name = CString::new("TZ").unwrap();
    let value = CString::new(tz_str).unwrap();

    #[allow(unsafe_code)]
    unsafe {
        // overwrite existing timezone
        libc::setenv(name.as_ptr(), value.as_ptr(), 1);
        tzset();
    }
}
