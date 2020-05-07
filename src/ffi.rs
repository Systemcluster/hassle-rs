#![allow(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::too_many_arguments)]

use crate::os::{HRESULT, LPCWSTR, LPWSTR};
pub(crate) use crate::unknown::IDxcUnknownShim;
use com::{com_interface, ComPtr, IID};
use std::ffi::c_void;

pub type DxcCreateInstanceProc =
    extern "system" fn(rclsid: &IID, riid: &IID, ppv: *mut *mut c_void) -> HRESULT;

pub type DxcCreateInstanceProc2 = extern "system" fn(
    malloc: /* IMalloc */ *const c_void,
    rclsid: &IID,
    riid: &IID,
    ppv: *mut *mut c_void,
) -> HRESULT;

#[com_interface("8BA5FB08-5195-40e2-AC58-0D989C3A0102")]
pub trait IDxcBlob: IDxcUnknownShim {
    unsafe fn get_buffer_pointer(&self) -> *mut c_void;
    unsafe fn get_buffer_size(&self) -> usize;
}

#[com_interface("7241d424-2646-4191-97c0-98e96e42fc68")]
pub trait IDxcBlobEncoding: IDxcBlob {
    unsafe fn get_encoding(&self, known: *mut u32, code_page: *mut u32) -> HRESULT;
}

#[com_interface("e5204dc7-d18c-4c3c-bdfb-851673980fe7")]
pub trait IDxcLibrary: IDxcUnknownShim {
    unsafe fn set_malloc(&self, malloc: *const c_void) -> HRESULT;
    unsafe fn create_blob_from_blob(
        &self,
        blob: ComPtr<dyn IDxcBlob>,
        offset: u32,
        length: u32,
        result_blob: *mut Option<ComPtr<dyn IDxcBlob>>,
    ) -> HRESULT;
    unsafe fn create_blob_from_file(
        &self,
        filename: LPCWSTR,
        code_page: *const u32,
        blob_encoding: *mut Option<ComPtr<dyn IDxcBlobEncoding>>,
    ) -> HRESULT;
    unsafe fn create_blob_with_encoding_from_pinned(
        &self,
        text: *const c_void,
        size: u32,
        code_page: u32,
        blob_encoding: *mut Option<ComPtr<dyn IDxcBlobEncoding>>,
    ) -> HRESULT;
    unsafe fn create_blob_with_encoding_on_heap_copy(
        &self,
        text: *const c_void,
        size: u32,
        code_page: u32,
        blob_encoding: *mut Option<ComPtr<dyn IDxcBlobEncoding>>,
    ) -> HRESULT;
    unsafe fn create_blob_with_encoding_on_malloc(
        &self,
        text: *const c_void,
        malloc: /* IMalloc */ *const c_void,
        size: u32,
        code_page: u32,
        blob_encoding: *mut Option<ComPtr<dyn IDxcBlobEncoding>>,
    ) -> HRESULT;
    unsafe fn create_include_handler(
        &self,
        include_handler: *mut Option<ComPtr<dyn IDxcIncludeHandler>>,
    ) -> HRESULT;
    unsafe fn create_stream_from_blob_read_only(
        &self,
        blob: ComPtr<dyn IDxcBlob>,
        stream: /* IStream */ *mut *mut c_void,
    ) -> HRESULT;
    unsafe fn get_blob_as_utf8(
        &self,
        blob: ComPtr<dyn IDxcBlob>,
        blob_encoding: *mut Option<ComPtr<dyn IDxcBlobEncoding>>,
    ) -> HRESULT;
    unsafe fn get_blob_as_utf16(
        &self,
        blob: ComPtr<dyn IDxcBlob>,
        blob_encoding: *mut Option<ComPtr<dyn IDxcBlobEncoding>>,
    ) -> HRESULT;
}

#[com_interface("CEDB484A-D4E9-445A-B991-CA21CA157DC2")]
pub trait IDxcOperationResult: IDxcUnknownShim {
    unsafe fn get_status(&self, status: *mut u32) -> HRESULT;
    unsafe fn get_result(&self, result: *mut Option<ComPtr<dyn IDxcBlob>>) -> HRESULT;
    unsafe fn get_error_buffer(&self, errors: *mut Option<ComPtr<dyn IDxcBlobEncoding>>)
        -> HRESULT;
}

#[com_interface("7f61fc7d-950d-467f-b3e3-3c02fb49187c")]
pub trait IDxcIncludeHandler: IDxcUnknownShim {
    unsafe fn load_source(
        &self,
        filename: LPCWSTR,
        include_source: *mut Option<ComPtr<dyn IDxcBlob>>,
    ) -> HRESULT;
}

#[repr(C)]
#[derive(Debug)]
pub struct DxcDefine {
    pub name: LPCWSTR,
    pub value: LPCWSTR,
}

#[com_interface("8c210bf3-011f-4422-8d70-6f9acb8db617")]
pub trait IDxcCompiler: IDxcUnknownShim {
    unsafe fn compile(
        &self,
        blob: ComPtr<dyn IDxcBlob>,
        source_name: LPCWSTR,
        entry_point: LPCWSTR,
        target_profile: LPCWSTR,
        arguments: *const LPCWSTR,
        arg_count: u32,
        defines: *const DxcDefine,
        def_count: u32,
        include_handler: *const dyn IDxcIncludeHandler,
        result: *mut Option<ComPtr<dyn IDxcOperationResult>>,
    ) -> HRESULT;

    unsafe fn preprocess(
        &self,
        blob: ComPtr<dyn IDxcBlob>,
        source_name: LPCWSTR,
        arguments: *const LPCWSTR,
        arg_count: u32,
        defines: *const DxcDefine,
        def_count: u32,
        include_handler: *const dyn IDxcIncludeHandler,
        result: *mut Option<ComPtr<dyn IDxcOperationResult>>,
    ) -> HRESULT;

    unsafe fn disassemble(
        &self,
        blob: ComPtr<dyn IDxcBlob>,
        disassembly: *mut Option<ComPtr<dyn IDxcBlobEncoding>>,
    ) -> HRESULT;
}

#[com_interface("A005A9D9-B8BB-4594-B5C9-0E633BEC4D37")]
pub trait IDxcCompiler2: IDxcCompiler {
    unsafe fn compile_with_debug(
        &self,
        blob: ComPtr<dyn IDxcBlob>,
        source_name: LPCWSTR,
        entry_point: LPCWSTR,
        target_profile: LPCWSTR,
        arguments: *const LPCWSTR,
        arg_count: u32,
        defines: *const DxcDefine,
        def_count: u32,
        include_handler: *const dyn IDxcIncludeHandler,
        result: *mut Option<ComPtr<dyn IDxcOperationResult>>,
        debug_blob_name: *mut LPWSTR,
        debug_blob: *mut Option<ComPtr<dyn IDxcBlob>>,
    ) -> HRESULT;
}

#[com_interface("F1B5BE2A-62DD-4327-A1C2-42AC1E1E78E6")]
pub trait IDxcLinker: IDxcUnknownShim {
    unsafe fn register_library(&self, lib_name: LPCWSTR, lib: ComPtr<dyn IDxcBlob>) -> HRESULT;

    unsafe fn link(
        &self,
        entry_name: LPCWSTR,
        target_profile: LPCWSTR,
        lib_names: *const LPCWSTR,
        lib_count: u32,
        arguments: *const LPCWSTR,
        arg_count: u32,
        result: *mut Option<ComPtr<dyn IDxcOperationResult>>,
    ) -> HRESULT;
}

pub const DXC_VALIDATOR_FLAGS_DEFAULT: u32 = 0;
pub const DXC_VALIDATOR_FLAGS_IN_PLACE_EDIT: u32 = 1; // Validator is allowed to update shader blob in-place.
pub const DXC_VALIDATOR_FLAGS_ROOT_SIGNATURE_ONLY: u32 = 2;
pub const DXC_VALIDATOR_FLAGS_MODULE_ONLY: u32 = 4;
pub const DXC_VALIDATOR_FLAGS_VALID_MASK: u32 = 0x7;

#[com_interface("A6E82BD2-1FD7-4826-9811-2857E797F49A")]
pub trait IDxcValidator: IDxcUnknownShim {
    unsafe fn validate(
        &self,
        shader: ComPtr<dyn IDxcBlob>,
        flags: u32,
        result: *mut Option<ComPtr<dyn IDxcOperationResult>>,
    ) -> HRESULT;
}

#[com_interface("334b1f50-2292-4b35-99a1-25588d8c17fe")]
pub trait IDxcContainerBuilder: IDxcUnknownShim {
    unsafe fn load(&self, dxil_container_header: ComPtr<dyn IDxcBlob>) -> HRESULT;
    unsafe fn add_part(&self, four_cc: u32, source: ComPtr<dyn IDxcBlob>) -> HRESULT;
    unsafe fn remove_part(&self, four_cc: u32) -> HRESULT;
    unsafe fn seralize_container(
        &self,
        result: *mut Option<ComPtr<dyn IDxcOperationResult>>,
    ) -> HRESULT;
}

#[com_interface("091f7a26-1c1f-4948-904b-e6e3a8a771d5")]
pub trait IDxcAssembler: IDxcUnknownShim {
    unsafe fn assemble_to_container(
        &self,
        shader: ComPtr<dyn IDxcBlob>,
        result: *mut Option<ComPtr<dyn IDxcOperationResult>>,
    ) -> HRESULT;
}

#[com_interface("d2c21b26-8350-4bdc-976a-331ce6f4c54c")]
pub trait IDxcContainerReflection: IDxcUnknownShim {
    unsafe fn load(&self, container: ComPtr<dyn IDxcBlob>) -> HRESULT;
    unsafe fn get_part_count(&self, result: *mut u32) -> HRESULT;
    unsafe fn get_part_kind(&self, idx: u32, result: *mut u32) -> HRESULT;
    unsafe fn get_part_content(&self, idx: u32, result: *mut *mut dyn IDxcBlob) -> HRESULT;
    unsafe fn find_first_part_kind(&self, kind: u32, result: *mut u32) -> HRESULT;
    unsafe fn get_part_reflection(
        &self,
        idx: u32,
        iid: *const IID,
        object: *mut *mut c_void,
    ) -> HRESULT;
}

#[com_interface("AE2CD79F-CC22-453F-9B6B-B124E7A5204C")]
pub trait IDxcOptimizerPass: IDxcUnknownShim {
    unsafe fn get_option_name(&self, result: *mut LPWSTR) -> HRESULT;
    unsafe fn get_description(&self, result: *mut LPWSTR) -> HRESULT;
    unsafe fn get_option_arg_count(&self, count: *mut u32) -> HRESULT;
    unsafe fn get_option_arg_name(&self, arg_idx: u32, result: *mut LPWSTR) -> HRESULT;
    unsafe fn get_option_arg_description(&self, arg_idx: u32, result: *mut LPWSTR) -> HRESULT;
}

#[com_interface("25740E2E-9CBA-401B-9119-4FB42F39F270")]
pub trait IDxcOptimizer: IDxcUnknownShim {
    unsafe fn get_available_pass_count(&self, count: *mut u32) -> HRESULT;
    unsafe fn get_available_pass(
        &self,
        index: u32,
        result: *mut Option<ComPtr<dyn IDxcOptimizerPass>>,
    ) -> HRESULT;
    unsafe fn run_optimizer(
        &self,
        blob: ComPtr<dyn IDxcBlob>,
        options: *const LPCWSTR,
        option_count: u32,
        output_module: *mut Option<ComPtr<dyn IDxcBlob>>,
        output_text: *mut Option<ComPtr<dyn IDxcBlobEncoding>>,
    ) -> HRESULT;
}

pub const DXC_VERSION_INFO_FLAGS_NONE: u32 = 0;
pub const DXC_VERSION_INFO_FLAGS_DEBUG: u32 = 1; // Matches VS_FF_DEBUG
pub const DXC_VERSION_INFO_FLAGS_INTERNAL: u32 = 2; // Internal Validator (non-signing)

#[com_interface("b04f5b50-2059-4f12-a8ff-a1e0cde1cc7e")]
pub trait IDxcVersionInfo: IDxcUnknownShim {
    unsafe fn get_version(&self, major: *mut u32, minor: *mut u32) -> HRESULT;
    unsafe fn get_flags(&self, flags: *mut u32) -> HRESULT;
}

#[com_interface("fb6904c4-42f0-4b62-9c46-983af7da7c83")]
pub trait IDxcVersionInfo2: IDxcUnknownShim {
    unsafe fn get_commit_info(&self, commit_count: *mut u32, commit_hash: *mut *mut u8) -> HRESULT;
}

pub const CLSID_DxcCompiler: IID = IID {
    data1: 0x73e22d93,
    data2: 0xe6ce,
    data3: 0x47f3,
    data4: [0xb5, 0xbf, 0xf0, 0x66, 0x4f, 0x39, 0xc1, 0xb0],
};
pub const CLSID_DxcLinker: IID = IID {
    data1: 0xef6a8087,
    data2: 0xb0ea,
    data3: 0x4d56,
    data4: [0x9e, 0x45, 0xd0, 0x7e, 0x1a, 0x8b, 0x78, 0x6],
};
pub const CLSID_DxcDiaDataSource: IID = IID {
    data1: 0xcd1f6b73,
    data2: 0x2ab0,
    data3: 0x484d,
    data4: [0x8e, 0xdc, 0xeb, 0xe7, 0xa4, 0x3c, 0xa0, 0x9f],
};
pub const CLSID_DxcLibrary: IID = IID {
    data1: 0x6245d6af,
    data2: 0x66e0,
    data3: 0x48fd,
    data4: [0x80, 0xb4, 0x4d, 0x27, 0x17, 0x96, 0x74, 0x8c],
};
pub const CLSID_DxcValidator: IID = IID {
    data1: 0x8ca3e215,
    data2: 0xf728,
    data3: 0x4cf3,
    data4: [0x8c, 0xdd, 0x88, 0xaf, 0x91, 0x75, 0x87, 0xa1],
};
pub const CLSID_DxcAssembler: IID = IID {
    data1: 0xd728db68,
    data2: 0xf903,
    data3: 0x4f80,
    data4: [0x94, 0xcd, 0xdc, 0xcf, 0x76, 0xec, 0x71, 0x51],
};
pub const CLSID_DxcContainerReflection: IID = IID {
    data1: 0xb9f54489,
    data2: 0x55b8,
    data3: 0x400c,
    data4: [0xba, 0x3a, 0x16, 0x75, 0xe4, 0x72, 0x8b, 0x91],
};
pub const CLSID_DxcOptimizer: IID = IID {
    data1: 0xae2cd79f,
    data2: 0xcc22,
    data3: 0x453f,
    data4: [0x9b, 0x6b, 0xb1, 0x24, 0xe7, 0xa5, 0x20, 0x4c],
};
pub const CLSID_DxcContainerBuilder: IID = IID {
    data1: 0x94134294,
    data2: 0x411f,
    data3: 0x4574,
    data4: [0xb4, 0xd0, 0x87, 0x41, 0xe2, 0x52, 0x40, 0xd2],
};
