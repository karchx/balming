use core::mem::MaybeUninit;
use std::ffi::CStr;

pub struct LocalTime {
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub wday: u8, // 0 = Sunday
    pub mday: u8,
}

pub fn set_timezone(posix_tx: &CStr) {
    unsafe {
        esp_idf_svc::sys::setenv(c"TZ".as_ptr(), posix_tx.as_ptr(), 1);
        esp_idf_svc::sys::tzset();
    }
}

pub fn new_local() -> LocalTime {
    unsafe {
        let mut now: esp_idf_svc::sys::time_t = 0;
        esp_idf_svc::sys::time(&mut now);

        let mut tm = MaybeUninit::<esp_idf_svc::sys::tm>::zeroed();
        esp_idf_svc::sys::localtime_r(&now, tm.as_mut_ptr());
        let tm = tm.assume_init();

        LocalTime {
            hour: tm.tm_hour as u8,
            min: tm.tm_min as u8,
            sec: tm.tm_sec as u8,
            wday: tm.tm_wday as u8,
            mday: tm.tm_mday as u8,
        }
    }
}
