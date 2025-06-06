mod cli;
mod error;
mod utils;

shadow_rs::shadow!(build);

use std::{
  collections::HashSet,
  fs::{self, File},
  io::Write,
  path::PathBuf,
  sync::Arc,
};

use futures::StreamExt;
use indicatif::{
  MultiProgress,
  MultiProgressAlignment,
  ProgressBar,
  ProgressStyle,
};
use reqwest::Client;
use tokio::task;
use tracing::info;
use url::Url;

use crate::{cli::Cli, error::Result};

// Struct to hold downloader configuration and state
#[derive(Clone)]
pub struct Downloader {
  urls:       Vec<String>,
  dest:       PathBuf,
  workers:    usize,
  client:     Client,
  total_size: Arc<tokio::sync::Mutex<u64>>,
  clean:      bool,
  seen_urls:  Arc<tokio::sync::Mutex<HashSet<String>>>,
}

impl std::fmt::Debug for Downloader {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let truncate = if self.urls.len() > 3 {
      3
    } else {
      self.urls.len()
    };
    let urls = self.urls.clone().into_iter().take(truncate).collect::<Vec<_>>();
    let urls = format!(
      "[{}{}; {}]",
      urls.join(", "),
      if self.urls.len() > truncate {
        "..."
      } else {
        ""
      },
      self.urls.len()
    );
    f.debug_struct("Downloader")
      .field("urls", &urls)
      .field("dest", &self.dest)
      .field("workers", &self.workers)
      .field("total_size", &self.total_size)
      .field("clean", &self.clean)
      .finish()
  }
}

impl Default for Downloader {
  fn default() -> Self {
    Self {
      urls:       Default::default(),
      dest:       PathBuf::from(".")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(".")),
      workers:    std::thread::available_parallelism().unwrap().get(),
      client:     Default::default(),
      total_size: Default::default(),
      clean:      true,
      seen_urls:  Default::default(),
    }
  }
}

/// Downloader implementation
impl Downloader {
  /// Create a new Downloader
  pub fn new(
    urls: Vec<String>,
    dest: String,
    workers: usize,
    clean: bool,
  ) -> Self {
    let dest = shellexpand::tilde(&dest).to_string();
    let dest = PathBuf::from(dest.clone())
      .canonicalize()
      .unwrap_or_else(|_| PathBuf::from(dest.clone()));
    let client = Client::new();
    let total_size = Arc::new(tokio::sync::Mutex::new(0));
    let seen_urls = Arc::new(tokio::sync::Mutex::new(HashSet::new()));

    Downloader {
      urls,
      dest,
      workers,
      client,
      total_size,
      clean,
      seen_urls,
    }
  }

  /// Get the filename from a given URL.
  /// If the URL is invalid or the url has no path segments, return
  /// "downloaded_file"
  pub fn get_filename(url: &str) -> String {
    Url::parse(url)
      .ok()
      .and_then(|u| {
        u.path_segments()
          .map(|mut s| s.next_back().unwrap_or("downloaded_file").to_string())
      })
      .unwrap_or_else(|| "downloaded_file".to_string())
  }

  /// Shell Expand tilde in string
  pub fn shellexpand_tilde<T: AsRef<str>>(s: T) -> String {
    shellexpand::tilde(s.as_ref()).to_string()
  }

  /// Shell Expand environment variables and tilde home directory in string
  pub fn shellexpand_full<T: AsRef<str>>(s: T) -> String {
    use std::{borrow::Cow, env};
    fn context(s: &str) -> Option<Cow<'static, str>> {
      match env::var(s) {
        Ok(value) => Some(value.into()),
        Err(env::VarError::NotPresent) => Some("".into()),
        Err(_) => Some("".into()),
      }
    }
    fn home_dir() -> Option<String> {
      env::var("HOME").ok()
    }
    shellexpand::full_with_context_no_errors(s.as_ref(), home_dir, context)
      .to_string()
  }

  pub fn num_workers(&self) -> usize {
    self.workers
  }

  /// Get the number of URLs
  pub fn num_urls(&self) -> usize {
    self.urls.len()
  }

  #[allow(dead_code)]
  pub fn get_dest(&self) -> PathBuf {
    self.dest.clone()
  }

  #[allow(dead_code)]
  /// Set the list of URLs
  pub fn with_urls(mut self, urls: Vec<String>) -> Self {
    self.urls = urls
      .into_iter()
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .filter_map(|s| Url::parse(&s).ok().map(|u| u.to_string()))
      .collect::<Vec<_>>();
    self
  }

  #[allow(dead_code)]
  /// Set the number of worker threads
  pub fn with_workers(mut self, workers: usize) -> Self {
    self.workers = workers;
    self
  }

  #[allow(dead_code)]
  /// Set the destination directory
  pub fn with_dest<T: AsRef<str>>(mut self, dest: T) -> Self {
    let dest = shellexpand::tilde(dest.as_ref()).to_string();
    let dest = PathBuf::from(dest.clone())
      .canonicalize()
      .unwrap_or_else(|_| PathBuf::from(dest.clone()));
    self.dest = dest;
    self
  }

  #[allow(dead_code)]
  /// Set the reqwest client
  pub fn with_client(mut self, client: Client) -> Self {
    self.client = client;
    self
  }

  #[allow(dead_code)]
  /// Enable file cleanup
  pub fn clean(mut self) -> Self {
    self.clean = true;
    self
  }

  #[allow(dead_code)]
  /// Disable file cleanup
  pub fn no_clean(mut self) -> Self {
    self.clean = false;
    self
  }

  #[allow(dead_code)]
  /// Get the total size of all downloaded files
  pub async fn get_total_size_bytes(&self) -> u64 {
    *self.total_size.lock().await
  }

  #[allow(dead_code)]
  /// Get the total size of all downloaded files
  pub async fn get_total_size_human(&self) -> String {
    human_readable_size(*self.total_size.lock().await)
  }

  /// Get file size of the file at `url` from http HEAD request
  #[tracing::instrument(skip(self), fields(url), err(level = tracing::Level::ERROR))]
  async fn get_file_size(&self, url: &str) -> Result<u64> {
    let resp = self.client.head(url).send().await?;
    // Retry on 429
    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
      let random_t = rand::random_range(500..1500);
      tokio::time::sleep(tokio::time::Duration::from_millis(random_t)).await;
      return Box::pin(self.get_file_size(url)).await;
    }
    // Handle error
    match resp.error_for_status_ref() {
      Ok(_) => (),
      Err(e) => return Err(error::DownloadError::ReqwestError(e)),
    }
    // Get content length from response or response headers
    let content_len = resp
      .headers()
      .get("content-length")
      .map(|v| v.to_str().unwrap().parse::<u64>().unwrap())
      .or(resp.content_length())
      .unwrap_or(0);
    if resp.status().is_success() {
      if !self.seen_urls.lock().await.contains(url) {
        // Update total size and seen urls
        self.seen_urls.lock().await.insert(url.to_string());
        *self.total_size.lock().await += content_len;
      }
      return Ok(content_len);
    }
    Ok(content_len)
  }

  /// Download a single file at `url` and show progress bar in `mp` and updating
  /// `total_pb`.
  ///
  /// Returns Ok(()) on success
  ///
  /// Skips file if it already exists
  /// Resumes download if file already exists and is partially downloaded
  #[tracing::instrument(skip(self, mp, total_pb), fields(url), err(level = tracing::Level::ERROR))]
  pub async fn download_file(
    &self,
    url: String,
    mp: Arc<MultiProgress>,
    total_pb: ProgressBar,
  ) -> Result<()> {
    let filename = Self::get_filename(&url);
    let filepath = self.dest.join(&filename);
    let temp_filepath = filepath.with_extension(format!(
      "{}.part",
      filepath.extension().unwrap_or_default().to_string_lossy()
    ));
    // Skip if file exists
    if filepath.exists() {
      let pb = mp.add(ProgressBar::new(0));
      pb.finish_with_message(format!(
        "\x1b[93mExists\x1b[0m {} {}",
        filename, "✔"
      ));
      total_pb.inc(1); // Increment total progress for skipped files
      tokio::time::sleep(tokio::time::Duration::from_millis(
        rand::random_range(200..500),
      ))
      .await;
      pb.finish_and_clear();
      return Ok(());
    }

    // Get existing size for resume
    let start_byte = temp_filepath.metadata().map(|m| m.len()).unwrap_or(0);
    let mut file_total_size = self.get_file_size(&url).await?;
    // Update total size message for total progress bar tracker
    total_pb.set_message(human_readable_size(*self.total_size.lock().await));

    // Setup progress bar
    let pb = mp.add(ProgressBar::new(file_total_size));
    pb.set_style(
      ProgressStyle::default_bar()
        .template(
          "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} \
           ({eta}) {msg}",
        )?
        .progress_chars("+>-"),
    );
    pb.set_message(format!(
      "\x1b[93m{}\x1b[0m  {}",
      human_readable_size(file_total_size),
      filename,
    ));

    // Check if Resume download done
    if start_byte > 0 {
      pb.set_position(start_byte);
      if start_byte >= file_total_size {
        total_pb.inc(1); // Increment total progress for completed partials
        fs::rename(&temp_filepath, &filepath).unwrap_or(());
        pb.set_position(start_byte);
        pb.finish_with_message(format!(
          "\x1b[96mDone\x1b[0m \x1b[92m{}\x1b[0m  {} {}",
          human_readable_size(file_total_size),
          filename,
          "✔",
        ));
        tokio::time::sleep(tokio::time::Duration::from_millis(
          rand::random_range(500..1000),
        ))
        .await;
        pb.finish_and_clear();
        return Ok(());
      }
    }

    // Setup request with range header for resume
    let resp = self
      .client
      .get(&url)
      .header("Range", format!("bytes={}-", start_byte))
      .send()
      .await?;

    // Retry on 429
    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
      let random_t = rand::random_range(1000..3000);
      let retry_after = resp
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok().and_then(|s| s.parse::<u64>().ok()))
        .or(Some(random_t));
      if let Some(retry_after) = retry_after {
        pb.finish_and_clear();
        tokio::time::sleep(tokio::time::Duration::from_millis(retry_after))
          .await;
        return Box::pin(self.download_file(url, mp, total_pb)).await;
      }
    }

    // Handle other http error
    match resp.error_for_status_ref() {
      Ok(_) => (),
      Err(e) => return Err(error::DownloadError::ReqwestError(e)),
    }

    // Update total size if not already determined from HEAD
    if file_total_size == 0 && resp.status().is_success() {
      file_total_size = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok().and_then(|s| s.parse::<u64>().ok()))
        .unwrap_or(0);
      self.seen_urls.lock().await.insert(url);
      *self.total_size.lock().await += file_total_size;
      total_pb.set_message(human_readable_size(*self.total_size.lock().await));
    }

    // Open file for writing
    let mut file =
      File::options().create(true).append(true).open(&temp_filepath)?;

    // Stream chunks and write to file
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
      let chunk = chunk?;
      let chunk_len = chunk.len();
      file.write_all(&chunk)?;
      pb.inc(chunk_len as u64);
    }

    // Rename temp file to final location
    fs::rename(&temp_filepath, &filepath)?;
    pb.finish_with_message(format!(
      "\x1b[32mOk\x1b[0m \x1b[32m{}\x1b[0m  {} {}",
      human_readable_size(file_total_size),
      filename,
      "✔",
    ));
    total_pb.inc(1); // Increment total progress when download completes
    tokio::time::sleep(tokio::time::Duration::from_millis(rand::random_range(
      500..1000,
    )))
    .await;
    pb.finish_and_clear();

    Ok(())
  }

  /// Run the downloader and return Ok(()) on success
  ///
  /// Deletes the `self.dest` directory if `self.clean` is true
  /// Creates the `self.dest` directory if it does not exist
  ///
  /// Downloads files concurrently using `self.workers` workers
  /// Returns Ok(()) on success
  pub async fn run(self) -> Result<()> {
    if self.clean {
      fs::remove_dir_all(&self.dest).unwrap_or(());
    }
    fs::create_dir_all(&self.dest)?;

    let mp = Arc::new(MultiProgress::new());
    mp.set_alignment(MultiProgressAlignment::Top);
    let total_files = self.urls.len() as u64;
    let total_pb = mp.add(ProgressBar::new(total_files));
    let downloader = Arc::new(self.clone());

    // Total progress bar tracking files completed
    total_pb.set_style(
      ProgressStyle::default_bar()
        .template(
          "Total: [{elapsed_precise}] [{bar:40.green/yellow}] {pos}/{len} \
           files (Total size: {msg})",
        )?
        .progress_chars("#>-"),
    );
    total_pb
      .set_message(human_readable_size(*downloader.total_size.lock().await));

    // Create tasks with worker limit
    let semaphore = Arc::new(tokio::sync::Semaphore::new(self.workers));
    let tasks = self
      .urls
      .clone()
      .into_iter()
      .map(|url| {
        let mp = mp.clone();
        let semaphore = semaphore.clone();
        let total_pb = total_pb.clone();
        let downloader = downloader.clone();
        async move {
          let _permit = semaphore.acquire().await.unwrap();
          downloader.download_file(url.clone(), mp, total_pb).await.inspect_err(
            |e| {
              tracing::error!(
                "Error downloading file from: {} error: {:?}",
                url,
                e
              )
            },
          )
        }
      })
      .collect::<task::JoinSet<_>>();

    // Wait for all downloads
    let results = tasks.join_all().await;
    for res in results {
      if res.is_ok() {};
    }

    // Finish total progress bar
    total_pb.finish_with_message(human_readable_size(
      *downloader.total_size.lock().await,
    ));
    Ok(())
  }
}

/// Convert bytes to human-readable format
pub fn human_readable_size(bytes: u64) -> String {
  use humansize::{format_size, DECIMAL};
  format_size(bytes, DECIMAL)
}

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
  use clap::{CommandFactory, Parser};
  utils::init_tracing();
  info!("Multi File Downloader v{}", build::PKG_VERSION);

  let mut cmd = Cli::command();
  let cli = Cli::parse();

  if let Some(shell) = cli.completion {
    cli::generate_completions("multifiledownloader", shell.as_ref(), &mut cmd);
    return Ok(());
  }

  if cli.get_urls().is_empty() {
    eprintln!("Error: No URLs provided");
    std::process::exit(1);
  }

  let downloader = Downloader::new(
    cli.get_urls(),
    cli.get_dest(),
    cli.get_workers(),
    cli.get_clean(),
  );
  let c = downloader.clone();

  downloader.run().await?;
  info!("Download completed successfully");
  info!(
    "Downloaded {} files of size {} to {} using {} workers",
    c.num_urls(),
    c.get_total_size_human().await,
    c.get_dest().display(),
    c.num_workers(),
  );
  Ok(())
}
