//! The interactive REPL: reedline with tungsten's prompt, highlighting,
//! completion and history. Behaviour lives in [`crate::repl`]; this file only
//! connects it to a terminal.

use crate::Settings;
use crate::repl::{COMMANDS, Repl, Reply};
use nu_ansi_term::{Color, Style};
use reedline::{
    ColumnarMenu, Completer, Emacs, FileBackedHistory, Highlighter, KeyCode, KeyModifiers,
    MenuBuilder, Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, Reedline,
    ReedlineEvent, ReedlineMenu, Signal, Span, StyledText, Suggestion, default_emacs_keybindings,
};
use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use tungsten_core::{Class, Scope};

/// Lines of history kept.
const HISTORY: usize = 10_000;

/// `$XDG_DATA_HOME/tungsten/history`, else `~/.local/share/tungsten/history`.
pub fn history_path() -> Option<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(data.join("tungsten").join("history"))
}

/// What the highlighter and completer know of the session.
type Shared = Arc<Mutex<Scope>>;

fn scope(s: &Shared) -> Scope {
    s.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// The colours of concept §8, from the filament palette.
fn style(c: Class) -> Style {
    match c {
        Class::Number => Color::Fixed(136).normal(),
        Class::Unit => Color::Fixed(30).normal(),
        Class::Entity | Class::Property => Color::Fixed(64).normal(),
        Class::Variable => Style::new().bold(),
        Class::Function | Class::Keyword => Style::new(),
        Class::Unknown => Style::new().underline(),
    }
}

struct Highlight {
    scope: Shared,
    color: bool,
}

impl Highlighter for Highlight {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut out = StyledText::new();
        if !self.color || line.starts_with(':') {
            out.push((Style::new(), line.to_string()));
            return out;
        }
        let mut at = 0;
        for (span, class) in tungsten_core::classify(line, &scope(&self.scope)) {
            if span.start < at || span.end > line.len() {
                continue;
            }
            if span.start > at {
                out.push((Style::new(), line[at..span.start].to_string()));
            }
            out.push((style(class), line[span.clone()].to_string()));
            at = span.end;
        }
        if at < line.len() {
            out.push((Style::new(), line[at..].to_string()));
        }
        out
    }
}

struct Complete {
    scope: Shared,
}

impl Completer for Complete {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let suggestion = |value: &str, note: &str, span: Span| Suggestion {
            value: value.to_string(),
            description: Some(note.to_string()),
            span,
            append_whitespace: true,
            ..Suggestion::default()
        };
        let pos = pos.min(line.len());
        if line.starts_with(':') {
            let typed = &line[..pos];
            let found: Vec<Suggestion> = COMMANDS
                .iter()
                .filter_map(|(c, what)| {
                    let name = c.split_whitespace().next()?;
                    name.starts_with(typed)
                        .then(|| suggestion(name, what, Span::new(0, pos)))
                })
                .collect();
            return found;
        }
        let c = tungsten_core::complete(line, pos, &scope(&self.scope));
        let span = Span::new(c.span.start, c.span.end);
        c.candidates
            .iter()
            .map(|k| suggestion(&k.text, &k.note, span))
            .collect()
    }
}

struct WPrompt {
    color: bool,
    /// `W›`, or `W>` for `--plain`.
    fancy: bool,
}

impl Prompt for WPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed("W")
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed(if self.fancy { "› " } else { "> " })
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("… ")
    }

    fn render_prompt_history_search_indicator(&self, h: PromptHistorySearch) -> Cow<'_, str> {
        let failed = match h.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "no match ",
        };
        Cow::Owned(format!("({failed}search: {}) ", h.term))
    }

    fn get_prompt_color(&self) -> reedline::Color {
        if self.color {
            reedline::Color::AnsiValue(136)
        } else {
            reedline::Color::Reset
        }
    }

    fn get_indicator_color(&self) -> reedline::Color {
        if self.color {
            reedline::Color::AnsiValue(244)
        } else {
            reedline::Color::Reset
        }
    }
}

/// Runs the REPL until Ctrl-D or `:quit`.
pub fn run(settings: Settings, width: impl Fn() -> usize) -> std::io::Result<()> {
    let mut repl = Repl::new(settings);
    let shared: Shared = Arc::new(Mutex::new(repl.scope()));

    let mut keys = default_emacs_keybindings();
    keys.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion".into()),
            ReedlineEvent::MenuNext,
        ]),
    );
    let menu = ColumnarMenu::default().with_name("completion");
    let mut editor = Reedline::create()
        .with_highlighter(Box::new(Highlight {
            scope: shared.clone(),
            color: settings.color,
        }))
        .with_completer(Box::new(Complete {
            scope: shared.clone(),
        }))
        .with_menu(ReedlineMenu::EngineCompleter(Box::new(menu)))
        .with_edit_mode(Box::new(Emacs::new(keys)))
        .with_ansi_colors(settings.color);
    // No history is better than no REPL: a read-only home still works.
    if let Some(path) = history_path()
        && path
            .parent()
            .is_some_and(|d| std::fs::create_dir_all(d).is_ok())
        && let Ok(h) = FileBackedHistory::with_file(HISTORY, path)
    {
        editor = editor.with_history(Box::new(h));
    }

    let prompt = WPrompt {
        color: settings.color,
        fancy: settings.fancy,
    };
    let dot = if settings.fancy { "·" } else { "-" };
    let mut out = std::io::stdout();
    let mut greeting = String::from("  ");
    tungsten_render::FILAMENT.paint(
        tungsten_render::Style::Dim,
        &format!(
            "tungsten {}  {dot}  :help for commands  {dot}  Ctrl-D to leave",
            env!("CARGO_PKG_VERSION")
        ),
        settings.color,
        &mut greeting,
    );
    writeln!(out, "{greeting}\n")?;

    loop {
        match editor.read_line(&prompt)? {
            Signal::Success(line) => {
                repl.set_width(width());
                let reply = repl.line(&line);
                // Keep what was typed even if the session is killed later.
                let _ = editor.sync_history();
                match reply {
                    Reply::Show(text) => {
                        out.write_all(text.as_bytes())?;
                        out.flush()?;
                    }
                    Reply::Quit => break,
                }
                *shared.lock().unwrap_or_else(PoisonError::into_inner) = repl.scope();
            }
            // reedline has already cleared the line.
            Signal::CtrlC => {}
            Signal::CtrlD => break,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn painted(line: &str, scope: Scope) -> Vec<(Style, String)> {
        let h = Highlight {
            scope: Arc::new(Mutex::new(scope)),
            color: true,
        };
        h.highlight(line, line.len()).buffer
    }

    #[test]
    fn paints_what_the_resolver_reads() {
        let line = "rent * 12 month in earth's mass";
        let scope = Scope {
            vars: ["rent".to_string()].into(),
            funcs: Default::default(),
        };
        let parts = painted(line, scope.clone());
        // Nothing is lost or added.
        assert_eq!(
            parts.iter().map(|(_, t)| t.as_str()).collect::<String>(),
            line
        );
        // Every classified span is painted in its class's style.
        for (span, class) in tungsten_core::classify(line, &scope) {
            assert!(
                parts.contains(&(style(class), line[span.clone()].to_string())),
                "{:?} not painted as {class:?}",
                &line[span]
            );
        }
        let style_of = |w: &str| parts.iter().find(|(_, t)| t == w).map(|(s, _)| *s);
        assert_eq!(style_of("rent"), Some(style(Class::Variable)));
        assert_eq!(style_of("12"), Some(style(Class::Number)));
        assert_eq!(style_of("month"), Some(style(Class::Unit)));
        assert_eq!(style_of("earth"), Some(style(Class::Entity)));
    }

    #[test]
    fn unknown_words_are_underlined() {
        let parts = painted("5 blorps", Scope::default());
        assert!(parts.contains(&(Style::new().underline(), "blorps".into())));
    }

    #[test]
    fn plain_and_commands_are_not_painted() {
        let h = Highlight {
            scope: Arc::default(),
            color: false,
        };
        assert_eq!(
            h.highlight("5 km", 4).buffer,
            [(Style::new(), "5 km".into())]
        );
        assert_eq!(
            painted(":vars", Scope::default()),
            [(Style::new(), ":vars".into())]
        );
    }

    #[test]
    fn completes_commands() {
        let mut c = Complete {
            scope: Arc::default(),
        };
        let found: Vec<String> = c.complete(":p", 2).into_iter().map(|s| s.value).collect();
        assert_eq!(found, [":pods"]);
        let found: Vec<String> = c
            .complete("earth's ra", 10)
            .into_iter()
            .map(|s| s.value)
            .collect();
        assert_eq!(found, ["radius"]);
    }
}
