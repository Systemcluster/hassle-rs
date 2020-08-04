#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    clippy::type_complexity
)]

use crate::ffi::*;
use crate::os::{HRESULT, LPCWSTR, LPWSTR, WCHAR};
use crate::utils::{from_wide, to_wide, HassleError};
use com::{class, Interface};
use libloading::{Library, Symbol};
use std::cell::RefCell;
use std::convert::Into;
use std::ffi::c_void;
use std::rc::Rc;

#[macro_export]
macro_rules! return_hr {
    ($hr:expr, $v: expr) => {
        let hr = $hr;
        if !hr.is_err() {
            return Ok($v);
        } else {
            println!("Failed HRESULT: {}", hr);
            return Err(hr);
        }
    };
}

macro_rules! return_hr_wrapped {
    ($hr:expr, $v: expr) => {
        let hr = $hr;
        if !hr.is_err() {
            return Ok($v);
        } else {
            return Err(HassleError::Win32Error(hr));
        }
    };
}

// #[derive(Debug)]
pub struct DxcBlob {
    inner: IDxcBlob,
}

impl DxcBlob {
    fn new(inner: IDxcBlob) -> Self {
        Self { inner }
    }

    pub fn to_vec<T>(&self) -> Vec<T>
    where
        T: Clone,
    {
        let slice = unsafe {
            std::slice::from_raw_parts(
                self.inner.get_buffer_pointer() as *const T,
                self.inner.get_buffer_size() / std::mem::size_of::<T>(),
            )
        };

        slice.to_vec()
    }
}

// #[derive(Debug)]
pub struct DxcBlobEncoding {
    inner: IDxcBlobEncoding,
}

impl DxcBlobEncoding {
    fn new(inner: IDxcBlobEncoding) -> Self {
        Self { inner }
    }
}

impl Into<DxcBlob> for DxcBlobEncoding {
    fn into(self) -> DxcBlob {
        // TODO: Refcounted ComRc!
        DxcBlob::new(self.inner.get_interface::<IDxcBlob>().unwrap())
    }
}

// #[derive(Debug)]
pub struct DxcOperationResult {
    inner: IDxcOperationResult,
}

impl DxcOperationResult {
    fn new(inner: IDxcOperationResult) -> Self {
        Self { inner }
    }

    pub fn get_status(&self) -> Result<u32, HRESULT> {
        let mut status: u32 = 0;
        return_hr!(unsafe { self.inner.get_status(&mut status) }, status);
    }

    pub fn get_result(&self) -> Result<DxcBlob, HRESULT> {
        let mut blob = None;
        return_hr!(
            unsafe { self.inner.get_result(&mut blob) },
            DxcBlob::new(blob.unwrap())
        );
    }

    pub fn get_error_buffer(&self) -> Result<DxcBlobEncoding, HRESULT> {
        let mut blob = None;
        return_hr!(
            unsafe { self.inner.get_error_buffer(&mut blob) },
            DxcBlobEncoding::new(blob.unwrap())
        );
    }
}

pub trait DxcIncludeHandler {
    fn load_source(&self, filename: String) -> Option<String>;
}

class! {
    class DxcIncludeHandlerWrapper: IDxcIncludeHandler(IDxcUnknownShim) {
        handler: Box<dyn DxcIncludeHandler>,
        blobs: RefCell<Vec<DxcBlobEncoding>>,
        pinned: RefCell<Vec<Rc<String>>>,
        library: DxcLibrary,
    }

    impl IDxcUnknownShim for DxcIncludeHandlerWrapper {
        #[cfg(not(windows))]
        fn complete_object_destructor(&self) -> HRESULT {
            HRESULT(0)
        }
        #[cfg(not(windows))]
        fn deleting_destructor(&self) -> HRESULT {
            HRESULT(0)
        }
    }

    impl IDxcIncludeHandler for DxcIncludeHandlerWrapper {
        fn load_source(
            &self,
            filename: LPCWSTR,
            include_source: *mut Option<IDxcBlob>,
        ) -> HRESULT {
            let filename = crate::utils::from_wide(filename as *mut _);

            let source = self.handler.load_source(filename);

            if let Some(source) = source {
                let pinned_source = Rc::new(source.clone());

                let blob = self
                    .library
                    .create_blob_with_encoding_from_str(&*pinned_source)
                    .unwrap();

                unsafe { *include_source = blob.inner.get_interface::<IDxcBlob>() };
                self.blobs.borrow_mut().push(blob);
                self.pinned.borrow_mut().push(Rc::clone(&pinned_source));

                // NOERROR
                0
            } else {
                -2_147_024_894 // ERROR_FILE_NOT_FOUND / 0x80070002
            }
            .into()
        }
    }
}

impl Default for DxcIncludeHandlerWrapper {
    // Required for com-rs, even if we never want it to create an instance for us.
    fn default() -> Self {
        unreachable!("Should never create an empty DxcIncludeHandlerWrapper")
    }
}

impl DxcIncludeHandlerWrapper {
    fn create_include_handler(
        library: &DxcLibrary,
        include_handler: Box<dyn DxcIncludeHandler>,
    ) -> Self {
        Self::new(
            include_handler,
            RefCell::new(vec![]),
            RefCell::new(vec![]),
            library.clone(),
        )
    }

    pub fn get_interface<I: Interface>(&self) -> Option<I> {
        use com::sys::{E_NOINTERFACE, E_POINTER, FAILED};
        use com::IID;
        let mut ppv = None;
        let hr = unsafe {
            self.query_interface(
                &I::IID as *const IID,
                &mut ppv as *mut _ as *mut *mut c_void,
            )
        };
        if FAILED(hr) {
            assert!(
                hr == E_NOINTERFACE || hr == E_POINTER,
                "QueryInterface returned non-standard error"
            );
            return None;
        }
        debug_assert!(ppv.is_some());
        ppv
    }
}

// use com::sys::{HRESULT, NOERROR};

// #[derive(Debug)]
pub struct DxcCompiler {
    inner: IDxcCompiler2,
    library: DxcLibrary,
}

impl DxcCompiler {
    fn new(inner: IDxcCompiler2, library: DxcLibrary) -> Self {
        Self { inner, library }
    }

    fn prep_defines(
        defines: &[(&str, Option<&str>)],
        wide_defines: &mut Vec<(Vec<WCHAR>, Vec<WCHAR>)>,
        dxc_defines: &mut Vec<DxcDefine>,
    ) {
        for (name, value) in defines {
            if value.is_none() {
                wide_defines.push((to_wide(name), to_wide("1")));
            } else {
                wide_defines.push((to_wide(name), to_wide(value.unwrap())));
            }
        }

        for (ref name, ref value) in wide_defines {
            dxc_defines.push(DxcDefine {
                name: name.as_ptr(),
                value: value.as_ptr(),
            });
        }
    }

    fn prep_args(args: &[&str], wide_args: &mut Vec<Vec<WCHAR>>, dxc_args: &mut Vec<LPCWSTR>) {
        for a in args {
            wide_args.push(to_wide(a));
        }

        for a in wide_args {
            dxc_args.push(a.as_ptr());
        }
    }

    fn prep_include_handler(
        library: &DxcLibrary,
        include_handler: Option<Box<dyn DxcIncludeHandler>>,
    ) -> Option<DxcIncludeHandlerWrapper> {
        include_handler.map(|include_handler| {
            DxcIncludeHandlerWrapper::create_include_handler(library, include_handler)
        })
    }

    pub fn compile(
        &self,
        blob: &DxcBlobEncoding,
        source_name: &str,
        entry_point: &str,
        target_profile: &str,
        args: &[&str],
        include_handler: Option<Box<dyn DxcIncludeHandler>>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<DxcOperationResult, (DxcOperationResult, HRESULT)> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(&args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(&defines, &mut wide_defines, &mut dxc_defines);

        let handler_wrapper = Self::prep_include_handler(&self.library, include_handler);
        let h = handler_wrapper.map(|hnd| hnd.get_interface::<IDxcIncludeHandler>().unwrap()).unwrap();

        let mut result = None;
        let result_hr = unsafe {
            self.inner.compile(
                &blob.inner,
                to_wide(source_name).as_ptr(),
                to_wide(entry_point).as_ptr(),
                to_wide(target_profile).as_ptr(),
                dxc_args.as_ptr(),
                dxc_args.len() as u32,
                dxc_defines.as_ptr(),
                dxc_defines.len() as u32,
                Some(h),
                &mut result,
            )
        };

        let result = result.unwrap();

        let mut compile_error = 0u32;
        unsafe {
            result.get_status(&mut compile_error);
        }

        if !result_hr.is_err() && compile_error == 0 {
            Ok(DxcOperationResult::new(result))
        } else {
            Err((DxcOperationResult::new(result), result_hr))
        }
    }

    pub fn compile_with_debug(
        &self,
        blob: &DxcBlobEncoding,
        source_name: &str,
        entry_point: &str,
        target_profile: &str,
        args: &[&str],
        include_handler: Option<Box<dyn DxcIncludeHandler>>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<(DxcOperationResult, String, DxcBlob), (DxcOperationResult, HRESULT)> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(&args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(&defines, &mut wide_defines, &mut dxc_defines);

        let handler_wrapper = Self::prep_include_handler(&self.library, include_handler);

        let mut result = None;
        let mut debug_blob = None;
        let mut debug_filename: LPWSTR = std::ptr::null_mut();

        let result_hr = unsafe {
            self.inner.compile_with_debug(
                &blob.inner,
                to_wide(source_name).as_ptr(),
                to_wide(entry_point).as_ptr(),
                to_wide(target_profile).as_ptr(),
                dxc_args.as_ptr(),
                dxc_args.len() as u32,
                dxc_defines.as_ptr(),
                dxc_defines.len() as u32,
                handler_wrapper.map(|hnd| hnd.get_interface::<IDxcIncludeHandler>().unwrap()),
                &mut result,
                &mut debug_filename,
                &mut debug_blob,
            )
        };
        let result = result.unwrap();
        let debug_blob = debug_blob.unwrap();

        let mut compile_error = 0u32;
        unsafe {
            result.get_status(&mut compile_error);
        }

        if !result_hr.is_err() && compile_error == 0 {
            Ok((
                DxcOperationResult::new(result),
                from_wide(debug_filename),
                DxcBlob::new(debug_blob),
            ))
        } else {
            Err((DxcOperationResult::new(result), result_hr))
        }
    }

    pub fn preprocess(
        &self,
        blob: &DxcBlobEncoding,
        source_name: &str,
        args: &[&str],
        include_handler: Option<Box<dyn DxcIncludeHandler>>,
        defines: &[(&str, Option<&str>)],
    ) -> Result<DxcOperationResult, (DxcOperationResult, HRESULT)> {
        let mut wide_args = vec![];
        let mut dxc_args = vec![];
        Self::prep_args(&args, &mut wide_args, &mut dxc_args);

        let mut wide_defines = vec![];
        let mut dxc_defines = vec![];
        Self::prep_defines(&defines, &mut wide_defines, &mut dxc_defines);

        let handler_wrapper = Self::prep_include_handler(&self.library, include_handler);

        let mut result = None;
        let result_hr = unsafe {
            self.inner.preprocess(
                &blob.inner,
                to_wide(source_name).as_ptr(),
                dxc_args.as_ptr(),
                dxc_args.len() as u32,
                dxc_defines.as_ptr(),
                dxc_defines.len() as u32,
                handler_wrapper.map(|hnd| hnd.get_interface::<IDxcIncludeHandler>().unwrap()),
                &mut result,
            )
        };

        let result = result.unwrap();

        let mut compile_error = 0u32;
        unsafe {
            result.get_status(&mut compile_error);
        }
        if !result_hr.is_err() && compile_error == 0 {
            Ok(DxcOperationResult::new(result))
        } else {
            Err((DxcOperationResult::new(result), result_hr))
        }
    }

    pub fn disassemble(&self, blob: &DxcBlob) -> Result<DxcBlobEncoding, HRESULT> {
        let mut result_blob = None;

        return_hr!(
            unsafe { self.inner.disassemble(&blob.inner, &mut result_blob,) },
            DxcBlobEncoding::new(result_blob.unwrap())
        );
    }
}

// // #[derive(Debug)]
#[derive(Clone)]
pub struct DxcLibrary {
    inner: IDxcLibrary,
}

impl DxcLibrary {
    fn new(inner: IDxcLibrary) -> Self {
        Self { inner }
    }

    pub fn create_blob_with_encoding(&self, data: &[u8]) -> Result<DxcBlobEncoding, HRESULT> {
        let mut blob = None;
        return_hr!(
            unsafe {
                self.inner.create_blob_with_encoding_from_pinned(
                    data.as_ptr() as *const c_void,
                    data.len() as u32,
                    0, // Binary; no code page
                    &mut blob,
                )
            },
            DxcBlobEncoding::new(blob.unwrap())
        );
    }

    pub fn create_blob_with_encoding_from_str(
        &self,
        text: &str,
    ) -> Result<DxcBlobEncoding, HRESULT> {
        let mut blob = None;
        const CP_UTF8: u32 = 65001; // UTF-8 translation

        return_hr!(
            unsafe {
                self.inner.create_blob_with_encoding_from_pinned(
                    text.as_ptr() as *const c_void,
                    text.len() as u32,
                    CP_UTF8,
                    &mut blob,
                )
            },
            DxcBlobEncoding::new(blob.unwrap())
        );
    }

    pub fn get_blob_as_string(&self, blob: &DxcBlobEncoding) -> String {
        let mut blob_utf8 = None;

        unsafe { self.inner.get_blob_as_utf8(&blob.inner, &mut blob_utf8) };

        let blob_utf8 = blob_utf8.unwrap();

        let slice = unsafe {
            std::slice::from_raw_parts(
                blob_utf8.get_buffer_pointer() as *const u8,
                blob_utf8.get_buffer_size(),
            )
        };

        String::from_utf8(slice.to_vec()).unwrap()
    }
}

#[derive(Debug)]
pub struct Dxc {
    dxc_lib: Library,
}

#[cfg(target_os = "windows")]
fn dxcompiler_lib_name() -> &'static str {
    "dxcompiler.dll"
}

#[cfg(target_os = "linux")]
fn dxcompiler_lib_name() -> &'static str {
    "./libdxcompiler.so"
}

#[cfg(target_os = "macos")]
fn dxcompiler_lib_name() -> &'static str {
    "./libdxcompiler.dynlib"
}

impl Dxc {
    pub fn new() -> Result<Self, HassleError> {
        let lib_name = dxcompiler_lib_name();
        let dxc_lib = Library::new(lib_name).map_err(|e| HassleError::LoadLibraryError {
            filename: lib_name.to_string(),
            inner: e,
        })?;

        Ok(Self { dxc_lib })
    }

    pub(crate) fn get_dxc_create_instance<T>(
        &self,
    ) -> Result<Symbol<DxcCreateInstanceProc<T>>, HassleError> {
        Ok(unsafe { self.dxc_lib.get(b"DxcCreateInstance\0")? })
    }

    pub fn create_compiler(&self) -> Result<DxcCompiler, HassleError> {
        let mut compiler = None;
        return_hr_wrapped!(
            self.get_dxc_create_instance()?(
                &CLSID_DxcCompiler,
                &IID_IDXC_COMPILER2,
                // &IDxcCompiler2::IID,
                &mut compiler, /*  as *mut _ as *mut *mut _ */
            ),
            DxcCompiler::new(
                compiler.unwrap(),
                // TODO: ComRc::from_raw?
                self.create_library().unwrap()
            )
        );
    }

    pub fn create_library(&self) -> Result<DxcLibrary, HassleError> {
        let mut library = None;
        return_hr_wrapped!(
            self.get_dxc_create_instance()?(
                &CLSID_DxcLibrary,
                // &IID_IDXC_LIBRARY,
                &IDxcLibrary::IID,
                &mut library /*  as *mut _ as *mut *mut _ */
            ),
            DxcLibrary::new(library.unwrap())
        );
    }
}

// #[derive(Debug)]
pub struct DxcValidator {
    inner: IDxcValidator,
}

pub type DxcValidatorVersion = (u32, u32);

impl DxcValidator {
    fn new(inner: IDxcValidator) -> Self {
        Self { inner }
    }

    pub fn version(&self) -> Result<DxcValidatorVersion, /* TODO HassleError */ HRESULT> {
        // let mut version = std::ptr::null_mut();

        // let result_hr = unsafe {
        //     self.inner.query_interface(
        //         &IID_IDXC_VERSION_INFO,
        //         &mut version as *mut _ as *mut *mut _,
        //     )
        // };

        // let version = unsafe { ComPtr::<IDxcVersionInfo>::new(&mut version) };

        // if result_hr != 0 {
        //     return Err(result_hr);
        // }

        // TODO: Keep above code to get HRESULT? Update get_interface to return a Result<>??
        let version = self
            .inner
            .get_interface::<IDxcVersionInfo>()
            .ok_or(HRESULT(com::sys::E_NOINTERFACE))?;

        let mut major = 0;
        let mut minor = 0;

        return_hr! {
            unsafe { version.get_version(&mut major, &mut minor) },
            (major, minor)
        }
    }

    pub fn validate(&self, blob: DxcBlob) -> Result<DxcBlob, (DxcOperationResult, HRESULT)> {
        // let mut result = std::ptr::null_mut::<c_void>();
        let mut result = None;
        let result_hr = unsafe {
            self.inner
                .validate(&blob.inner, DXC_VALIDATOR_FLAGS_IN_PLACE_EDIT, &mut result)
        };

        let result = result.unwrap();

        let mut validate_status = 0u32;
        unsafe { result.get_status(&mut validate_status) };

        if !result_hr.is_err() && validate_status == 0 {
            Ok(blob)
        } else {
            Err((DxcOperationResult::new(result), result_hr))
        }
    }
}

#[derive(Debug)]
pub struct Dxil {
    dxil_lib: Library,
}

impl Dxil {
    pub fn new() -> Result<Self, HassleError> {
        if cfg!(windows) {
            Library::new("dxil.dll")
                .map_err(|e| HassleError::LoadLibraryError {
                    filename: "dxil".to_string(),
                    inner: e,
                })
                .map(|dxil_lib| Self { dxil_lib })
        } else {
            Err(HassleError::WindowsOnly(
                "DXIL Signing is only supported on windows at the moment".to_string(),
            ))
        }
    }

    fn get_dxc_create_instance<T>(&self) -> Result<Symbol<DxcCreateInstanceProc<T>>, HassleError> {
        Ok(unsafe { self.dxil_lib.get(b"DxcCreateInstance\0")? })
    }

    pub fn create_validator(&self) -> Result<DxcValidator, HassleError> {
        let mut validator = None;
        return_hr_wrapped!(
            self.get_dxc_create_instance()?(
                &CLSID_DxcValidator,
                &IID_IDXC_VALIDATOR,
                // &mut validator as *mut _ as *mut *mut _,
                &mut validator,
            ),
            DxcValidator::new(validator.unwrap())
        );
    }
}
