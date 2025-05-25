use clap::Parser;
use clap_complete::{generate, shells};
use url::Url;

#[derive(
  Debug, Clone, Copy, clap::ValueEnum, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
  Bash,
  Zsh,
  Fish,
  Powershell,
  Elvish,
}

impl std::str::FromStr for Shell {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "bash" => Ok(Shell::Bash),
      "zsh" => Ok(Shell::Zsh),
      "fish" => Ok(Shell::Fish),
      "powershell" => Ok(Shell::Powershell),
      "elvish" => Ok(Shell::Elvish),
      _ => Err(format!("Unknown shell: {}", s)),
    }
  }
}

impl AsRef<str> for Shell {
  fn as_ref(&self) -> &str {
    match self {
      Shell::Bash => "bash",
      Shell::Zsh => "zsh",
      Shell::Fish => "fish",
      Shell::Powershell => "powershell",
      Shell::Elvish => "elvish",
    }
  }
}

impl std::fmt::Display for Shell {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Shell::Bash => write!(f, "bash"),
      Shell::Zsh => write!(f, "zsh"),
      Shell::Fish => write!(f, "fish"),
      Shell::Powershell => write!(f, "powershell"),
      Shell::Elvish => write!(f, "elvish"),
    }
  }
}

#[derive(Parser, Debug, Clone)]
#[command(
  author = env!("CARGO_PKG_AUTHORS"),
  name = "multifiledownloader",
  version=crate::build::CLAP_LONG_VERSION,
  about="A concurrent and configurable multi-file downloader with progress tracking",
  long_about = None,
)]
pub struct Cli {
  #[arg(
    short,
    long,
    help = "Comma-separated list of URLs to download",
    required_unless_present = "completion",
    default_value = ""
  )]
  urls: String,

  #[arg(short, long, default_value = ".", help = "Destination folder")]
  pub dest: String,

  #[arg(
    short,
    long,
    default_value_t = 8,
    help = "Number of concurrent workers"
  )]
  pub workers: usize,

  #[arg(
    short,
    long,
    default_value_t = false,
    help = "Clean destination folder if it exists"
  )]
  pub clean: bool,

  #[arg(
    long,
    alias = "compl",
    alias = "generate-completions",
    help = "Shell to generate completion script for."
  )]
  pub completion: Option<Shell>,
}

impl Cli {
  pub fn get_urls(&self) -> Vec<String> {
    self
      .urls
      .split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .filter_map(|url| Url::parse(&url).ok().map(|u| u.to_string()))
      .collect()
  }

  pub fn get_dest(&self) -> String {
    shellexpand::tilde(&self.dest).to_string()
  }

  pub fn get_workers(&self) -> usize {
    self.workers
  }

  pub fn get_clean(&self) -> bool {
    self.clean
  }
}

/// Generate shell completions for the CLI
pub fn generate_completions<S: AsRef<str>>(
  bin_name: S,
  shell: S,
  cmd: &mut clap::Command,
) {
  match shell.as_ref().to_lowercase().as_str() {
    "bash" => {
      generate(shells::Bash, cmd, bin_name.as_ref(), &mut std::io::stdout())
    },
    "zsh" => {
      generate(shells::Zsh, cmd, bin_name.as_ref(), &mut std::io::stdout())
    },
    "fish" => {
      generate(shells::Fish, cmd, bin_name.as_ref(), &mut std::io::stdout())
    },
    "powershell" => generate(
      shells::PowerShell,
      cmd,
      bin_name.as_ref(),
      &mut std::io::stdout(),
    ),
    "elvish" => {
      generate(shells::Elvish, cmd, bin_name.as_ref(), &mut std::io::stdout())
    },
    _ => println!("Unsupported shell {}", shell.as_ref()),
  }
}
