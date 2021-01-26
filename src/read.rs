extern crate nix;
use crate::extra;
use std::process::{Command, Stdio};
use std::{fs, io};
use nix::unistd;

/// Read battery percentage from __/sys/class/power_supply/BAT0/capacity__
pub fn read_battery_percentage() -> String {
    let mut percentage = fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .expect("Could not read battery percentage from /sys/class/power_supply/BAT0/capacity");
    percentage = extra::pop_newline(percentage);
    String::from(&percentage)
}

/// Read battery status from __/sys/class/power_supply/BAT0/status__
pub fn read_battery_status() -> String {
    let mut status = fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .expect("Could not read battery percentage from /sys/class/power_supply/BAT0/status");
    status = extra::pop_newline(status);
    status
}

/// Read current terminal instance using __ps__ command
pub fn read_terminal() -> String {
    //  ps -p $$ -o ppid=
    //  $$ doesn't work natively in rust but its value can be
    //  accessed through nix::unistd::getppid()

    let ppid = Command::new("ps")
        .arg("-p")
        .arg(unistd::getppid().to_string())
        .arg("-o")
        .arg("ppid=")
        .output()
        .expect("Failed to get current terminal instance PPID using 'ps -p <PID> o ppid='");

    let terminal_ppid = String::from_utf8(ppid.stdout)
        .expect("'ps' process stdout wasn't valid UTF-8")
        .trim()
        .to_string();

    let name = Command::new("ps")
        .arg("-p")
        .arg(terminal_ppid)
        .arg("o")
        .arg("comm=")
        .output()
        .expect("Failed to get current terminal instance name using 'ps -p <PID> o comm='");

    let terminal_name =
        String::from_utf8(name.stdout).expect("'ps' process stdout wasn't valid UTF-8");
    terminal_name.trim().to_string()
}

/// Read current shell instance name using __ps__ command
pub fn read_shell(shorthand: bool) -> String {
    //  ps -p $$ -o comm=
    //  $$ doesn't work natively in rust but its value can be
    //  accessed through nix::unistd::getppid()
    if shorthand {
        let output = Command::new("ps")
            .arg("-p")
            .arg(unistd::getppid().to_string())
            .arg("o")
            .arg("comm=")
            .output()
            .expect("Failed to get current shell instance name 'ps -p <PID> o args='");

        let shell_name = String::from_utf8(output.stdout)
            .expect("read_terminal: stdout to string conversion failed");
        return shell_name.trim().to_string();
    }

    // If shell shorthand is false, we use "args=" instead of "comm="
    // to print the full path of the current shell instance name
    let output = Command::new("ps")
        .arg("-p")
        .arg(unistd::getppid().to_string())
        .arg("o")
        .arg("args=")
        .output()
        .expect("Failed to get current shell instance name 'ps -p <PID> o args='");

    let shell_name = String::from_utf8(output.stdout)
        .expect("read_terminal: stdout to string conversion failed");
    shell_name.trim().to_string()
}

pub fn read_package_count() -> usize {
    let pacman = Command::new("pacman")
        .arg("-Q")
        .arg("-q")
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to start 'pacman' process");

    let pac_out =
        String::from_utf8(pacman.stdout).expect("'pacman' process stdout wasn't valid UTF-8");
    let packages: Vec<&str> = pac_out.split('\n').collect();

    packages.len() - 1
}

/// Read kernel version by calling "uname -r"
pub fn read_kernel_version() -> String {
    let output = Command::new("uname")
        .arg("-r")
        .output()
        .expect("Failed to get kernel release using 'uname -r'");

    let kern_vers = String::from_utf8(output.stdout)
        .expect("read_kernel_version: stdout to string conversion failed");
    kern_vers.trim().to_string()
}

/// Read hostname by calling "uname -n"
pub fn read_hostname() -> String {
    let mut buf = [0u8; 64];
    let hostname_cstr = unistd::gethostname(&mut buf).expect("Failed getting hostname");
    let hostname = hostname_cstr.to_str().expect("Hostname wasn't valid UTF-8");
    hostname.to_string()
}

/// Read operating system name from __/etc/os-release__
pub fn read_operating_system() -> String {
    let mut os = String::from(
        extra::get_line_at("/etc/os-release", 0, "Could not extract OS name!").unwrap(),
    );
    if !os.contains("NAME=\"") {
        return os.replace("NAME=", "");
    }
    os.pop();
    os.replace("NAME=\"", "")
}

/// Read processor information from __/proc/cpuinfo__
pub fn read_cpu_model_name(shorthand: bool) -> String {
    let mut cpu = String::from(
        extra::get_line_at("/proc/cpuinfo", 4, "Could not extract CPU model name!").unwrap(),
    );
    cpu = cpu
        .replace("model name", "")
        .replace(":", "")
        .trim()
        .to_string();
    if shorthand && cpu.contains("Intel(R) Core(TM)") {
        cpu = cpu.replace("Intel(R) Core(TM)", "").replace("CPU ", "");
        return cpu.trim().to_string();
    }
    cpu
}

/// Read first float (uptime) from __/proc/uptime
pub fn read_uptime() -> Result<String, io::Error> {
    let uptime = fs::read_to_string("/proc/uptime")?;
    //  Read first float from uptime
    let up = uptime.split_whitespace().next().unwrap_or("").to_string();
    Ok(up)
}