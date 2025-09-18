// Thank you to https://github.com/bgreenwell/lstr/blob/main/src/view.rs
//! Implements the classic, non-interactive directory tree view.

// use crate::app::ViewArgs;
use crate::git;
// use crate::icons;
use crate::utils;
// use colored::{control, Colorize};
use ignore::{self, WalkBuilder};
use lscolors::LsColors;
// use lscolors::style;
use devicons::icon_for_file;
use nu_ansi_term::{Color, Style};
use std::{
    fmt, fs,
    io::{self, Write},
    path::PathBuf,
};

// Platform-specific import for unix permissions
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Defines the choices for the --color option.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ColorChoice {
    Always,
    #[default]
    Auto,
    Never,
}

/// Implements the Display trait for ColorChoice to show possible values in help messages.
impl fmt::Display for ColorChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorChoice::Always => write!(f, "always"),
            ColorChoice::Auto => write!(f, "auto"),
            ColorChoice::Never => write!(f, "never"),
        }
    }
}

/// Arguments for the classic `view` command.
#[derive(Debug, Default)]
pub struct ViewArgs {
    /// The path to the directory to display. Defaults to the current directory.
    // #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Specify when to use colorized output.
    // #[arg(long, value_name = "WHEN", default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,
    /// Maximum depth to descend in the directory tree.
    // #[arg(short = 'L', long)]
    pub level: Option<usize>,
    /// Display directories only.
    // #[arg(short = 'd', long)]
    pub dirs_only: bool,
    /// Display the size of files.
    // #[arg(short = 's', long)]
    pub size: bool,
    /// Display file permissions.
    // #[arg(short = 'p', long)]
    pub permissions: bool,
    /// Show all files, including hidden ones.
    // #[arg(short = 'a', long, help = "Show all files, including hidden ones")]
    pub all: bool,
    /// Respect .gitignore and other standard ignore files.
    // #[arg(short = 'g', long)]
    pub gitignore: bool,
    /// Show git status for files and directories.
    // #[arg(short = 'G', long)]
    pub git_status: bool,
    /// Display file-specific icons (requires a Nerd Font).
    // #[arg(long, help = "Display file-specific icons (requires a Nerd Font)")]
    pub icons: bool,
}

/// Executes the classic directory tree view
pub fn run(args: &ViewArgs, ls_colors: &LsColors) -> anyhow::Result<()> {
    // eprintln!("Running view with args: {:?}", args);
    if !args.path.is_dir() {
        anyhow::bail!("'{}' is not a directory.", args.path.display());
    }

    let canonical_root = fs::canonicalize(&args.path)?;

    //TODO: Change this to nu_protocol's color handling UseAnsiColoring::Auto/True/False
    // engine.get_config()?.use_ansi_coloring = true;

    // match args.color {
    //     ColorChoice::Always => control::set_override(true),
    //     ColorChoice::Never => control::set_override(false),
    //     ColorChoice::Auto => {}
    // }

    if writeln!(
        io::stdout(),
        "{}",
        //args.path.display().to_string().blue().bold()
        Style::new().bold().paint(args.path.display().to_string())
    )
    .is_err()
    {
        return Ok(());
    }

    let git_repo_status = if args.git_status {
        git::load_status(&canonical_root)?
    } else {
        None
    };
    let status_cache = git_repo_status.as_ref().map(|s| &s.cache);
    let repo_root = git_repo_status.as_ref().map(|s| &s.root);

    let mut builder = WalkBuilder::new(&args.path);
    builder.hidden(!args.all).git_ignore(args.gitignore);
    if let Some(level) = args.level {
        builder.max_depth(Some(level));
    }

    let mut dir_count = 0;
    let mut file_count = 0;

    for result in builder.build() {
        let entry = match result {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("ERROR: {}", err);
                continue;
            }
        };

        if entry.depth() == 0 {
            continue;
        }

        let is_dir = entry.file_type().is_some_and(|ft| ft.is_dir());
        if args.dirs_only && !is_dir {
            continue;
        }

        let git_status_str = if let (Some(cache), Some(root)) = (status_cache, repo_root) {
            if let Ok(canonical_entry) = entry.path().canonicalize() {
                if let Ok(relative_path) = canonical_entry.strip_prefix(root) {
                    cache
                        .get(relative_path)
                        .map(|s| {
                            let status_char = s.get_char();
                            let color = match s {
                                git::FileStatus::New | git::FileStatus::Renamed => {
                                    Color::Green.normal()
                                }
                                git::FileStatus::Modified | git::FileStatus::Typechange => {
                                    Color::Yellow.normal()
                                }
                                git::FileStatus::Deleted => Color::Red.normal(),
                                git::FileStatus::Conflicted => Color::LightRed.normal(),
                                git::FileStatus::Untracked => Color::Magenta.normal(),
                            };
                            // format!("{} ", status_char).color(color).to_string()
                            color.paint(format!("{status_char} ")).to_string()
                        })
                        .unwrap_or_else(|| "  ".to_string())
                } else {
                    "  ".to_string()
                }
            } else {
                "  ".to_string()
            }
        } else {
            String::new()
        };

        let metadata = if args.size || args.permissions {
            entry.metadata().ok()
        } else {
            None
        };
        let permissions_str = if args.permissions {
            let perms = if let Some(md) = &metadata {
                // <-- Use 'md' here
                #[cfg(unix)]
                {
                    // Use 'md' for Unix-specific logic
                    let mode = md.permissions().mode();
                    let file_type_char = if md.is_dir() { 'd' } else { '-' };
                    format!("{}{}", file_type_char, utils::format_permissions(mode))
                }
                #[cfg(not(unix))]
                {
                    // This line tells the compiler we've intentionally not used 'md' on non-Unix systems
                    let _ = md;
                    "----------".to_string()
                }
            } else {
                "----------".to_string()
            };
            format!("{} ", perms)
        } else {
            String::new()
        };

        let indent = "    ".repeat(entry.depth().saturating_sub(1));
        let name = entry.file_name().to_string_lossy();
        let icon_str = if args.icons {
            // let (icon, color) = icons::get_icon_for_path(entry.path(), is_dir);
            let icon_info = icon_for_file(&entry.path(), &None);
            // format!("{} ", icon.color(color))
            Style::new()
                .fg(lookup_ansi_color_style(icon_info.color))
                .paint(format!("{} ", icon_info.icon))
                .to_string()
        } else {
            String::new()
        };
        let size_str = if args.size && !is_dir {
            metadata
                .as_ref()
                .map(|m| format!(" ({})", utils::format_size(m.len())))
                .unwrap_or_default()
        } else {
            String::new()
        };

        // --- Corrected Logic Block ---
        let ls_style = ls_colors
            .style_for_path(entry.path())
            .cloned()
            .unwrap_or_default();
        //let mut styled_name = name.to_string().normal();
        // let mut styled_name = Style::new().normal().paint(name.to_string());
        let mut styled_name = Style::new();

        if let Some(fg) = ls_style.foreground {
            use lscolors::Color as LsColor;
            let color = match fg {
                LsColor::Black => Color::Black,
                LsColor::Red => Color::Red,
                LsColor::Green => Color::Green,
                LsColor::Yellow => Color::Yellow,
                LsColor::Blue => Color::Blue,
                LsColor::Magenta => Color::Magenta,
                LsColor::Cyan => Color::Cyan,
                LsColor::White => Color::White,
                LsColor::BrightBlack => Color::DarkGray,
                LsColor::BrightRed => Color::LightRed,
                LsColor::BrightGreen => Color::LightGreen,
                LsColor::BrightYellow => Color::LightYellow,
                LsColor::BrightBlue => Color::LightBlue,
                LsColor::BrightMagenta => Color::LightMagenta,
                LsColor::BrightCyan => Color::LightCyan,
                LsColor::BrightWhite => Color::LightGray,
                LsColor::Fixed(_) => Color::White,
                LsColor::RGB(r, g, b) => Color::Rgb(r, g, b),
            };
            // styled_name = styled_name.color(color);
            styled_name = styled_name.fg(color)
        }

        if ls_style.font_style.bold {
            // styled_name = styled_name.bold();
            styled_name = styled_name.bold();
        }
        if ls_style.font_style.italic {
            styled_name = styled_name.italic();
        }
        if ls_style.font_style.underline {
            styled_name = styled_name.underline();
        }
        // --- End Corrected Logic Block ---

        if is_dir {
            dir_count += 1;
        } else {
            file_count += 1;
        }

        if writeln!(
            io::stdout(),
            "{}{}{}└── {}{}{}",
            git_status_str,
            //permissions_str.dimmed(),
            Style::new().dimmed().paint(permissions_str),
            indent,
            icon_str,
            styled_name.paint(name),
            // size_str.dimmed()
            Style::new().dimmed().paint(size_str)
        )
        .is_err()
        {
            break;
        }
    }

    let summary = format!("\n{} directories, {} files", dir_count, file_count);
    _ = writeln!(io::stdout(), "{}", summary);

    Ok(())
}

pub fn lookup_ansi_color_style(s: &str) -> Color {
    if s.starts_with('#') {
        color_from_hex(s)
            .ok()
            // .and_then(|c| c.map(|c| c.normal()))
            // .and_then(|c| Some(c.normal()))
            .unwrap_or_default()
    } else {
        Color::Default
    }
}

pub fn color_from_hex(hex_color: &str) -> std::result::Result<Color, std::num::ParseIntError> {
    // right now we only allow hex colors with hashtag and 6 characters
    let trimmed = hex_color.trim_matches('#');
    if trimmed.len() != 6 {
        Ok(Color::Default)
    } else {
        // make a nu_ansi_term::Color::Rgb color by converting hex to decimal
        Ok(Color::Rgb(
            u8::from_str_radix(&trimmed[..2], 16)?,
            u8::from_str_radix(&trimmed[2..4], 16)?,
            u8::from_str_radix(&trimmed[4..6], 16)?,
        ))
    }
}
