use super::*;

pub const DEFAULT_PID_FILE: &str = "/var/run/learnerd.pid";
pub const DEFAULT_WORKING_DIR: &str = "/var/lib/learnerd";
pub const DEFAULT_LOG_DIR: &str = "/var/log/learnerd";

pub fn install_system_daemon(_daemon: &Daemon) -> Result<(), LearnerdErrors> {
  let service = format!(
    r#"[Unit]
Description=Academic Paper Management Daemon
After=network.target
Documentation=https://github.com/autoparallel/learner

[Service]
Type=simple
User=root
Group=root
ExecStart=/usr/local/bin/learnerd daemon start
Restart=on-failure
RestartSec=60
RemainAfterExit=yes

# Logging configuration
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
"#
  );

  // Install the binary to /usr/local/bin if it's not there
  if let Ok(current_exe) = std::env::current_exe() {
    if current_exe.to_str().unwrap_or("").contains(".cargo") {
      std::process::Command::new("cp")
        .args([current_exe.to_str().unwrap(), "/usr/local/bin/learnerd"])
        .output()?;
      std::process::Command::new("chmod").args(["755", "/usr/local/bin/learnerd"]).output()?;
    }
  }

  fs::write("/etc/systemd/system/learnerd.service", service)?;

  // Reload systemd
  std::process::Command::new("systemctl").args(["daemon-reload"]).output()?;
  Ok(())
}

pub fn uninstall_system_daemon() -> Result<(), LearnerdErrors> {
  Ok(fs::remove_file("/etc/systemd/system/learnerd.service")?)
}

pub fn daemon_install_prompt(daemon: &Daemon) {
  println!("{} Daemon service installed", style(SUCCESS).green());

  println!("\n{} To activate the service:", style("Next steps").blue());
  println!("   1. Reload:   {}", style("sudo systemctl daemon-reload").yellow());
  println!("   2. Enable:   {}", style("sudo systemctl enable learnerd").yellow());
  println!("   3. Start:    {}", style("sudo systemctl start learnerd").yellow());
  println!("   4. Verify:   {}", style("sudo systemctl status learnerd").yellow());

  println!("\n{} Troubleshooting commands:", style("Debug").blue());
  println!("   View logs:     {}", style("sudo journalctl -u learnerd -f").yellow());
  println!(
    "   Check paths:   {}",
    style("sudo systemctl show learnerd -p ExecStart,PIDFile,RuntimeDirectory").yellow()
  );
  println!("   Check status:  {}", style("sudo systemctl status learnerd --no-pager -l").yellow());

  println!("\n{} Service paths:", style("Configuration").blue());
  println!("   Working dir: {}", style(daemon.working_dir.display()).yellow());
  println!("   PID file:    {}", style(daemon.pid_file.display()).yellow());
  println!("   Log dir:     {}", style(daemon.log_dir.display()).yellow());
}
