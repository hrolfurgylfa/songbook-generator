use std::{ffi::OsStr, fmt, iter::repeat, os::windows::prelude::OsStrExt};

use genpdf::fonts::{FontData, FontFamily};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::BOOL,
        Globalization::GetUserDefaultLocaleName,
        Graphics::DirectWrite::{
            DWriteCreateFactory, IDWriteFactory, IDWriteFont, IDWriteFontCollection,
            IDWriteLocalizedStrings, DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_FACE_TYPE,
            DWRITE_FONT_FILE_TYPE, DWRITE_FONT_FILE_TYPE_TRUETYPE, DWRITE_FONT_STRETCH_NORMAL,
            DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD,
            DWRITE_FONT_WEIGHT_NORMAL,
        },
        System::SystemServices::LOCALE_NAME_MAX_LENGTH,
    },
};

fn to_pcwstr(s: &str) -> PCWSTR {
    PCWSTR::from_raw(
        OsStr::new(s)
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect::<Vec<_>>()
            .as_ptr(),
    )
}

#[derive(Clone, PartialEq, Eq)]
pub struct FontError {
    msg: String,
    win_err: Option<windows::core::Error>,
}

impl FontError {
    pub fn new_msg_only(msg: impl Into<String>) -> FontError {
        FontError {
            msg: msg.into(),
            win_err: None,
        }
    }
    pub fn new(msg: impl Into<String>, win_err: windows::core::Error) -> FontError {
        FontError {
            msg: msg.into(),
            win_err: Some(win_err),
        }
    }
}

impl fmt::Debug for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Error");
        debug
            .field("message", &self.msg)
            .field("windows error", &self.win_err)
            .finish()
    }
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(win_err) = &self.win_err {
            write!(f, "{} ({})", self.msg, win_err)
        } else {
            write!(f, "{}", self.msg)
        }
    }
}

impl From<windows::core::Error> for FontError {
    fn from(err: windows::core::Error) -> Self {
        FontError {
            msg: "".to_owned(),
            win_err: Some(err),
        }
    }
}

fn get_direct_write_factory() -> Result<IDWriteFactory, FontError> {
    unsafe {
        DWriteCreateFactory::<IDWriteFactory>(DWRITE_FACTORY_TYPE_SHARED)
            .map_err(|e| FontError::new("Failed to instantiate DWrite factory", e))
    }
}

fn get_system_font_collection(check_for_updates: bool) -> Result<IDWriteFontCollection, FontError> {
    let mut maybe_font_collection = None;
    unsafe {
        let factory = get_direct_write_factory()?;
        factory
            .GetSystemFontCollection(&mut maybe_font_collection, check_for_updates)
            .map_err(|e| FontError::new("Failed to get system font collection", e))?;
    }
    Ok(maybe_font_collection.unwrap())
}

fn localize_string(strings: IDWriteLocalizedStrings) -> Result<String, FontError> {
    // Get the region correct name
    let mut index = 0;
    let mut exists: BOOL = false.into();
    unsafe {
        // Get the OS default locale
        let mut default_locale_name = repeat(0)
            .take(LOCALE_NAME_MAX_LENGTH as usize + 1)
            .collect::<Vec<u16>>();
        let success = GetUserDefaultLocaleName(&mut default_locale_name);
        if success != 0 {
            strings.FindLocaleName(
                PCWSTR::from_raw(default_locale_name.as_ptr()),
                &mut index,
                &mut exists,
            )?;
        }

        // Get the english locale
        if exists.0 == 0 {
            strings.FindLocaleName(to_pcwstr("en_us"), &mut index, &mut exists)?;
        }
    }

    // Get the first one if no locales were found
    if exists.0 == 0 {
        index = 0;
    }

    // Move string of index into return value
    let length = unsafe { strings.GetStringLength(index) }
        .map_err(|e| FontError::new("Failed to get string length", e))?;
    let mut family_name_buffer = repeat(0 as u16)
        .take((length + 1) as usize)
        .collect::<Vec<_>>();
    unsafe {
        strings
            .GetString(index, &mut family_name_buffer)
            .map_err(|e| FontError::new("Failed to get string", e))?;
    }

    Ok(String::from_utf16_lossy(&family_name_buffer))
}

pub fn get_fonts() -> Result<Vec<String>, FontError> {
    let mut family_names = vec![];
    let font_collection = get_system_font_collection(false)?;
    unsafe {
        for i in 0..font_collection.GetFontFamilyCount() {
            // Get the font family and names
            let family = font_collection
                .GetFontFamily(i)
                .map_err(|e| FontError::new("Failed to get font family", e))?;
            let names = family
                .GetFamilyNames()
                .map_err(|e| FontError::new("Failed to get font family names", e))?;

            let family_name = localize_string(names)?;
            family_names.push(family_name);
        }
    }

    family_names.sort();
    Ok(family_names)
}

fn get_font_data(font: IDWriteFont) -> Result<Vec<u8>, FontError> {
    unsafe {
        let font_face = font.CreateFontFace()?;
        let mut num_files = 0;
        font_face.GetFiles(&mut num_files, None)?;

        // let factory = get_direct_write_factory()?;
        let mut maybe_file = None; // Some(factory.CreateFontFileReference(to_pcwstr(""), None)?);
        font_face.GetFiles(&mut num_files, Some(&mut maybe_file))?;
        let file = maybe_file.unwrap();

        let mut is_supported = false.into();
        let mut file_type = DWRITE_FONT_FILE_TYPE::default();
        let mut face_type = DWRITE_FONT_FACE_TYPE::default();
        let mut num_faces = 0;
        file.Analyze(
            &mut is_supported,
            &mut file_type,
            Some(&mut face_type),
            &mut num_faces,
        )?;
        if file_type != DWRITE_FONT_FILE_TYPE_TRUETYPE {
            return Err(FontError::new_msg_only(format!(
                "Failed to use non-TrueType font, it is type {:?}. Please select another font.",
                file_type
            )));
        }

        let mut reference_key: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut reference_key_size = 0;
        file.GetReferenceKey(&mut reference_key, &mut reference_key_size)?;
        let loader = file.GetLoader()?;
        let file_stream = loader.CreateStreamFromKey(reference_key, reference_key_size)?;
        let file_size = file_stream.GetFileSize()?;
        let mut file_buffer: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut fragment_context: *mut std::ffi::c_void = std::ptr::null_mut();
        file_stream.ReadFileFragment(&mut file_buffer, 0, file_size, &mut fragment_context)?;
        let file_buffer_slice =
            std::slice::from_raw_parts(file_buffer as *mut u8, file_size as usize);
        let file_buffer_clone = file_buffer_slice.to_owned();
        drop(file_buffer_slice); // We can't use the file buffer after releasing the file fragment
        file_stream.ReleaseFileFragment(fragment_context);
        Ok(file_buffer_clone)
    }
}

pub fn get_font(name: &str) -> Result<FontFamily<FontData>, FontError> {
    let font_collection = get_system_font_collection(false)?;
    unsafe {
        let mut index = 0;
        let mut exists = false.into();
        font_collection.FindFamilyName(to_pcwstr(name), &mut index, &mut exists)?;
        if exists.0 == 0 {
            return Err(FontError::new_msg_only(format!(
                "Failed to find font family with name {}",
                name
            )));
        }
        let family = font_collection.GetFontFamily(index)?;
        let mut fonts = [
            (DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_STYLE_ITALIC), // Bold-Italic
            (DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_STYLE_ITALIC), // Italic
            (DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_STYLE_NORMAL), // Bold
            (DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_STYLE_NORMAL), // Regular
        ]
        .into_iter()
        .map(|(weight, style)| {
            let font = family.GetFirstMatchingFont(weight, DWRITE_FONT_STRETCH_NORMAL, style)?;
            let font_data = get_font_data(font)?;
            FontData::new(font_data, None).map_err(|e| {
                FontError::new_msg_only(format!("Failed to read font data with rusttype: {}", e))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

        Ok(FontFamily {
            regular: fonts.remove(3),
            bold: fonts.remove(2),
            italic: fonts.remove(1),
            bold_italic: fonts.remove(0),
        })
    }
}
