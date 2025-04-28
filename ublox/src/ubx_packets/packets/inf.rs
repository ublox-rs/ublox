pub(crate) fn convert_to_str(bytes: &[u8]) -> Option<&str> {
    core::str::from_utf8(bytes).ok()
}

pub(crate) fn is_valid(_bytes: &[u8]) -> bool {
    // Validity is checked in convert_to_str
    true
}
