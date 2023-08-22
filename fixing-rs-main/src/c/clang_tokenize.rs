use clang_sys::{
    clang_createIndex, clang_createTranslationUnitFromSourceFile, clang_getCString,
    clang_getCursorExtent, clang_getTokenKind, clang_getTokenSpelling,
    clang_getTranslationUnitCursor, CXErrorCode, CXError_Failure, CXToken, CXTokenKind,
    CXUnsavedFile,
};
use std::{
    error::Error,
    ffi::{CStr, CString},
    fmt::Display,
    os::raw::c_char,
};

#[derive(Debug)]
pub enum ClangTokenizeError {
    Utf8Error(std::str::Utf8Error),
    ClangError(CXErrorCode),
}

impl Display for ClangTokenizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ClangTokenizeError {}

pub fn clang_tokenize_file(path: &str) -> Result<Vec<(CXTokenKind, String)>, ClangTokenizeError> {
    let path = path.as_bytes();
    let mut file_name = Vec::new();
    file_name.extend(path);
    let file_name = CString::new(file_name).unwrap();

    let mut err = None;
    let mut result = Vec::new();

    unsafe {
        let index = clang_createIndex(0, 1);
        let tu = clang_createTranslationUnitFromSourceFile(
            index,
            file_name.as_ptr(),
            0,
            0 as *const *const c_char,
            0,
            0 as *mut CXUnsavedFile,
        );
        if tu as usize == 0 {
            return Err(ClangTokenizeError::ClangError(CXError_Failure));
        }
        let cursor = clang_getTranslationUnitCursor(tu);
        let range = clang_getCursorExtent(cursor);
        let mut tokens: *mut CXToken = 0 as *mut CXToken;
        let mut n_tokens: u32 = 0;
        clang_sys::clang_tokenize(tu, range, &mut tokens, &mut n_tokens);

        for i in 0..n_tokens {
            let token = *tokens.offset(i as isize);
            let token_kind = clang_getTokenKind(token);
            let spelling = clang_getTokenSpelling(tu, token);
            let spelling = clang_getCString(spelling);
            let spelling = CStr::from_ptr(spelling).to_str();
            let spelling = match spelling {
                Ok(s) => s,
                Err(e) => {
                    err = Some(ClangTokenizeError::Utf8Error(e));
                    break;
                }
            };
            result.push((token_kind, spelling.to_string()));
        }

        clang_sys::clang_disposeTokens(tu, tokens, n_tokens);
        clang_sys::clang_disposeIndex(index);
    }

    if let Some(e) = err {
        return Err(e);
    }

    Ok(result)
}

#[cfg(test)]
#[test]
fn test_lexer() {
    let tokens =
        clang_tokenize_file("/home/zwj/research-compiling-error-fixing/fixing-rs/src/c/c-test.c")
            .unwrap();
    for token in tokens {
        println!("{:?}", token);
    }
}
