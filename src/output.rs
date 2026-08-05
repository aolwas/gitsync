use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::ColorChoice;

pub struct OutputManager {
    verbose: bool,
    use_color: bool,
    progress_bar: Option<ProgressBar>,
}

impl OutputManager {
    pub fn new(verbose: bool, color_choice: ColorChoice) -> Self {
        let use_color = match color_choice {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => atty::is(atty::Stream::Stdout),
        };

        Self {
            verbose,
            use_color,
            progress_bar: None,
        }
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

    pub fn start_progress(&mut self, total: u64, message: &str) {
        // Always create progress bar, but only show it in interactive terminals
        let pb = ProgressBar::new(total);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("##-"));
        pb.set_message(message.to_string());
        
        // Only show progress bar in interactive terminals
        if !atty::is(atty::Stream::Stdout) {
            pb.set_draw_target(indicatif::ProgressDrawTarget::hidden());
        }
        
        self.progress_bar = Some(pb);
    }

    pub fn update_progress(&self, delta: u64) {
        if let Some(pb) = &self.progress_bar {
            pb.inc(delta);
        }
    }

    pub fn finish_progress(&mut self, message: &str) {
        if let Some(pb) = self.progress_bar.take() {
            pb.finish_with_message(message.to_string());
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
