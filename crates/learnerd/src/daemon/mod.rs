//! Daemon implementation for the learnerd service.
//!
//! This module provides functionality for running learnerd as a system daemon,
//! supporting both systemd (Linux) and launchd (macOS) environments. It handles:
//! - Daemon process management (start/stop/restart)
//! - System service installation
//! - Logging configuration
//! - Platform-specific requirements

use std::{fs, path::PathBuf};

use nix::{
  sys::signal::{self, Signal},
  unistd::Pid,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use tracing_appender::rolling;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use super::*;

#[cfg(target_os = "linux")] pub mod linux;
#[cfg(target_os = "linux")] pub use linux::*;
#[cfg(target_os = "macos")] pub mod macos;
#[cfg(target_os = "macos")] pub use macos::*;

/// Subcommands for daemon management
#[derive(Subcommand)]
pub enum DaemonCommands {
  /// Start the daemon
  Start,
  /// Stop the daemon
  Stop,
  /// Restart the daemon
  Restart,
  /// Install daemon as system service
  Install,
  /// Remove daemon from system services
  Uninstall,
  /// Show daemon status
  Status,
}

/// Configuration for the daemon service
#[derive(Debug, Serialize, Deserialize)]
pub struct Daemon {
  /// Path to store the PID file
  pub pid_file:    PathBuf,
  /// Working directory for the daemon
  pub working_dir: PathBuf,
  /// Directory for log files
  pub log_dir:     PathBuf,
}

impl Default for Daemon {
  fn default() -> Self {
    Self {
      pid_file:    PathBuf::from(DEFAULT_PID_FILE),
      working_dir: PathBuf::from(DEFAULT_WORKING_DIR),
      log_dir:     PathBuf::from(DEFAULT_LOG_DIR),
    }
  }
}

impl Daemon {
  /// Creates a new daemon instance with default configuration
  pub fn new() -> Self { Self::default() }

  /// Starts the daemon process
  pub fn start(&self) -> Result<(), LearnerdErrors> {
    // Ensure directories exist
    fs::create_dir_all(&self.working_dir)?;
    fs::create_dir_all(&self.log_dir)?;

    // Configure file logging
    let file_appender = rolling::RollingFileAppender::builder()
      .rotation(rolling::Rotation::DAILY)
      .filename_prefix("learnerd")
      .filename_suffix("log")
      .build(&self.log_dir)?;

    // Create a file layer for file logging
    let file_layer = tracing_subscriber::fmt::layer()
      .with_writer(file_appender)
      .with_ansi(false)
      .with_thread_ids(true)
      .with_target(true)
      .with_file(true)
      .with_line_number(true);

    // Create a stdout layer for systemd/journal capture
    let stdout_layer = tracing_subscriber::fmt::layer().with_ansi(false).with_target(true);

    // Initialize both layers
    tracing_subscriber::registry()
      .with(file_layer)
      .with(stdout_layer)
      .with(EnvFilter::new("debug"))
      .init();

    info!("Starting learnerd daemon");
    debug!("Using config: {:?}", self);

    info!("Daemon started successfully");
    self.run()
  }

  // TODO (autoparallel): this is actually never really able to be used at the moment.
  /// Stops the daemon process
  pub fn stop(&self) -> Result<(), LearnerdErrors> {
    if let Ok(pid) = fs::read_to_string(&self.pid_file) {
      let pid: i32 = pid.trim().parse().map_err(|e: std::num::ParseIntError| {
        LearnerdErrors::Daemon(format!("pid.trim().parse() gave error: {}", e))
      })?;

      #[cfg(unix)]
      {
        // Send SIGTERM to the process
        if let Err(e) = signal::kill(Pid::from_raw(pid), Signal::SIGTERM) {
          error!("Failed to send SIGTERM to process: {}", e);
          return Err(LearnerdErrors::Daemon(format!("Failed to stop daemon: {}", e)));
        }
      }

      if let Err(e) = fs::remove_file(&self.pid_file) {
        error!("Failed to remove PID file: {}", e);
      }

      Ok(())
    } else {
      error!("PID file not found");
      Err(LearnerdErrors::Daemon("Daemon not running".to_string()))
    }
  }

  /// Restarts the daemon process
  pub fn restart(&self) -> Result<(), LearnerdErrors> {
    self.stop()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    self.start()
  }

  /// Installs the daemon as a system service
  pub fn install(&self) -> Result<(), LearnerdErrors> { install_system_daemon(&self) }

  /// Removes the daemon from system services
  pub fn uninstall(&self) -> Result<(), LearnerdErrors> { uninstall_system_daemon() }

  /// Main daemon loop
  fn run(&self) -> Result<(), LearnerdErrors> {
    info!("Daemon running");

    // TODO: Implement actual daemon functionality
    loop {
      std::thread::sleep(std::time::Duration::from_secs(5));
      debug!("Daemon heartbeat");
    }
  }
}
