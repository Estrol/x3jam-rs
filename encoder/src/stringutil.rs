use std::ffi::{CStr, CString};

use encoding_rs::{self, EUC_KR, GBK, WINDOWS_1252};

pub fn try_string(cstring: &CStr) -> Result<String, std::str::Utf8Error> {
    let bytes = cstring.to_bytes();

    match std::str::from_utf8(bytes) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => {
            for enc in [EUC_KR, GBK, WINDOWS_1252] {
                let (cow, _, had_errors) = enc.decode(bytes);
                if !had_errors {
                    return Ok(cow.to_string());
                }
            }

            // If all decoding attempts fail, return the original UTF-8 error
            Err(e)
        }
    }
}

#[derive(Debug, Clone)]
pub struct CStrEx {
    pub string: Vec<u8>,
    pub encoding: &'static str,
}

impl CStrEx {
    pub const fn empty() -> Self {
        CStrEx {
            string: Vec::new(),
            encoding: "UTF-8",
        }
    }

    pub fn from_string(s: &str) -> Self {
        CStrEx {
            string: s.as_bytes().to_vec(),
            encoding: "UTF-8",
        }
    }

    pub fn from_cstr(cstring: &CStr) -> Self {
        let bytes = cstring.to_bytes();

        // This bypassed the UTF-8 check and returns the original bytes if they are valid UTF-8.
        if std::str::from_utf8(bytes).is_ok() {
            return CStrEx {
                string: bytes.to_vec(),
                encoding: "UTF-8",
            };
        }

        for enc in [EUC_KR, GBK, WINDOWS_1252] {
            let (cow, _, had_errors) = enc.decode(bytes);
            if !had_errors {
                return CStrEx {
                    string: cow.into_owned().into_bytes(),
                    encoding: enc.name(),
                };
            }
        }

        // Well we return as-is, since we don't know what encoding it is, and we don't want to lose data.
        CStrEx {
            string: bytes.to_vec(),
            encoding: "UTF-8",
        }
    }

    pub fn to_string(&self) -> String {
        unsafe {
            // We already know this is valid UTF-8, so we can use from_utf8_unchecked.
            std::str::from_utf8_unchecked(&self.string).to_string()
        }
    }

    pub fn to_cstring(&self) -> Result<CString, std::ffi::NulError> {
        CString::new(self.string.clone())
    }

    pub fn len(&self) -> usize {
        self.string.len()
    }

    pub fn contains(&self, other: &CStrEx) -> bool {
        self.string.windows(other.len()).any(|window| window == other.string.as_slice())
    }
}