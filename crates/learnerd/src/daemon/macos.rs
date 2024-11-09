use super::*;

pub const DEFAULT_PID_FILE: &str = "/Library/Application Support/learnerd/learnerd.pid";
pub const DEFAULT_WORKING_DIR: &str = "/Library/Application Support/learnerd";
pub const DEFAULT_LOG_DIR: &str = "/Library/Logs/learnerd";

// Constants for service naming
pub const SERVICE_NAME: &str = "learnerd.daemon";
pub const SERVICE_FILE: &str = "learnerd.daemon.plist";

pub fn install_system_daemon(daemon: &Daemon) -> Result<(), LearnerdErrors> {
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
</dict>
</plist>"#,
    SERVICE_NAME,
    std::env::current_exe()?.display(),
    daemon.working_dir.display(),
    daemon.log_dir.display(),
    daemon.log_dir.display(),
  );

  Ok(fs::write(format!("/Library/LaunchDaemons/{}", SERVICE_FILE), plist)?)
}

pub fn uninstall_system_daemon() -> Result<(), LearnerdErrors> {
  Ok(fs::remove_file(format!("/Library/LaunchDaemons/{}", SERVICE_FILE))?)
}

pub fn daemon_install_prompt(daemon: &Daemon) {
  println!("{} Daemon service installed", style(SUCCESS).green());

  println!("\n{} To activate the service:", style("Next steps").blue());
  println!(
    "   1. Load:     {}",
    style(format!("sudo launchctl load /Library/LaunchDaemons/{}", SERVICE_FILE)).yellow()
  );
  println!("   2. Verify:   {}", style("sudo launchctl list | grep learnerd").yellow());

  println!("\n{} Troubleshooting commands:", style("Debug").blue());
  println!(
    "   View logs:     {}",
    style(format!("tail -f {}/stdout.log", daemon.log_dir.display())).yellow()
  );
  println!(
    "   Check status:  {}",
    style(format!("sudo launchctl print system/{}", SERVICE_NAME)).yellow()
  );
  println!("   List service:  {}", style("sudo launchctl list | grep learnerd").yellow());

  println!("\n{} Service management:", style("Control").blue());
  println!(
    "   Stop:          {}",
    style(format!("sudo launchctl bootout system/{}", SERVICE_NAME)).yellow()
  );
  println!(
    "   Start:         {}",
    style(format!("sudo launchctl bootstrap system /Library/LaunchDaemons/{}", SERVICE_FILE))
      .yellow()
  );
  println!(
    "   Restart:       {}",
    style(format!("sudo launchctl kickstart -k system/{}", SERVICE_NAME)).yellow()
  );

  println!("\n{} Service paths:", style("Configuration").blue());
  println!("   Working dir: {}", style(daemon.working_dir.display()).yellow());
  println!("   PID file:    {}", style(daemon.pid_file.display()).yellow());
  println!("   Log dir:     {}", style(daemon.log_dir.display()).yellow());
}
