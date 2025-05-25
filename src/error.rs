use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
  #[error("Failed to parse URL: {0}")]
  UrlParseError(#[from] url::ParseError),

  #[error("HTTP request failed: {0}")]
  ReqwestError(#[from] reqwest::Error),

  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),

  #[error("Failed to create destination directory: {0}")]
  DestDirError(String),

  #[error("File already exists and cannot be resumed: {0}")]
  FileExistsError(String),

  #[error("Invalid URL format: {0}")]
  InvalidUrlError(String),

  #[error("Indicatif error: {0}")]
  IndicatifError(#[from] indicatif::style::TemplateError),
}

pub type Result<T> = std::result::Result<T, DownloadError>;
