#![allow(clippy::all)]
#![allow(unused_variables)]
#![allow(dead_code)]

#[derive(Clone, Copy)]
pub struct DebugContext {
    enabled: bool,
}

impl DebugContext {
    pub fn from_attrs(attrs: &[syn::Attribute]) -> Self {
        let enabled = attrs.iter().any(|attr| attr.path.is_ident("debug_this"));
        DebugContext { enabled }
    }

    /// Print with file and line information
    ///
    /// Usage:
    /// ```ignore
    /// dbg_ctx.print_at(file!(), line!(), format_args!("foo type: {ty:?}"));
    /// ```

    pub fn print_at(&self, file: &str, line: u32, args: std::fmt::Arguments) {
        #[cfg(test)]
        if self.enabled {
            println!("[{}:{}] {}", file, line, args);
        }
    }

    /// Prints as is
    pub fn print(&self, msg: impl std::fmt::Display) {
        #[cfg(test)]
        if self.enabled {
            println!("{msg}");
        }
    }

    /// Prints formatted code or an error if the code couldn't be formatted
    pub fn print_code(&self, code: impl std::fmt::Display) {
        #[cfg(test)]
        if self.enabled {
            match format_rust_code(&code) {
                Ok(formatted_code) => print_highlighted(&formatted_code),
                Err(e) => println!("Failed to format code '{code}': {e:?}"),
            }
        }
    }
}

#[cfg(test)]
mod debug_only {
    use std::{
        io::Write as _,
        process::{Command, Stdio},
    };
    use syntect::{
        easy::HighlightLines,
        highlighting::{Style, ThemeSet},
        parsing::SyntaxSet,
        util::{as_24_bit_terminal_escaped, LinesWithEndings},
    };

    // Spawns rustfmt, runs code through it and returns the formatted code
    fn format_rust_code(
        code: impl std::fmt::Display,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut child = Command::new("rustfmt")
            .arg("--emit")
            .arg("stdout")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        {
            let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
            stdin.write_all(code.to_string().as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(format!(
                "rustfmt failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into())
        }
    }

    fn print_highlighted(code: &str) {
        use syntect::{
            easy::HighlightLines,
            highlighting::{Style, ThemeSet},
            parsing::SyntaxSet,
            util::{as_24_bit_terminal_escaped, LinesWithEndings},
        };
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let syntax = ps.find_syntax_by_extension("rs").unwrap();
        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

        for line in LinesWithEndings::from(code) {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            print!("{}", escaped);
        }
        // Clear the terminal formatting
        print!("\x1b[0m");
    }
}
