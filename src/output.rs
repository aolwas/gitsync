use colored::*;

use crate::cli::ColorChoice;

pub struct OutputManager {
    verbose: bool,
    use_color: bool,
}

impl OutputManager {
    pub fn new(verbose: bool, color_choice: ColorChoice) -> Self {
        let use_color = match color_choice {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => atty::is(atty::Stream::Stdout),
        };

        Self { verbose, use_color }
    }

    pub fn verbose(&self, message: &str) {
        if self.verbose {
            if self.use_color {
                println!("{}", message.magenta());
            } else {
                println!("{}", message);
            }
        }
    }

    pub fn info(&self, message: &str) {
        if self.use_color {
            println!("{}", message.green());
        } else {
            println!("{}", message);
        }
    }

    pub fn warning(&self, message: &str) {
        if self.use_color {
            eprintln!("{}", message.yellow());
        } else {
            eprintln!("{}", message);
        }
    }

    pub fn success(&self, message: &str) {
        if self.use_color {
            println!("{}", message.green());
        } else {
            println!("{}", message);
        }
    }
}
