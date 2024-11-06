//! Daemon implementation for the learnerd service.
//!
//! This module provides functionality for running learnerd as a system daemon,
//! supporting both systemd (Linux) and launchd (macOS) environments. It handles:
//! - Daemon process management (start/stop/restart)
//! - System service installation
//! - Logging configuration
//! - Platform-specific requirements

use std::{
  fs::{self, File},
  path::PathBuf,
};

use daemonize::Daemonize;
use nix::{
  sys::signal::{self, Signal},
  unistd::Pid,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use tracing_appender::rolling;

use super::*;
use crate::errors::LearnerdErrors;

// Constants for service naming
pub const SERVICE_NAME: &str = "learnerd.daemon";
pub const SERVICE_FILE: &str = "learnerd.daemon.plist";

// TODO (autoparallel): group this up better
#[cfg(target_os = "linux")]
/// Default paths for daemon-related files
const DEFAULT_PID_FILE: &str = "/var/run/learnerd.pid";
const DEFAULT_WORKING_DIR: &str = "/var/lib/learnerd";
const DEFAULT_LOG_DIR: &str = "/var/log/learnerd";

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
pub struct DaemonConfig {
  /// Path to store the PID file
  pub pid_file:    PathBuf,
  /// Working directory for the daemon
  pub working_dir: PathBuf,
  /// Directory for log files
  pub log_dir:     PathBuf,
}

impl Default for DaemonConfig {
  fn default() -> Self {
    // Use platform-specific paths
    #[cfg(target_os = "macos")]
    {
      Self {
        pid_file:    PathBuf::from("/Library/Application Support/learnerd/learnerd.pid"),
        working_dir: PathBuf::from("/Library/Application Support/learnerd"),
        log_dir:     PathBuf::from("/Library/Logs/learnerd"),
      }
    }
    #[cfg(target_os = "linux")]
    {
      Self {
        pid_file:    PathBuf::from(DEFAULT_PID_FILE),
        working_dir: PathBuf::from(DEFAULT_WORKING_DIR),
        log_dir:     PathBuf::from(DEFAULT_LOG_DIR),
      }
    }
  }
}

/// Manages the daemon process and its lifecycle
pub struct Daemon {
  pub config: DaemonConfig,
}

impl Daemon {
  /// Creates a new daemon instance with default configuration
  pub fn new() -> Self { Self { config: DaemonConfig::default() } }

  /// Starts the daemon process
  pub fn start(&self) -> Result<(), LearnerdErrors> {
    // Ensure directories exist
    fs::create_dir_all(&self.config.working_dir)?;
    fs::create_dir_all(&self.config.log_dir)?;

    // Configure file logging
    let file_appender = rolling::RollingFileAppender::builder()
      .rotation(rolling::Rotation::NEVER) // TODO (autoparallel): This should be rotated, but I changed this so that the files are named more easily for now.
      .filename_prefix("learnerd")
      .filename_suffix("log")
      .build(&self.config.log_dir)?;

    // Initialize daemon logger
    tracing_subscriber::fmt()
      .with_writer(file_appender)
      .with_ansi(false)
      .with_thread_ids(true)
      .with_target(true)
      .with_file(true)
      .with_line_number(true)
      .with_env_filter(EnvFilter::new("debug")) // TODO (autoparallel): Make this configurable?
      .init();

    info!("Starting learnerd daemon");
    debug!("Using config: {:?}", self.config);

    let stdout = File::create(self.config.log_dir.join("stdout.log"))?;
    let stderr = File::create(self.config.log_dir.join("stderr.log"))?;

    let daemonize = Daemonize::new()
      .pid_file(&self.config.pid_file)
      .chown_pid_file(true)
      .working_directory(&self.config.working_dir)
      .stdout(stdout)
      .stderr(stderr);

    match daemonize.start() {
      Ok(_) => {
        info!("Daemon started successfully");
        self.run()?;
        Ok(())
      },
      Err(e) => {
        error!("Failed to start daemon: {}", e);
        Err(LearnerdErrors::Daemon(e.to_string()))
      },
    }
  }

  /// Stops the daemon process
  pub fn stop(&self) -> Result<(), LearnerdErrors> {
    if let Ok(pid) = fs::read_to_string(&self.config.pid_file) {
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

      if let Err(e) = fs::remove_file(&self.config.pid_file) {
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
  pub fn install(&self) -> Result<(), LearnerdErrors> {
    #[cfg(target_os = "linux")]
    {
      self.install_systemd_service()?;
    }
    #[cfg(target_os = "macos")]
    {
      self.install_launchd_service()?;
    }
    Ok(())
  }

  /// Removes the daemon from system services
  pub fn uninstall(&self) -> Result<(), LearnerdErrors> {
    #[cfg(target_os = "linux")]
    {
      self.uninstall_systemd_service()?;
    }
    #[cfg(target_os = "macos")]
    {
      self.uninstall_launchd_service()?;
    }
    Ok(())
  }

  /// Main daemon loop
  fn run(&self) -> Result<(), LearnerdErrors> {
    info!("Daemon running");

    // TODO: Implement actual daemon functionality
    loop {
      std::thread::sleep(std::time::Duration::from_secs(5));
      debug!("Daemon heartbeat");
    }
  }

  #[cfg(target_os = "linux")]
  fn install_systemd_service(&self) -> Result<(), LearnerdErrors> {
    let service = format!(
      r#"[Unit]
Description=Academic Paper Management Daemon
After=network.target
Documentation=https://github.com/autoparallel/learner

[Service]
Type=forking
User=root
Group=root
PIDFile={}
ExecStart={} daemon start
ExecStop={} daemon stop
Restart=on-failure
RestartSec=60

# Security settings
NoNewPrivileges=yes
ProtectSystem=full
ProtectHome=read-only
PrivateTmp=yes
PrivateDevices=yes

# Logging
StandardOutput=append:{}
StandardError=append:{}

[Install]
WantedBy=multi-user.target
"#,
      self.config.pid_file.display(),
      std::env::current_exe()?.display(),
      std::env::current_exe()?.display(),
      self.config.log_dir.join("stdout.log").display(),
      self.config.log_dir.join("stderr.log").display(),
    );

    fs::write("/etc/systemd/system/learnerd.service", service)?;
    Ok(())
  }

  #[cfg(target_os = "macos")]
  fn install_launchd_service(&self) -> Result<(), LearnerdErrors> {
    let plist = format!(
      r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{}</string>
  <key>ProgramArguments</key>
  <array>
      <string>{}</string>
      <string>daemon</string>
      <string>start</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <dict>
      <key>SuccessfulExit</key>
      <false/>
      <key>Crashed</key>
      <true/>
  </dict>
  <key>ThrottleInterval</key>
  <integer>60</integer>
  <key>WorkingDirectory</key>
  <string>{}</string>
  <key>StandardOutPath</key>
  <string>{}/stdout.log</string>
  <key>StandardErrorPath</key>
  <string>{}/stderr.log</string>
  <key>ProcessType</key>
  <string>Background</string>
  <key>HardStopExec</key>
  <string>{} daemon stop</string>
</dict>
</plist>"#,
      SERVICE_NAME,
      std::env::current_exe()?.display(),
      self.config.working_dir.display(),
      self.config.log_dir.display(),
      self.config.log_dir.display(),
      std::env::current_exe()?.display(),
    );

    fs::write(format!("/Library/LaunchDaemons/{}", SERVICE_FILE), plist)?;
    Ok(())
  }

  #[cfg(target_os = "linux")]
  fn uninstall_systemd_service(&self) -> Result<(), LearnerdErrors> {
    fs::remove_file("/etc/systemd/system/learnerd.service")?;
    Ok(())
  }

  #[cfg(target_os = "macos")]
  fn uninstall_launchd_service(&self) -> Result<(), LearnerdErrors> {
    fs::remove_file("/Library/LaunchDaemons/com.autoparallel.learnerd.plist")?;
    Ok(())
  }
}
