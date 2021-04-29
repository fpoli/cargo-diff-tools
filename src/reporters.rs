use crate::diagnostics::{Diagnostic, Level};
use clap::arg_enum;

arg_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum OutputKind {
        Json,
        Rendered,
        GitHub,
    }
}

pub fn report_diagnostic(json_line: &str, diagnostic: &Diagnostic, output: OutputKind) {
    match output {
        OutputKind::Json => {
            println!("{}", json_line);
        }
        OutputKind::Rendered => {
            if let Some(ref message) = diagnostic.message {
                println!("{}", message.rendered);
            }
        }
        OutputKind::GitHub => {
            if let Some(ref message) = diagnostic.message {
                if let Some(primary_span) = message.primary_span() {
                    let message_kind = match message.level {
                        Level::Help => "debug",
                        Level::Note => "debug",
                        Level::Warning => "warning",
                        Level::Error => "error",
                    };
                    println!(
                        "::{message_kind} file={name},line={line},col={col}::{message}",
                        message_kind = message_kind,
                        name = primary_span.file_name,
                        line = primary_span.line_start,
                        col = primary_span.column_start,
                        message = escape_github_message(&message.rendered),
                    );
                }
            }
        }
    }
}

fn escape_github_message(message: &str) -> String {
    message
        .replace("%", "%25")
        .replace("\r", "%0D")
        .replace("\n", "%0A")
}
