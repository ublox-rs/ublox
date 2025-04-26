#[derive(Clone, Copy)]
pub struct DebugContext {
    enabled: bool,
}

impl DebugContext {
    pub fn from_attrs(attrs: &[syn::Attribute]) -> Self {
        let enabled = attrs.iter().any(|attr| attr.path.is_ident("debug_this"));
        DebugContext { enabled }
    }

    pub fn print_at(&self, file: &str, line: u32, args: std::fmt::Arguments) {
        if self.enabled {
            println!("[{}:{}] {}", file, line, args);
        }
    }
}
