//! File › Print › Printing preferences: the selected printer driver's settings for printing
//! (paper type, print quality, colour).
//!
//! - Windows: the driver's Printing Preferences window (`printui.dll`): paper type, quality.
//! - Linux and the BSDs: system-config-printer's printer window (its Printer Options page has
//!   the paper and quality defaults) through its D-Bus service (Linux Mint, Ubuntu, Fedora…); without it, the printer's page in the CUPS web
//!   interface.
//! - macOS: System Settings › Printers & Scanners.
//!
//! The window opens on a worker, so a slow driver or D-Bus activation never holds the UI thread.

use std::process::{Command, Stdio};

use wordcraft_ui_egui::PrintingPreferencesHook;

/// The hook WordCraft's File › Print page calls with the selected printer's queue name.
pub fn hook() -> Option<PrintingPreferencesHook> {
    Some(Box::new(|printer: &str| {
        if printer.is_empty() {
            return Err("no printer is selected".into());
        }
        let printer = printer.to_owned();
        std::thread::Builder::new()
            .name("printing-preferences".into())
            .spawn(move || {
                if let Err(error) = open(&printer) {
                    log::warn!("printing preferences for {printer}: {error}");
                }
            })
            .map(|_| ())
            .map_err(|error| format!("couldn't start: {error}"))
    }))
}

/// Runs `command` to the end, quietly; an error when it can't start or fails.
fn run(command: &mut Command) -> Result<(), String> {
    let status = command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().map_err(|error| error.to_string())?;
    if status.success() { Ok(()) } else { Err(format!("exited with {status}")) }
}

#[cfg(target_os = "windows")]
fn open(printer: &str) -> Result<(), String> {
    // `/e`: the driver's Printing Preferences (paper type, quality, colour), as Word's Printer
    // Properties opens; `/p` would be the queue's admin Properties (ports, sharing). `/n`: which printer.
    run(Command::new("rundll32.exe").args(["printui.dll,PrintUIEntry", "/e", "/n", printer]))
}

#[cfg(target_os = "macos")]
fn open(_printer: &str) -> Result<(), String> {
    run(Command::new("open").arg("x-apple.systempreferences:com.apple.Print-Scan-Settings.extension"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open(printer: &str) -> Result<(), String> {
    let dbus = run(Command::new("gdbus").args([
        "call",
        "--session",
        "--timeout",
        "60",
        "--dest",
        "org.fedoraproject.Config.Printing",
        "--object-path",
        "/org/fedoraproject/Config/Printing",
        "--method",
        "org.fedoraproject.Config.Printing.PrinterPropertiesDialog",
        "0",
        &gvariant_string(printer),
    ]));
    let Err(dbus) = dbus else { return Ok(()) };
    log::info!("system-config-printer's printer window is unavailable ({dbus}); opening the CUPS page");
    run(Command::new("xdg-open").arg(cups_page(printer)))
}

#[cfg(not(any(unix, target_os = "windows")))]
fn open(_printer: &str) -> Result<(), String> {
    Err("printing preferences aren't available on this system".into())
}

/// `text` as a GVariant string literal, the form `gdbus call` reads its arguments in.
#[cfg(any(all(unix, not(target_os = "macos")), test))]
fn gvariant_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('\'');
    for c in text.chars() {
        if matches!(c, '\\' | '\'') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('\'');
    out
}

/// The printer's page in the local CUPS web interface.
#[cfg(any(all(unix, not(target_os = "macos")), test))]
fn cups_page(printer: &str) -> String {
    let mut url = String::from("http://localhost:631/printers/");
    for byte in printer.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            url.push(char::from(byte));
        } else {
            url.push_str(&format!("%{byte:02X}"));
        }
    }
    url
}

#[cfg(test)]
mod tests {
    #[test]
    fn printer_names_are_quoted_for_gdbus() {
        assert_eq!(super::gvariant_string("HP_LaserJet"), "'HP_LaserJet'");
        assert_eq!(super::gvariant_string(r"it's\x"), r"'it\'s\\x'");
    }

    #[test]
    fn the_cups_page_escapes_the_queue_name() {
        assert_eq!(super::cups_page("Brother_HL-L2350"), "http://localhost:631/printers/Brother_HL-L2350");
        assert_eq!(super::cups_page("a b?c"), "http://localhost:631/printers/a%20b%3Fc");
    }
}
