use owo_colors::OwoColorize;
use std::io::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl ColorMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "always" => Self::Always,
            "never" => Self::Never,
            _ => Self::Auto,
        }
    }

    pub fn is_enabled(self) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => std::io::stdout().is_terminal(),
        }
    }
}

pub struct Printer {
    pub color: bool,
    pub quiet: bool,
    pub verbose: bool,
}

impl Printer {
    pub fn new(color_mode: ColorMode, quiet: bool, verbose: bool) -> Self {
        Self {
            color: color_mode.is_enabled(),
            quiet,
            verbose,
        }
    }

    pub fn println(&self, msg: &str) {
        if !self.quiet {
            println!("{msg}");
        }
    }

    #[allow(dead_code)]
    pub fn eprintln(&self, msg: &str) {
        eprintln!("{msg}");
    }

    pub fn verbose(&self, msg: &str) {
        if self.verbose && !self.quiet {
            eprintln!("{msg}");
        }
    }

    pub fn success(&self, msg: &str) {
        if self.quiet {
            return;
        }
        if self.color {
            println!("{}", msg.green());
        } else {
            println!("{msg}");
        }
    }

    pub fn error(&self, msg: &str) {
        if self.color {
            eprintln!("{}: {}", "error".red().bold(), msg);
        } else {
            eprintln!("error: {msg}");
        }
    }

    pub fn warning(&self, msg: &str) {
        if self.color {
            eprintln!("{}: {}", "warning".yellow().bold(), msg);
        } else {
            eprintln!("warning: {msg}");
        }
    }

    pub fn hint(&self, msg: &str) {
        if self.quiet {
            return;
        }
        if self.color {
            eprintln!("  {}: {}", "hint".cyan(), msg);
        } else {
            eprintln!("  hint: {msg}");
        }
    }

    pub fn domain_name<'a>(&self, name: &'a str, is_current: bool) -> String {
        if !self.color {
            return name.to_owned();
        }
        if is_current {
            name.cyan().bold().to_string()
        } else {
            name.white().to_string()
        }
    }

    pub fn stale_text<'a>(&self, text: &'a str) -> String {
        if self.color {
            text.yellow().to_string()
        } else {
            text.to_owned()
        }
    }

    pub fn path_text<'a>(&self, text: &'a str) -> String {
        if self.color {
            text.blue().dimmed().to_string()
        } else {
            text.to_owned()
        }
    }

    pub fn change_id_text<'a>(&self, text: &'a str) -> String {
        if self.color {
            text.magenta().to_string()
        } else {
            text.to_owned()
        }
    }

    #[allow(dead_code)]
    pub fn dim<'a>(&self, text: &'a str) -> String {
        if self.color {
            text.dimmed().to_string()
        } else {
            text.to_owned()
        }
    }
}

/// Emit shell-integration IPC: cd directive on stdout.
pub fn emit_cd(path: &std::path::Path) {
    println!("RYOIKI:CD:{}", path.display());
}
