use serde::Deserialize;
use std::{fs, path::Path};

pub const SECRET_STORAGE_DPAPI: &str = "dpapi";
pub const SECRET_STORAGE_PLAIN: &str = "plain";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenHelperState {
    token_mode: String,
    #[serde(default)]
    key_path: Option<String>,
    #[serde(default)]
    secret_storage: Option<String>,
}

pub fn protect_gateway_secret(secret: &str) -> Result<(Vec<u8>, &'static str), String> {
    if secret.trim().is_empty() {
        return Err("Gateway key is empty".to_string());
    }
    protect_secret(secret)
}

pub fn gateway_bearer_from_config(path: &Path) -> Result<Option<String>, String> {
    let data =
        fs::read_to_string(path).map_err(|error| format!("Read Codex Assistant state: {error}"))?;
    let state: TokenHelperState = serde_json::from_str(&data)
        .map_err(|error| format!("Parse Codex Assistant state: {error}"))?;
    gateway_bearer_from_fields(
        &state.token_mode,
        state.key_path.as_deref(),
        state.secret_storage.as_deref(),
    )
}

pub fn gateway_bearer_from_fields(
    token_mode: &str,
    key_path: Option<&str>,
    secret_storage: Option<&str>,
) -> Result<Option<String>, String> {
    match token_mode {
        "none" => Ok(None),
        "static" => {
            let secret = read_secret(key_path, secret_storage)?;
            Ok(Some(secret))
        }
        other => Err(format!("Unsupported token mode: {other}")),
    }
}

fn read_secret(key_path: Option<&str>, secret_storage: Option<&str>) -> Result<String, String> {
    let key_path =
        key_path.ok_or_else(|| "Missing keyPath in Codex Assistant state".to_string())?;
    let data = fs::read(key_path).map_err(|error| format!("Read gateway secret: {error}"))?;
    let secret = match secret_storage.unwrap_or(SECRET_STORAGE_PLAIN) {
        SECRET_STORAGE_DPAPI => unprotect_secret(&data)?,
        SECRET_STORAGE_PLAIN | "" => {
            String::from_utf8(data).map_err(|error| format!("Decode gateway secret: {error}"))?
        }
        other => return Err(format!("Unsupported secret storage: {other}")),
    };
    let trimmed = secret.trim().to_string();
    if trimmed.is_empty() {
        return Err("Gateway secret is empty".to_string());
    }
    Ok(trimmed)
}

#[cfg(windows)]
fn protect_secret(secret: &str) -> Result<(Vec<u8>, &'static str), String> {
    windows_dpapi::protect(secret.as_bytes()).map(|data| (data, SECRET_STORAGE_DPAPI))
}

#[cfg(not(windows))]
fn protect_secret(secret: &str) -> Result<(Vec<u8>, &'static str), String> {
    Ok((secret.as_bytes().to_vec(), SECRET_STORAGE_PLAIN))
}

#[cfg(windows)]
fn unprotect_secret(data: &[u8]) -> Result<String, String> {
    let plaintext = windows_dpapi::unprotect(data)?;
    String::from_utf8(plaintext).map_err(|error| format!("Decode DPAPI secret: {error}"))
}

#[cfg(not(windows))]
fn unprotect_secret(data: &[u8]) -> Result<String, String> {
    String::from_utf8(data.to_vec()).map_err(|error| format!("Decode gateway secret: {error}"))
}

#[cfg(windows)]
mod windows_dpapi {
    use std::{ffi::c_void, io, ptr, slice};

    const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x1;

    #[repr(C)]
    struct DataBlob {
        cb_data: u32,
        pb_data: *mut u8,
    }

    #[link(name = "Crypt32")]
    unsafe extern "system" {
        fn CryptProtectData(
            data_in: *mut DataBlob,
            data_descr: *const u16,
            optional_entropy: *mut DataBlob,
            reserved: *mut c_void,
            prompt_struct: *mut c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;

        fn CryptUnprotectData(
            data_in: *mut DataBlob,
            data_descr: *mut *mut u16,
            optional_entropy: *mut DataBlob,
            reserved: *mut c_void,
            prompt_struct: *mut c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;
    }

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn LocalFree(mem: *mut c_void) -> *mut c_void;
    }

    pub fn protect(data: &[u8]) -> Result<Vec<u8>, String> {
        let mut input = DataBlob {
            cb_data: data
                .len()
                .try_into()
                .map_err(|_| "Secret is too large for DPAPI".to_string())?,
            pb_data: data.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            cb_data: 0,
            pb_data: ptr::null_mut(),
        };
        let ok = unsafe {
            CryptProtectData(
                &mut input,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        take_output_blob(ok, output, "CryptProtectData")
    }

    pub fn unprotect(data: &[u8]) -> Result<Vec<u8>, String> {
        let mut input = DataBlob {
            cb_data: data
                .len()
                .try_into()
                .map_err(|_| "Secret is too large for DPAPI".to_string())?,
            pb_data: data.as_ptr() as *mut u8,
        };
        let mut output = DataBlob {
            cb_data: 0,
            pb_data: ptr::null_mut(),
        };
        let ok = unsafe {
            CryptUnprotectData(
                &mut input,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        take_output_blob(ok, output, "CryptUnprotectData")
    }

    fn take_output_blob(ok: i32, output: DataBlob, operation: &str) -> Result<Vec<u8>, String> {
        if ok == 0 {
            return Err(format!(
                "{operation} failed: {}",
                io::Error::last_os_error()
            ));
        }
        if output.pb_data.is_null() {
            return Err(format!("{operation} returned empty data"));
        }
        let data =
            unsafe { slice::from_raw_parts(output.pb_data, output.cb_data as usize).to_vec() };
        unsafe {
            let _ = LocalFree(output.pb_data as *mut c_void);
        }
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_gateway_secret_round_trips() {
        let (protected, storage) = protect_gateway_secret("local-test-key").expect("protect");
        let secret = match storage {
            SECRET_STORAGE_DPAPI => unprotect_secret(&protected).expect("unprotect"),
            SECRET_STORAGE_PLAIN => String::from_utf8(protected).expect("plain"),
            other => panic!("unexpected storage {other}"),
        };
        assert_eq!(secret, "local-test-key");
    }
}
