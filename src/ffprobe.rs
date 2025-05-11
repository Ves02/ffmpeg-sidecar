//! Utilities related to the FFprobe binary.

use crate::command::BackgroundCommand;
use anyhow::Context;
use std::{env::current_exe, ffi::OsStr, path::PathBuf};
use std::{
  path::Path,
  process::{Command, Stdio},
};

/// Returns the path of the downloaded FFprobe executable, or falls back to
/// assuming its installed in the system path. Note that not all FFmpeg
/// distributions include FFprobe.
pub fn ffprobe_path() -> PathBuf {
  let default = Path::new("ffprobe").to_path_buf();
  match ffprobe_sidecar_path() {
    Ok(sidecar_path) => match sidecar_path.exists() {
      true => sidecar_path,
      false => default,
    },
    Err(_) => default,
  }
}

/// The (expected) path to an FFmpeg binary adjacent to the Rust binary.
///
/// The extension between platforms, with Windows using `.exe`, while Mac and
/// Linux have no extension.
pub fn ffprobe_sidecar_path() -> anyhow::Result<PathBuf> {
  let mut path = current_exe()?
    .parent()
    .context("Can't get parent of current_exe")?
    .join("ffprobe");
  if cfg!(windows) {
    path.set_extension("exe");
  }
  Ok(path)
}

/// Alias for `ffprobe -version`, parsing the version number and returning it.
pub fn ffprobe_version() -> anyhow::Result<String> {
  ffprobe_version_with_path(ffprobe_path())
}

/// Lower level variant of `ffprobe_version` that exposes a customized the path
/// to the ffmpeg binary.
pub fn ffprobe_version_with_path<S: AsRef<OsStr>>(path: S) -> anyhow::Result<String> {
  let output = Command::new(&path)
    .arg("-version")
    .create_no_window()
    .output()?;

  // note:version parsing is not implemented for ffprobe

  Ok(String::from_utf8(output.stdout)?)
}

/// Verify whether ffprobe is installed on the system. This will return true if
/// there is an ffprobe binary in the PATH, or in the same directory as the Rust
/// executable.
pub fn ffprobe_is_installed() -> bool {
  Command::new(ffprobe_path())
    .create_no_window()
    .arg("-version")
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .status()
    .map(|s| s.success())
    .unwrap_or_else(|_| false)
}

/// A wrapper around [`std::process::Command`] with some convenient preset
/// argument sets and customization for `ffprobe` specifically.
///
/// The `rustdoc` on each method includes relevant information from the FFprobe
/// documentation: <https://ffmpeg.org/ffprobe.html>. Refer there for the
/// exhaustive list of possible arguments.
pub struct FfprobeCommand {
  inner: Command,
}

impl FfprobeCommand {
  //// Generic option aliases ////
  //// https://ffmpeg.org/ffprobe.html#Generic-options

  /// alias for `-hide_banner` argument.
  ///
  /// Suppress printing banner.
  ///
  /// All FFmpeg tools will normally show a copyright notice, build options and
  /// library versions. This option can be used to suppress printing this
  /// information.
  pub fn hide_banner(&mut self) -> &mut Self {
    self.arg("-hide_banner");
    self
  }

  /// alias for `-print_format` argument.
  ///
  /// Set the output printing format.
  ///
  /// writer_name specifies the name of the writer, and writer_options specifies
  /// the options to be passed to the writer.
  pub fn print_format<S: AsRef<str>>(&mut self, format: S) -> &mut Self {
    self.arg("-print_format");
    self.arg(format.as_ref());
    self
  }

  //// `std::process::Command` passthrough methods

  ///
  /// Adds an argument to pass to the program.
  ///
  /// Identical to `arg` in [`std::process::Command`].
  pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
    self.inner.arg(arg.as_ref());
    self
  }

  /// Adds multiple arguments to pass to the program.
  ///
  /// Identical to `args` in [`std::process::Command`].
  pub fn args<I, S>(&mut self, args: I) -> &mut Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    for arg in args {
      self.arg(arg.as_ref());
    }
    self
  }
}
