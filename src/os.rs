#[cfg(windows)]
mod os_defs {
    pub use winapi::shared::{
        ntdef::{HRESULT, LPCSTR, LPCWSTR, LPSTR, LPWSTR, WCHAR},
        wtypes::BSTR,
    };

    pub use winapi::um::combaseapi::CoTaskMemFree;
    pub use winapi::um::oleauto::SysFreeString;
}

#[cfg(not(windows))]
mod os_defs {
    pub type CHAR = i8;
    pub type WCHAR = u32;
    pub type OLECHAR = WCHAR;
    pub type LPSTR = *mut CHAR;
    pub type LPWSTR = *mut WCHAR;
    pub type LPCSTR = *const CHAR;
    pub type LPCWSTR = *const WCHAR;
    pub type BSTR = *mut OLECHAR;
    pub type LPBSTR = *mut BSTR;
    pub type HRESULT = i32;

    #[allow(non_snake_case)]
    pub unsafe fn CoTaskMemFree(p: *mut libc::c_void) {
        // https://github.com/microsoft/DirectXShaderCompiler/blob/a8d9780046cb64a1cea842fa6fc28a250e3e2c09/include/dxc/Support/WinAdapter.h#L46
        libc::free(p)
    }

    #[allow(non_snake_case)]
    pub unsafe fn SysFreeString(p: BSTR) {
        // https://github.com/microsoft/DirectXShaderCompiler/blob/a8d9780046cb64a1cea842fa6fc28a250e3e2c09/include/dxc/Support/WinAdapter.h#L48-L50
        libc::free(p as _)
    }
}

pub use os_defs::*;

#[must_use]
#[repr(transparent)]
pub struct HRESULT(pub os_defs::HRESULT);
impl HRESULT {
    pub fn is_err(&self) -> bool {
        self.0 < 0
    }
}

impl From<i32> for HRESULT {
    fn from(i: i32) -> Self {
        Self(i)
    }
}

impl std::fmt::Debug for HRESULT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(&self, f)
    }
}

impl std::fmt::Display for HRESULT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self)
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
