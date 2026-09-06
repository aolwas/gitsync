use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Verbose output
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub verbose: bool,

    /// Colorize output (always, never, auto)
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

    /// Dry run (show what would be done without making changes)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,

    /// Continue on error (process all branches even if some fail)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub continue_on_error: bool,

    /// Remote to sync with (overrides auto-detection)
    #[arg(short, long)]
    pub remote: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ColorChoice {
    Always,
    Never,
    Auto,
}
