use crate::intellisense::ffi::*;
use crate::os::{BSTR, HRESULT, LPSTR};
use crate::utils::HassleError;
use crate::wrapper::Dxc;
use com::ComPtr;
use std::ffi::CString;

pub struct DxcIntellisense {
    inner: ComPtr<dyn IDxcIntelliSense>,
}

impl DxcIntellisense {
    fn new(inner: ComPtr<dyn IDxcIntelliSense>) -> Self {
        Self { inner }
    }

    pub fn get_default_editing_tu_options(&self) -> Result<DxcTranslationUnitFlags, HRESULT> {
        let mut options: DxcTranslationUnitFlags = DxcTranslationUnitFlags::NONE;
        unsafe {
            return_hr!(
                self.inner.get_default_editing_tu_options(&mut options),
                options
            );
        }
    }

    pub fn create_index(&self) -> Result<DxcIndex, HRESULT> {
        let mut index = None;
        unsafe {
            return_hr!(
                self.inner.create_index(&mut index),
                DxcIndex::new(index.unwrap())
            );
        }
    }

    pub fn create_unsaved_file(
        &self,
        file_name: &str,
        contents: &str,
    ) -> Result<DxcUnsavedFile, HRESULT> {
        let c_file_name = CString::new(file_name).expect("Failed to convert `file_name`");
        let c_contents = CString::new(contents).expect("Failed to convert `contents`");

        let mut file = None;
        unsafe {
            return_hr!(
                self.inner.create_unsaved_file(
                    c_file_name.as_ptr(),
                    c_contents.as_ptr(),
                    contents.len() as u32,
                    &mut file
                ),
                DxcUnsavedFile::new(file.unwrap())
            );
        }
    }
}

pub struct DxcIndex {
    inner: ComPtr<dyn IDxcIndex>,
}

impl DxcIndex {
    fn new(inner: ComPtr<dyn IDxcIndex>) -> Self {
        Self { inner }
    }
}

impl DxcIndex {
    pub fn parse_translation_unit(
        &self,
        source_filename: &str,
        args: &[&str],
        unsaved_files: &[&DxcUnsavedFile],
        options: DxcTranslationUnitFlags,
    ) -> Result<DxcTranslationUnit, HRESULT> {
        let c_source_filename =
            CString::new(source_filename).expect("Failed to convert `source_filename`");

        let uf = unsaved_files
            .iter()
            .map(|unsaved_file| unsaved_file.inner.clone())
            .collect::<Vec<_>>();

        unsafe {
            let mut c_args: Vec<CString> = vec![];
            let mut cliargs = vec![];

            for arg in args.iter() {
                let c_arg = CString::new(*arg).expect("Failed to convert `arg`");
                cliargs.push(c_arg.as_ptr() as *const u8);
                c_args.push(c_arg);
            }

            let mut tu = None;
            return_hr!(
                self.inner.parse_translation_unit(
                    c_source_filename.as_ptr() as *const u8,
                    cliargs.as_ptr(),
                    cliargs.len() as i32,
                    uf.as_ptr(),
                    uf.len() as u32,
                    options,
                    &mut tu
                ),
                DxcTranslationUnit::new(tu.unwrap())
            );
        }
    }
}

pub struct DxcUnsavedFile {
    inner: ComPtr<dyn IDxcUnsavedFile>,
}

impl DxcUnsavedFile {
    pub fn get_length(&self) -> Result<u32, HRESULT> {
        let mut length: u32 = 0;
        unsafe {
            return_hr!(self.inner.get_length(&mut length), length);
        }
    }

    fn new(inner: ComPtr<dyn IDxcUnsavedFile>) -> Self {
        DxcUnsavedFile { inner }
    }
}

pub struct DxcTranslationUnit {
    inner: ComPtr<dyn IDxcTranslationUnit>,
}

impl DxcTranslationUnit {
    fn new(inner: ComPtr<dyn IDxcTranslationUnit>) -> Self {
        DxcTranslationUnit { inner }
    }

    pub fn get_file(&self, name: &[u8]) -> Result<DxcFile, HRESULT> {
        let mut file = None;
        unsafe {
            return_hr!(
                self.inner.get_file(name.as_ptr(), &mut file),
                DxcFile::new(file.unwrap())
            );
        }
    }

    pub fn get_cursor(&self) -> Result<DxcCursor, HRESULT> {
        let mut cursor = None;
        unsafe {
            return_hr!(
                self.inner.get_cursor(&mut cursor),
                DxcCursor::new(cursor.unwrap())
            );
        }
    }
}

pub struct DxcCursor {
    inner: ComPtr<dyn IDxcCursor>,
}

impl DxcCursor {
    fn new(inner: ComPtr<dyn IDxcCursor>) -> Self {
        DxcCursor { inner }
    }

    pub fn get_children(&self, skip: u32, max_count: u32) -> Result<Vec<DxcCursor>, HRESULT> {
        let mut result: *mut ComPtr<dyn IDxcCursor> = std::ptr::null_mut();
        let mut result_length: u32 = 0;

        return_hr!(
            unsafe {
                self.inner
                    .get_children(skip, max_count, &mut result_length, &mut result)
            },
            {
                (0..result_length)
                    .map(|i| {
                        let child = unsafe { (*result.offset(i as isize)).clone() };
                        DxcCursor::new(child)
                    })
                    .collect::<Vec<_>>()
            }
        );
    }

    pub fn get_all_children(&self) -> Result<Vec<DxcCursor>, HRESULT> {
        let max_children_per_chunk = 10;
        let mut children = vec![];

        loop {
            let res = self.get_children(children.len() as u32, max_children_per_chunk)?;
            let res_len = res.len();
            children.extend(res);
            if res_len < max_children_per_chunk as usize {
                break Ok(children);
            }
        }
    }

    pub fn get_extent(&self) -> Result<DxcSourceRange, HRESULT> {
        unsafe {
            let mut range = None;
            return_hr!(
                self.inner.get_extent(&mut range),
                DxcSourceRange::new(range.unwrap())
            );
        }
    }

    pub fn get_location(&self) -> Result<DxcSourceLocation, HRESULT> {
        unsafe {
            let mut location = None;
            return_hr!(
                self.inner.get_location(&mut location),
                DxcSourceLocation::new(location.unwrap())
            );
        }
    }

    pub fn get_display_name(&self) -> Result<String, HRESULT> {
        unsafe {
            let mut name: BSTR = std::ptr::null_mut();
            return_hr!(
                self.inner.get_display_name(&mut name),
                crate::utils::from_bstr(name)
            );
        }
    }

    pub fn get_formatted_name(&self, formatting: DxcCursorFormatting) -> Result<String, HRESULT> {
        unsafe {
            let mut name: BSTR = std::ptr::null_mut();
            return_hr!(
                self.inner.get_formatted_name(formatting, &mut name),
                crate::utils::from_bstr(name)
            );
        }
    }

    pub fn get_qualified_name(&self, include_template_args: bool) -> Result<String, HRESULT> {
        unsafe {
            let mut name: BSTR = std::ptr::null_mut();
            return_hr!(
                self.inner
                    .get_qualified_name(include_template_args, &mut name),
                crate::utils::from_bstr(name)
            );
        }
    }

    pub fn get_kind(&self) -> Result<DxcCursorKind, HRESULT> {
        unsafe {
            let mut cursor_kind: DxcCursorKind = DxcCursorKind::UNEXPOSED_DECL;
            return_hr!(self.inner.get_kind(&mut cursor_kind), cursor_kind);
        }
    }

    pub fn get_kind_flags(&self) -> Result<DxcCursorKindFlags, HRESULT> {
        unsafe {
            let mut cursor_kind_flags: DxcCursorKindFlags = DxcCursorKindFlags::NONE;
            return_hr!(
                self.inner.get_kind_flags(&mut cursor_kind_flags),
                cursor_kind_flags
            );
        }
    }

    pub fn get_semantic_parent(&self) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = None;
            return_hr!(
                self.inner.get_semantic_parent(&mut inner),
                DxcCursor::new(inner.unwrap())
            );
        }
    }

    pub fn get_lexical_parent(&self) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = None;
            return_hr!(
                self.inner.get_lexical_parent(&mut inner),
                DxcCursor::new(inner.unwrap())
            );
        }
    }

    pub fn get_cursor_type(&self) -> Result<DxcType, HRESULT> {
        unsafe {
            let mut inner = None;
            return_hr!(
                self.inner.get_cursor_type(&mut inner),
                DxcType::new(inner.unwrap())
            );
        }
    }

    pub fn get_num_arguments(&self) -> Result<i32, HRESULT> {
        unsafe {
            let mut result: i32 = 0;
            return_hr!(self.inner.get_num_arguments(&mut result), result);
        }
    }

    pub fn get_argument_at(&self, index: i32) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = None;
            return_hr!(
                self.inner.get_argument_at(index, &mut inner),
                DxcCursor::new(inner.unwrap())
            );
        }
    }

    pub fn get_referenced_cursor(&self) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = None;
            return_hr!(
                self.inner.get_referenced_cursor(&mut inner),
                DxcCursor::new(inner.unwrap())
            );
        }
    }

    pub fn get_definition_cursor(&self) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = None;
            return_hr!(
                self.inner.get_definition_cursor(&mut inner),
                DxcCursor::new(inner.unwrap())
            );
        }
    }

    pub fn find_references_in_file(
        &self,
        file: &DxcFile,
        skip: u32,
        top: u32,
    ) -> Result<Vec<DxcCursor>, HRESULT> {
        let mut result: *mut ComPtr<dyn IDxcCursor> = std::ptr::null_mut();
        let mut result_length: u32 = 0;

        return_hr!(
            unsafe {
                self.inner.find_references_in_file(
                    file.inner.clone(),
                    skip,
                    top,
                    &mut result_length,
                    &mut result,
                )
            },
            {
                (0..result_length)
                    .map(|i| {
                        let child = unsafe { (*result.offset(i as isize)).clone() };
                        DxcCursor::new(child)
                    })
                    .collect::<Vec<_>>()
            }
        );
    }

    pub fn get_spelling(&self) -> Result<String, HRESULT> {
        unsafe {
            let mut spelling: LPSTR = std::ptr::null_mut();
            return_hr!(
                self.inner.get_spelling(&mut spelling),
                crate::utils::from_lpstr(spelling)
            );
        }
    }

    pub fn is_equal_to(&self, other: &DxcCursor) -> Result<bool, HRESULT> {
        unsafe {
            let mut result: bool = false;
            return_hr!(
                self.inner.is_equal_to(other.inner.clone(), &mut result),
                result
            );
        }
    }

    pub fn is_null(&mut self) -> Result<bool, HRESULT> {
        unsafe {
            let mut result: bool = false;
            return_hr!(IDxcCursor::is_null(&self.inner, &mut result), result);
        }
    }

    pub fn is_definition(&self) -> Result<bool, HRESULT> {
        unsafe {
            let mut result: bool = false;
            return_hr!(self.inner.is_definition(&mut result), result);
        }
    }

    pub fn get_snapped_child(&self, location: &DxcSourceLocation) -> Result<DxcCursor, HRESULT> {
        unsafe {
            let mut inner = None;
            return_hr!(
                self.inner
                    .get_snapped_child(location.inner.clone(), &mut inner),
                DxcCursor::new(inner.unwrap())
            );
        }
    }

    pub fn get_source<'a>(&self, source: &'a str) -> Result<&'a str, HRESULT> {
        let range = self.get_extent()?;

        let DxcSourceOffsets {
            start_offset,
            end_offset,
        } = range.get_offsets()?;

        let source_range = (start_offset as usize)..(end_offset as usize);

        Ok(&source[source_range])
    }
}

pub struct DxcType {
    inner: ComPtr<dyn IDxcType>,
}

impl DxcType {
    fn new(inner: ComPtr<dyn IDxcType>) -> Self {
        DxcType { inner }
    }

    pub fn get_spelling(&self) -> Result<String, HRESULT> {
        unsafe {
            let mut spelling: LPSTR = std::ptr::null_mut();
            return_hr!(
                self.inner.get_spelling(&mut spelling),
                crate::utils::from_lpstr(spelling)
            );
        }
    }
}

pub struct DxcSourceLocation {
    inner: ComPtr<dyn IDxcSourceLocation>,
}

impl std::fmt::Debug for DxcSourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DxcSourceLocation")
            .field("inner", &self.inner.as_raw())
            .finish()
    }
}

impl DxcSourceLocation {
    fn new(inner: ComPtr<dyn IDxcSourceLocation>) -> Self {
        DxcSourceLocation { inner }
    }
}

#[derive(Debug)]
pub struct DxcSourceOffsets {
    pub start_offset: u32,
    pub end_offset: u32,
}

pub struct DxcSourceRange {
    inner: ComPtr<dyn IDxcSourceRange>,
}

impl std::fmt::Debug for DxcSourceRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DxcSourceRange")
            .field("inner", &self.inner.as_raw())
            .finish()
    }
}

impl DxcSourceRange {
    pub fn get_offsets(&self) -> Result<DxcSourceOffsets, HRESULT> {
        unsafe {
            let mut start_offset: u32 = 0;
            let mut end_offset: u32 = 0;
            return_hr!(
                self.inner.get_offsets(&mut start_offset, &mut end_offset),
                DxcSourceOffsets {
                    start_offset,
                    end_offset
                }
            );
        }
    }
}

impl DxcSourceRange {
    fn new(inner: ComPtr<dyn IDxcSourceRange>) -> Self {
        DxcSourceRange { inner }
    }
}

pub struct DxcFile {
    inner: ComPtr<dyn IDxcFile>,
}

impl DxcFile {
    fn new(inner: ComPtr<dyn IDxcFile>) -> Self {
        DxcFile { inner }
    }
}

impl Dxc {
    pub fn create_intellisense(&self) -> Result<DxcIntellisense, HassleError> {
        let mut intellisense = None;
        return_hr_wrapped!(
            self.get_dxc_create_instance()?(
                &CLSID_DxcIntelliSense,
                &IID_IDXC_INTELLI_SENSE,
                &mut intellisense as *mut _ as *mut *mut _,
            ),
            DxcIntellisense::new(intellisense.unwrap())
        );
    }
}
