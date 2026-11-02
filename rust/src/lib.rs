//! zeolite-parser: scaffold for the Rust side of the module pipeline.
//! Today it only answers the question the JS side asks before parsing:
//! does this source mention `location` at all? The real rewriter comes next.

#[no_mangle]
pub extern "C" fn needs_location_rewrite(ptr: *const u8, len: usize) -> u32 {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let src = std::str::from_utf8(bytes).unwrap_or("");
    if src.contains("location") { 1 } else { 0 }
}
