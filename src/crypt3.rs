use std::ffi::{CStr, CString};

use anyhow::*;

pub fn crypt3_sha256_escaped(password: String, salt: String) -> Result<String> {
    let sha256_setting = "$5$";
    let settings = format!("{sha256_setting}{salt}");

    let csetting = CString::new(settings).context("invalid password")?;
    let cpassword = CString::new(password).context("invalid salt")?;

    let mut output = vec![0_i8; 256];

    let ret_cstr = unsafe {
        let _ret = crypt3_sys::crypt_r(cpassword.as_ptr(), csetting.as_ptr(), output.as_mut_ptr());
        CStr::from_ptr(output.as_ptr())
    };

    let ret_str = ret_cstr.to_str().context("can not parse hashed password")?.to_string();
    let escaped = ret_str.replace('$', "\\$");
    Ok(escaped)
}
