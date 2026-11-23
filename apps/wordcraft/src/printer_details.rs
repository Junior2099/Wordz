//! What the Print dialog shows about a printer: its type (driver or model), where it is (its port
//! or device address) and its comment, looked up on a worker so the dialog never waits.
//!
//! - Windows: the printer's key in the spooler's registry (`reg query`): Printer Driver, Port,
//!   Description. Shared printers from another computer have no key there and show nothing.
//! - Linux and the BSDs: CUPS (`lpoptions -p`): printer-make-and-model, device-uri (without any
//!   user name or password in it), printer-info.
//! - Elsewhere nothing is shown.

use std::sync::mpsc::Receiver;

use wordcraft_ui_egui::{PrinterDetails, PrinterDetailsHook};

/// The hook the Print dialog calls with a printer's queue name.
pub fn hook() -> Option<PrinterDetailsHook> {
    Some(Box::new(|printer: &str| -> Receiver<PrinterDetails> {
        let (tx, rx) = std::sync::mpsc::channel();
        let printer = printer.to_owned();
        // A thread that can't start drops `tx`: the dialog shows nothing.
        let _ = std::thread::Builder::new().name("printer-details".into()).spawn(move || {
            let _ = tx.send(lookup(&printer));
        });
        rx
    }))
}

/// Most bytes of a lookup's output read.
const MAX_OUTPUT: usize = 64 << 10;
/// Longest text shown for one detail.
const MAX_CHARS: usize = 200;

fn clip(s: &str) -> String {
    s.trim().chars().take(MAX_CHARS).collect()
}

#[cfg(target_os = "windows")]
fn lookup(printer: &str) -> PrinterDetails {
    use std::os::windows::process::CommandExt;
    // A connection to another computer's printer (\\server\name) isn't under this key.
    if printer.is_empty() || printer.starts_with('\\') || printer.contains(['\0', '\n']) {
        return PrinterDetails::default();
    }
    let key = format!(r"HKLM\SYSTEM\CurrentControlSet\Control\Print\Printers\{printer}");
    // CREATE_NO_WINDOW: no console flashes up.
    let out = std::process::Command::new("reg").args(["query", &key]).creation_flags(0x0800_0000).output();
    match out {
        Ok(o) if o.status.success() => parse_reg(&String::from_utf8_lossy(&o.stdout[..o.stdout.len().min(MAX_OUTPUT)])),
        _ => PrinterDetails::default(),
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn lookup(printer: &str) -> PrinterDetails {
    if printer.is_empty() || printer.starts_with('-') {
        return PrinterDetails::default();
    }
    match std::process::Command::new("lpoptions").args(["-p", printer]).stdin(std::process::Stdio::null()).output() {
        Ok(o) if o.status.success() => parse_lpoptions(&String::from_utf8_lossy(&o.stdout[..o.stdout.len().min(MAX_OUTPUT)]), printer),
        _ => PrinterDetails::default(),
    }
}

#[cfg(not(any(target_os = "windows", all(unix, not(target_os = "macos")))))]
fn lookup(_printer: &str) -> PrinterDetails {
    PrinterDetails::default()
}

/// The details in `reg query`'s listing of a printer's key: `    Port    REG_SZ    USB001`.
#[cfg(any(target_os = "windows", test))]
fn parse_reg(text: &str) -> PrinterDetails {
    let mut d = PrinterDetails::default();
    for line in text.lines() {
        let Some((name, value)) = line.split_once("REG_SZ") else { continue };
        match name.trim() {
            "Printer Driver" => d.kind = clip(value),
            "Port" => d.location = clip(value),
            "Description" => d.comment = clip(value),
            _ => {}
        }
    }
    d
}

/// The details in `lpoptions -p`'s `key=value` list, values quoted as a shell would.
#[cfg(any(all(unix, not(target_os = "macos")), test))]
fn parse_lpoptions(text: &str, printer: &str) -> PrinterDetails {
    let mut d = PrinterDetails::default();
    for word in shell_words(text) {
        let Some((key, value)) = word.split_once('=') else { continue };
        match key {
            "printer-make-and-model" => d.kind = clip(value),
            "device-uri" => d.location = clip(&without_password(value)),
            "printer-info" if value != printer => d.comment = clip(value),
            "printer-location" if d.comment.is_empty() => d.comment = clip(value),
            _ => {}
        }
    }
    d
}

/// `text` split into words as a POSIX shell splits it: blanks separate them; quotes and
/// backslashes keep blanks inside one.
#[cfg(any(all(unix, not(target_os = "macos")), test))]
fn shell_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                started = true;
                for c in chars.by_ref() {
                    if c == '\'' {
                        break;
                    }
                    word.push(c);
                }
            }
            '"' => {
                started = true;
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => word.extend(chars.next()),
                        c => word.push(c),
                    }
                }
            }
            '\\' => {
                started = true;
                word.extend(chars.next());
            }
            c if c.is_whitespace() => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            c => {
                started = true;
                word.push(c);
            }
        }
    }
    if started {
        words.push(word);
    }
    words
}

/// A device address without the user name and password some carry (`ipp://user:pw@host/…`).
#[cfg(any(all(unix, not(target_os = "macos")), test))]
fn without_password(uri: &str) -> String {
    let Some((scheme, rest)) = uri.split_once("://") else { return uri.to_owned() };
    let host_end = rest.find('/').unwrap_or(rest.len());
    match rest.get(..host_end).and_then(|h| h.rfind('@')) {
        Some(at) => format!("{scheme}://{}", rest.get(at + 1..).unwrap_or_default()),
        None => uri.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_listing_gives_driver_port_and_comment() {
        let text = "\r\nHKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Print\\Printers\\Microsoft Print to PDF\r\n    \
            Name    REG_SZ    Microsoft Print to PDF\r\n    Port    REG_SZ    PORTPROMPT:\r\n    Printer Driver    REG_SZ    Microsoft Print To PDF\r\n    \
            Description    REG_SZ    \r\n    Attributes    REG_DWORD    0xa40\r\n\r\nHKEY_LOCAL_MACHINE\\...\\DsSpooler\r\n";
        let d = parse_reg(text);
        assert_eq!((d.kind.as_str(), d.location.as_str(), d.comment.as_str()), ("Microsoft Print To PDF", "PORTPROMPT:", ""));
    }

    #[test]
    fn cups_options_give_model_address_and_comment() {
        let text = "copies=1 device-uri=ipp://jo:secret@192.0.2.7/ipp/print printer-info='Office laser' printer-location=Hall \
            printer-make-and-model='Brother HL-L2350DW series' printer-is-shared=false";
        let d = parse_lpoptions(text, "Brother_HL");
        assert_eq!(d.kind, "Brother HL-L2350DW series");
        assert_eq!(d.location, "ipp://192.0.2.7/ipp/print", "no password shown");
        assert_eq!(d.comment, "Office laser");
        assert_eq!(shell_words(r#"a='b c' d="e \"f\"" g\ h"#), vec!["a=b c", "d=e \"f\"", "g h"]);
        assert_eq!(parse_lpoptions("printer-info=Office printer-location=Hall", "Office").comment, "Hall");
    }
}
