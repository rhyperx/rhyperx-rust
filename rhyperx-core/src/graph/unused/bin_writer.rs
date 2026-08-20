use std::fs::write;

pub fn save_vec(path: &str, data: &[(u32, u32)]) -> std::io::Result<()> {
    let mut bytes = Vec::with_capacity(4 + data.len() * 8);

    // length (u32 = 4 bytes)
    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());

    for &(a, b) in data {
        bytes.extend_from_slice(&a.to_le_bytes());
        bytes.extend_from_slice(&b.to_le_bytes());
    }

    write(path, bytes)
}

pub fn load_vec(path: &str) -> std::io::Result<Vec<(u32, u32)>> {
    let bytes = std::fs::read(path)?;

    let len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    let data = &bytes[4..];

    let u32_slice: &[u32] =
        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u32, data.len() / 4) };

    let mut result = Vec::with_capacity(len);

    let ptr = u32_slice.as_ptr();

    unsafe {
        std::ptr::copy_nonoverlapping(
            ptr as *const (u32, u32),
            result.as_mut_ptr() as *mut (u32, u32),
            len,
        );
        result.set_len(len);
    }

    Ok(result)
}
