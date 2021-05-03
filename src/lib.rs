use crate::diagnostics::{Diagnostic, Level};
use crate::diff::{parse_diff, FileChanges};
use crate::intervals::intersect_intervals;
use crate::reporters::{report_diagnostic, OutputKind};
use anyhow::{bail, Context, Result};
use clap::{crate_authors, crate_description, crate_version, value_t, App, AppSettings, Arg};
use std::process::{Command, Stdio};
use std::{
    env,
    io::{self, BufRead, BufReader, Write},
};

mod diagnostics;
mod diff;
mod intervals;
mod reporters;

pub fn build_app(binary_name: &str, subcommand: Option<(&str, &[&str])>) -> Result<()> {
    // Rip off the arguments to be passed to the subcommand
    let app_args: Vec<String>;
    let subcommand_extra_args: Vec<String>;
    if subcommand.is_none() {
        app_args = env::args().collect();
        subcommand_extra_args = vec![];
    } else {
        let args: Vec<_> = env::args().collect();
        if let Some(split_pos) = args.iter().position(|v| v == "--") {
            app_args = args[0..split_pos].into();
            if split_pos + 1 == args.len() {
                subcommand_extra_args = vec![];
            } else {
                subcommand_extra_args = args[(split_pos + 1)..].into();
            }
        } else {
            app_args = args;
            subcommand_extra_args = vec![];
        }
    }

    // Parse the argument of this binary
    let matches = App::new(binary_name)
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::AllowLeadingHyphen)
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FORMAT")
                .help("Format of the output")
                .possible_values(&OutputKind::variants())
                .case_insensitive(true),
        )
        .arg(
            Arg::with_name("args")
                .value_name("FORMAT")
                .help("Additional arguments to pass to `git diff`")
                .multiple(true),
        )
        .get_matches_from(&app_args);

    // Read `git diff` arguments
    let git_diff_args = matches.values_of("args").unwrap_or_default();

    // Obtain diff
    let output = Command::new("git")
        .arg("diff")
        .arg("--unified=0")
        .args(git_diff_args)
        .output()
        .with_context(|| "Failed to start `git diff`")?;
    if !output.stderr.is_empty() {
        io::stderr()
            .write_all(&output.stderr)
            .with_context(|| "Failed to report the stderr of `git diff`")?;
    }
    if !output.status.success() {
        bail!(
            "`git diff` terminated with exit status {:?}",
            output.status.code().unwrap()
        );
    }
    let diff = String::from_utf8_lossy(&output.stdout);
    let file_changes = parse_diff(&diff)?;

    // Filter and report JSON diagnostic messages from standard input
    if let Some((subcommand_name, subcommand_args)) = subcommand {
        let output_kind = value_t!(matches, "output", OutputKind).unwrap_or(OutputKind::Rendered);

        let json_arg = if matches!(output_kind, OutputKind::GitHub) {
            // Colorless
            "--message-format=json"
        } else {
            // Colored
            "--message-format=json-diagnostic-rendered-ansi"
        };

        // Spawn the subprocess
        let mut child = Command::new(subcommand_name)
            .args(subcommand_args)
            .arg(json_arg)
            .args(&subcommand_extra_args)
            .stdout(Stdio::piped()) // filter stdout
            .stderr(Stdio::inherit()) // do not filter stderr
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to start subprocess {:?} with arguments {:?}",
                    subcommand, subcommand_args,
                )
            })?;

        // Process output
        let stdout = child
            .stdout
            .as_mut()
            .with_context(|| "Failed to open standard output of subprocess")?;
        process_stream(BufReader::new(stdout), &file_changes, output_kind)?;

        // Wait for end of subprocess
        let exit_status = child
            .wait()
            .with_context(|| "Failed to wait for subprocess")?;
        if !exit_status.success() {
            bail!(
                "Subprocess terminated with exit code {}",
                exit_status.code().unwrap_or(-1)
            )
        }
    } else {
        // Process standard input
        let output_kind = value_t!(matches, "output", OutputKind).unwrap_or(OutputKind::Json);
        process_stream(io::stdin().lock(), &file_changes, output_kind)?;
    }

    Ok(())
}

fn process_stream<T: BufRead>(
    stream: T,
    file_changes: &FileChanges,
    output: OutputKind,
) -> Result<()> {
    for maybe_line in stream.lines() {
        let json_line =
            maybe_line.with_context(|| "Failed to read line from standard output of subprocess")?;
        let diagnostic: Diagnostic = serde_json::from_str(&json_line).with_context(|| {
            format!("Failed to parse JSON from standard input: {:?}", json_line)
        })?;
        if should_report_diagnostic(&diagnostic, &file_changes) {
            report_diagnostic(&json_line, &diagnostic, output);
        }
    }
    Ok(())
}

/// Return `false` iff the message is a warning not related to changed lines.
fn should_report_diagnostic(diagnostic: &Diagnostic, file_changes: &FileChanges) -> bool {
    if let Some(ref message) = diagnostic.message {
        if matches!(message.level, Level::Warning) {
            let mut intersects_changes = false;
            for span in &message.spans {
                if let Some(file_changes) = file_changes.get(&span.file_name).as_ref() {
                    if intersect_intervals(span.line_start, span.line_end, file_changes) {
                        intersects_changes = true;
                        break;
                    }
                }
            }
            if !intersects_changes {
                return false;
            }
        }
    }
    true
}
