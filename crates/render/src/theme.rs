//! The filament-glow theme (docs/concept.md §1). The only place in the
//! workspace that writes ANSI escape codes.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Plain,
    /// Pod glyph and title.
    Title,
    ErrorTitle,
    /// Result numbers.
    Value,
    Unit,
    /// Interpretation, hints, footer.
    Dim,
    Error,
}

/// ANSI SGR parameters per style. Colours are from the 256-colour palette and
/// were chosen to keep at least 3:1 contrast on both black and white
/// backgrounds (see docs/screenshots/).
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub title: &'static str,
    pub error_title: &'static str,
    pub value: &'static str,
    pub unit: &'static str,
    pub dim: &'static str,
    pub error: &'static str,
}

pub const FILAMENT: Theme = Theme {
    // amber #af8700 (3.2:1 on white, 6.3:1 on black; #d78700 fell to 2.9 on white)
    title: "1;38;5;136",
    error_title: "1;38;5;167",
    // default foreground, bold: bright on dark, dark on light
    value: "1",
    // teal #008787
    unit: "38;5;30",
    // grey #808080
    dim: "38;5;244",
    // muted red #d75f5f
    error: "38;5;167",
};

impl Theme {
    fn code(&self, s: Style) -> Option<&'static str> {
        match s {
            Style::Plain => None,
            Style::Title => Some(self.title),
            Style::ErrorTitle => Some(self.error_title),
            Style::Value => Some(self.value),
            Style::Unit => Some(self.unit),
            Style::Dim => Some(self.dim),
            Style::Error => Some(self.error),
        }
    }

    pub fn paint(&self, s: Style, text: &str, color: bool, out: &mut String) {
        match self.code(s).filter(|_| color && !text.is_empty()) {
            Some(code) => {
                out.push_str("\x1b[");
                out.push_str(code);
                out.push('m');
                out.push_str(text);
                out.push_str("\x1b[0m");
            }
            None => out.push_str(text),
        }
    }
}
