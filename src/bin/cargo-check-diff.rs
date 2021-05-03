use anyhow::Result;
use cargo_diff_tools::build_app;

fn main() -> Result<()> {
    build_app(env!("CARGO_BIN_NAME"), Some(("cargo", &["check"])))
}
