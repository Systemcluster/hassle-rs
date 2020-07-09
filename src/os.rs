#[cfg(windows)]
mod os_defs {
    pub use winapi::shared::{
        ntdef::{HRESULT, LPCSTR, LPCWSTR, LPSTR, LPWSTR, WCHAR},
        wtypes::BSTR,
    };
}

#[cfg(not(windows))]
mod os_defs {
    pub type CHAR = std::os::raw::c_char;
    pub type WCHAR = u32;
    pub type OLECHAR = WCHAR;
    pub type LPSTR = *mut CHAR;
    pub type LPWSTR = *mut WCHAR;
    pub type LPCSTR = *const CHAR;
    pub type LPCWSTR = *const WCHAR;
    pub type BSTR = *mut OLECHAR;
    pub type LPBSTR = *mut BSTR;
    pub type HRESULT = i32;
}

pub use os_defs::*;

pub struct HRESULT(pub os_defs::HRESULT);
impl HRESULT {
    pub fn is_err(&self) -> bool {
        self.0 < 0
    }
}

impl Into<HRESULT> for i32 {
    fn into(self) -> HRESULT {
        HRESULT(self)
    }
}

impl std::fmt::Debug for HRESULT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(&self, f)
    }
}

impl std::fmt::Display for HRESULT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self))
    }
}

impl std::fmt::LowerHex for HRESULT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = if f.alternate() { "0x" } else { "" };
        let bare_hex = format!("{:x}", self.0.abs());
        // https://stackoverflow.com/a/44712309
        f.pad_integral(self.0 >= 0, prefix, &bare_hex)
        // <i32 as std::fmt::LowerHex>::fmt(&self.0, f)
    }
}
