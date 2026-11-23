//! The Print dialog (Ctrl+P): the printer (name, its Properties, status, type, port and comment),
//! Reverse output, Print to file and two-sided printing; the page range, odd or even pages and
//! the order pages fill a sheet; copies and collation; pages per sheet, scale to paper size and
//! lines between pages. Options… holds Print Markup, the orientation of a sheet with several
//! pages and the preview (File › Print).
//!
//! The settings are the File › Print page's own ([`crate::print_page::PrintSettings`] and the
//! shared printer state), so both show the same job; Print runs `ui.print`, Print to file
//! saves the same job as a PDF.

use egui::{Align, Layout, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use serde::Serialize;
use serde_json::json;

use crate::WordApp;
use crate::printing::{Duplex, Feedback};
use crate::theme::{regular, semibold};

/// The Print dialog's own state; the job's settings are the app's.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintForm {
    /// Print to file: Print saves the job as a PDF instead of sending it to the printer.
    pub to_file: bool,
}

impl PrintForm {
    pub fn read(_app: &WordApp) -> PrintForm {
        PrintForm::default()
    }
}

/// The dialog's width, its label column and the height of a row.
const WIDTH: f32 = 700.0;
const LABEL_W: f32 = 104.0;
const ROW_H: f32 = 24.0;
const PRINTER_W: f32 = 390.0;

/// The sheet layouts Pages per sheet offers.
const PAGES_PER_SHEET: [(&str, &str); 8] = [
    ("1", "1 page"),
    ("2", "2 pages"),
    ("4", "4 pages"),
    ("6", "6 pages"),
    ("9", "9 pages"),
    ("16", "16 pages"),
    ("booklet", "Booklet (left binding)"),
    ("bookletReverse", "Booklet (right binding)"),
];
const PARITIES: [(&str, &str); 3] = [("all", "All pages in range"), ("odd", "Odd pages"), ("even", "Even pages")];
const ORDERS: [(&str, &str); 3] = [("across", "Left to right"), ("down", "Top to bottom"), ("repeat", "Repeat")];
const ORIENTATIONS: [(&str, &str); 3] = [("auto", "Auto orientation"), ("portrait", "Portrait"), ("landscape", "Landscape")];
const PAGES_HELP: &str = "Type page numbers and page ranges separated by commas, counting from the start of the document. For example: 1, 3, 5-12.";

fn heading(ui: &mut Ui, s: &str) {
    ui.label(RichText::new(tl!(s)).font(semibold(13.0)));
}

/// A label in the left column of a row, vertically centred.
fn label(ui: &mut Ui, s: &str, width: f32) -> egui::Response {
    ui.allocate_ui_with_layout(vec2(width, ROW_H), Layout::left_to_right(Align::Center), |ui| {
        ui.set_min_width(width);
        ui.label(tl!(s))
    })
    .inner
}

/// A row: `label` and then `add`.
fn row(ui: &mut Ui, s: &str, width: f32, add: impl FnOnce(&mut Ui, &egui::Response)) {
    ui.horizontal(|ui| {
        ui.set_min_height(ROW_H);
        let l = label(ui, s, width);
        add(ui, &l);
    });
}

/// A choice list over `(id, label)` pairs; true when the choice changed.
fn choice(ui: &mut Ui, salt: &str, current: &mut String, items: &[(&str, &str)], width: f32, labelled: &egui::Response) -> bool {
    let shown = items.iter().find(|(id, _)| *id == current.as_str()).map_or("", |(_, l)| *l);
    let mut changed = false;
    let combo = craft_ui::ComboBox::from_id_salt(salt).width(width).selected_text(tl!(shown)).show_ui(ui, |ui| {
        for (id, item) in items {
            if ui.selectable_label(current == id, tl!(item)).clicked() && current != id {
                *current = (*id).to_string();
                changed = true;
            }
        }
    });
    combo.response.labelled_by(labelled.id);
    changed
}

/// The status line of the selected printer.
fn status(app: &WordApp) -> String {
    let p = &app.printing;
    if p.discovering() {
        tl!("Finding printers…").to_owned()
    } else if p.busy() {
        tl!("Submitting print job…").to_owned()
    } else if matches!(p.message, Some(Feedback::NoPrinters)) {
        tl!("No printers found").to_owned()
    } else if matches!(p.message, Some(Feedback::DiscoveryFailed)) {
        tl!("Printer discovery failed.").to_owned()
    } else if p.printers.iter().any(|x| x.name == p.settings.printer) {
        tl!("Ready").to_owned()
    } else {
        String::new()
    }
}

/// The details of the selected printer: asked of the host once per printer, kept per window.
fn details(app: &mut WordApp, ctx: &egui::Context) -> crate::PrinterDetails {
    let name = app.printing.settings.printer.clone();
    if let Some((asked, rx)) = &app.print_memo.details_pending {
        match rx.try_recv() {
            Ok(d) => {
                let asked = asked.clone();
                app.print_memo.details.insert(asked, d);
                app.print_memo.details_pending = None;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                let asked = asked.clone();
                app.print_memo.details.insert(asked, Default::default());
                app.print_memo.details_pending = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint_after(std::time::Duration::from_millis(100)),
        }
    }
    if let Some(d) = app.print_memo.details.get(&name) {
        return d.clone();
    }
    if !name.is_empty()
        && app.print_memo.details_pending.is_none()
        && let Some(hook) = &app.services.printer_details
    {
        app.print_memo.details_pending = Some((name, hook(&app.printing.settings.printer)));
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
    Default::default()
}

/// Two little stacks of pages: collated (1 2 3, 1 2 3) or not (1 1, 2 2).
fn collate_picture(ui: &mut Ui, collate: bool) {
    let (r, _) = ui.allocate_exact_size(vec2(96.0, 44.0), Sense::hover());
    let v = ui.visuals();
    let (paper, ink, edge) = (v.extreme_bg_color, v.text_color(), v.widgets.noninteractive.fg_stroke.color);
    let painter = ui.painter_at(r);
    let stacks: [[&str; 3]; 2] = if collate { [["1", "2", "3"], ["1", "2", "3"]] } else { [["1", "1", "1"], ["2", "2", "2"]] };
    for (s, stack) in stacks.iter().enumerate() {
        let x0 = r.min.x + 4.0 + s as f32 * 48.0;
        // Back to front: each sheet behind sits up and to the right, its number in the corner
        // that shows.
        for (i, n) in stack.iter().enumerate().rev() {
            let k = i as f32;
            let page = Rect::from_min_size(pos2(x0 + k * 7.0, r.min.y + 12.0 - k * 6.0), vec2(20.0, 30.0));
            painter.rect(page, 1.0, paper, Stroke::new(1.0, edge), egui::StrokeKind::Inside);
            painter.text(pos2(page.max.x - 5.0, page.min.y + 7.0), egui::Align2::CENTER_CENTER, *n, regular(9.0), ink);
        }
    }
}

/// How pages fill a sheet in `order`: a sheet of six cells, numbered.
fn order_picture(ui: &mut Ui, order: &str, enabled: bool) {
    let (r, _) = ui.allocate_exact_size(vec2(44.0, 58.0), Sense::hover());
    let v = ui.visuals();
    let ink = if enabled { v.text_color() } else { v.weak_text_color() };
    let edge = v.widgets.noninteractive.fg_stroke.color;
    let painter = ui.painter_at(r);
    let sheet = r.shrink(2.0);
    painter.rect(sheet, 1.0, v.extreme_bg_color, Stroke::new(1.0, edge), egui::StrokeKind::Inside);
    let (cols, rows) = (2, 3);
    let (cw, ch) = (sheet.width() / cols as f32, sheet.height() / rows as f32);
    for i in 0..cols * rows {
        let (col, row) = (i % cols, i / cols);
        let n = match order {
            "down" => col * rows + row + 1,
            "repeat" => 1,
            _ => i + 1,
        };
        let cell = Rect::from_min_size(pos2(sheet.min.x + col as f32 * cw, sheet.min.y + row as f32 * ch), vec2(cw, ch)).shrink(2.0);
        painter.rect_stroke(cell, 0.0, Stroke::new(0.5, edge), egui::StrokeKind::Inside);
        painter.text(cell.center(), egui::Align2::CENTER_CENTER, n.to_string(), regular(9.0), ink);
    }
}

/// A small round "?" that explains on hover.
fn help(ui: &mut Ui, text: &str) {
    let (r, resp) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
    let c = ui.visuals().weak_text_color();
    ui.painter().circle_stroke(r.center(), 7.5, Stroke::new(1.0, c));
    ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, "?", regular(11.0), c);
    resp.on_hover_text(tl!(text));
}

/// The printer box: name and Properties, status, type, where, comment, paper source, and the
/// checkboxes beside them.
fn printer_box(app: &mut WordApp, ui: &mut Ui, f: &mut PrintForm, native: bool) {
    heading(ui, "Printer");
    let info = details(app, ui.ctx());
    let busy = app.printing.busy();
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(LABEL_W + PRINTER_W + 8.0);
            row(ui, "Name:", LABEL_W, |ui, l| {
                ui.add_enabled_ui(native && !busy, |ui| {
                    let names = craft_ui::printing::shown_names(&app.printing.printers);
                    let chosen = app.printing.settings.printer.clone();
                    let shown = app.printing.printers.iter().zip(&names).find(|(p, _)| p.name == chosen).map(|(_, n)| n.clone());
                    let text =
                        shown.unwrap_or_else(|| if native { tl!("Select a printer").to_owned() } else { tl!("System print dialog").to_owned() });
                    craft_ui::ComboBox::from_id_salt("print_dialog_printer")
                        .width(PRINTER_W)
                        .wrap_mode(egui::TextWrapMode::Truncate)
                        .selected_text(text)
                        .show_ui(ui, |ui| {
                            for (printer, name) in app.printing.printers.iter().zip(&names) {
                                ui.selectable_value(&mut app.printing.settings.printer, printer.name.clone(), name);
                            }
                            ui.separator();
                            if ui.add_enabled(!app.printing.discovering(), egui::Button::new(tl!("Refresh printers"))).clicked()
                                && let Some(discover) = &app.services.printers
                            {
                                app.printing.refresh(discover);
                            }
                        })
                        .response
                        .labelled_by(l.id);
                });
            });
            let status = status(app);
            for (name, value) in
                [("Status:", status.as_str()), ("Type:", info.kind.as_str()), ("Where:", info.location.as_str()), ("Comment:", info.comment.as_str())]
            {
                row(ui, name, LABEL_W, |ui, _| {
                    ui.add(egui::Label::new(value).truncate());
                });
            }
            row(ui, "Paper source:", LABEL_W, |ui, l| {
                // The tray is the driver's setting: Properties… changes it.
                let mut source = "printer".to_string();
                choice(ui, "print_dialog_source", &mut source, &[("printer", "Use printer settings")], 220.0, l);
            });
        });
        ui.vertical(|ui| {
            ui.set_min_height(ROW_H);
            let chosen = app.printing.printers.iter().any(|p| p.name == app.printing.settings.printer);
            let properties = ui
                .add_enabled(
                    chosen && !busy && app.services.printing_preferences.is_some(),
                    egui::Button::new(tl!("Properties…")).min_size(vec2(150.0, ROW_H)),
                )
                .on_hover_text(tl!("Change the paper type, print quality and other settings of this printer"));
            if properties.clicked() {
                let _ = app.printing_preferences();
            }
            ui.add_space(4.0);
            ui.checkbox(&mut app.ui.print.reverse, tl!("Reverse output")).on_hover_text(tl!("Print the last page first"));
            ui.checkbox(&mut f.to_file, tl!("Print to file")).on_hover_text(tl!("Save the pages as a PDF file instead of printing them"));
            ui.add_enabled_ui(native && !busy, |ui| {
                let mut duplex = matches!(app.printing.settings.duplex, Some(Duplex::LongEdge | Duplex::ShortEdge));
                if ui.checkbox(&mut duplex, tl!("Two-sided printing")).on_hover_text(tl!("Print on both sides of the paper")).changed() {
                    app.printing.settings.duplex = duplex.then_some(Duplex::LongEdge);
                }
            });
        });
    });
}

/// Page range and Print (all, odd or even pages), then Order.
fn range_box(app: &mut WordApp, ui: &mut Ui) {
    heading(ui, "Page range");
    let s = &mut app.ui.print;
    ui.radio_value(&mut s.range, "all".to_string(), tl!("All"));
    ui.radio_value(&mut s.range, "current".to_string(), tl!("Current page"));
    ui.horizontal(|ui| {
        let pages = ui.radio_value(&mut s.range, "custom".to_string(), tl!("Pages:"));
        let edit = ui.add(
            egui::TextEdit::singleline(&mut s.pages)
                .desired_width(190.0)
                .hint_text(tl!("1-3, 5"))
                .char_limit(wordcraft_engine::print::MAX_SPEC_CHARS),
        );
        let edit = edit.labelled_by(pages.id);
        if edit.gained_focus() || edit.changed() {
            s.range = "custom".into();
        }
        help(ui, PAGES_HELP);
    });
    let has_selection = !app.session.sel.is_collapsed();
    let s = &mut app.ui.print;
    if !has_selection && s.range == "selection" {
        s.range = "all".into();
    }
    ui.add_enabled_ui(has_selection, |ui| ui.radio_value(&mut s.range, "selection".to_string(), tl!("Selection")));
    ui.add_space(6.0);
    row(ui, "Print:", 60.0, |ui, l| {
        choice(ui, "print_dialog_parity", &mut s.parity, &PARITIES, 210.0, l);
    });
    ui.add_space(6.0);
    let grid = s.pages_per_sheet != "1" && !s.pages_per_sheet.starts_with("booklet");
    ui.add_enabled_ui(grid, |ui| {
        heading(ui, "Order");
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for (id, name) in ORDERS {
                    ui.radio_value(&mut s.order, id.to_string(), tl!(name));
                }
            });
            ui.add_space(16.0);
            order_picture(ui, &s.order, grid);
        });
    });
}

/// Copies and Collate, then Zoom: pages per sheet, scale to paper size, lines between pages.
fn copies_box(app: &mut WordApp, ui: &mut Ui, native: bool) {
    heading(ui, "Copies");
    let busy = app.printing.busy();
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            row(ui, "Number of copies:", 120.0, |ui, l| {
                let mut n = if native { app.printing.settings.copies } else { app.ui.print.copies };
                ui.add_enabled(!busy, egui::DragValue::new(&mut n).range(1..=wordcraft_engine::print::MAX_COPIES).speed(0.2)).labelled_by(l.id);
                let n = n.clamp(1, wordcraft_engine::print::MAX_COPIES);
                app.ui.print.copies = n;
                if native {
                    app.printing.settings.copies = n;
                }
            });
            ui.add_enabled_ui(!busy, |ui| ui.checkbox(&mut app.ui.print.collate, tl!("Collate")));
        });
        ui.add_space(6.0);
        collate_picture(ui, app.ui.print.collate);
    });
    ui.add_space(8.0);
    heading(ui, "Zoom");
    let s = &mut app.ui.print;
    row(ui, "Pages per sheet:", 150.0, |ui, l| {
        choice(ui, "print_dialog_pps", &mut s.pages_per_sheet, &PAGES_PER_SHEET, 160.0, l);
    });
    row(ui, "Scale to paper size:", 150.0, |ui, l| {
        let papers: Vec<(&str, &str)> =
            std::iter::once(("none", "No Scaling")).chain(wordcraft_geom::PAPER_SIZES.iter().map(|(n, _, _)| (*n, *n))).collect();
        choice(ui, "print_dialog_paper", &mut s.paper, &papers, 160.0, l);
    });
    let several = s.pages_per_sheet != "1";
    ui.add_enabled_ui(several, |ui| ui.checkbox(&mut s.lines, tl!("Draw lines between pages")));
    if s.pages_per_sheet.starts_with("booklet") {
        ui.label(RichText::new(tl!("Booklets: duplex on the short edge. Blank pages are added at the end.")).small().weak());
    }
}

/// What Options… changed: the preview asked for.
fn options_menu(app: &mut WordApp, ui: &mut Ui) -> bool {
    let mut preview = false;
    ui.menu_button(tl!("Options…"), |ui| {
        let mut markup = app.session.prefs.print_markup;
        if ui.checkbox(&mut markup, tl!("Print Markup")).changed() {
            let _ = app.run("file.printMarkup", json!({"value": markup}));
        }
        ui.separator();
        ui.label(RichText::new(tl!("Orientation")).weak());
        for (id, name) in ORIENTATIONS {
            ui.radio_value(&mut app.ui.print.orientation, id.to_string(), tl!(name));
        }
        ui.separator();
        if ui.button(tl!("Print Preview")).clicked() {
            preview = true;
            ui.close();
        }
    });
    preview
}

/// Draws the Print dialog; true closes it.
pub(super) fn print_ui(app: &mut WordApp, ui: &mut Ui, f: &mut PrintForm) -> bool {
    let native = app.services.print_native.is_some();
    let hook = native || app.services.print.is_some();
    // Printers are found once per run, as the File › Print page does; Refresh finds them again.
    if native && let Some(discover) = &app.services.printers {
        if app.printing.printers.is_empty() && !app.printing.discovering() && !app.printing.busy() && app.printing.message.is_none() {
            app.printing.refresh(discover);
        }
        app.printing.poll(ui.ctx());
    }
    ui.set_width(WIDTH);
    ui.spacing_mut().item_spacing.y = 4.0;
    printer_box(app, ui, f, native);
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);
    ui.columns(2, |cols| {
        if let [left, right] = cols {
            range_box(app, left);
            copies_box(app, right, native);
        }
    });

    // What will print, or why it can't.
    let lay = app.session.export_layout();
    let total = lay.pages.len();
    let plan = wordcraft_engine::print::plan(&app.ui.print.params(), total, app.session.current_pages(&lay));
    let message = match &plan {
        Ok(p) => Ok(crate::i18n::fmt(tl!("{count} of {total} pages will print"), &[("count", &p.chosen.to_string()), ("total", &total.to_string())])),
        Err(e) => Err(e.to_string()),
    };
    ui.add_space(6.0);
    match &message {
        Ok(m) => ui.label(RichText::new(m).weak()),
        Err(e) => ui.label(RichText::new(e).color(ui.visuals().error_fg_color)),
    };
    if let Some(Feedback::Failed(e)) = &app.printing.message {
        ui.label(RichText::new(crate::i18n::fmt(tl!("Print failed: {error}"), &[("error", e)])).color(ui.visuals().error_fg_color));
    }
    ui.add_space(8.0);

    let mut close = false;
    let mut print = false;
    let mut preview = false;
    ui.horizontal(|ui| {
        preview = options_menu(app, ui);
        ui.with_layout(crate::rtl::far_side(ui, Align::Center), |ui| {
            let can = plan.is_ok() && !app.printing.busy() && (f.to_file || (hook && (!native || app.printing.ready())));
            let (okb, closeb) = crate::dialogs::ok_cancel_responses(
                ui,
                |ui| ui.add_enabled(can, crate::widgets::primary_button(ui, tl!("Print")).min_size(vec2(80.0, 28.0))),
                tl!("Close"),
            );
            close |= closeb.clicked() || crate::dialogs::cancel_key_pressed(ui);
            print |= okb.clicked() || (can && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !ui.ctx().text_edit_focused());
        });
    });
    if preview {
        app.ui.backstage = true;
        app.ui.backstage_page = "print".into();
        return true;
    }
    if print {
        if f.to_file {
            app.print_pdf_dialog();
            return true;
        }
        let params = app.ui.print.params();
        return match app.run("ui.print", params) {
            Ok(_) => true,
            Err(e) => {
                app.status(e);
                false
            }
        };
    }
    close
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Services;
    use egui_kittest::kittest::{By, Queryable};
    use wordcraft_engine::Session;

    /// Ctrl+P opens the Print dialog over the document (File › Print stays a page when the
    /// option is off); its settings are the File › Print page's, Reverse output, Order and the
    /// lines between pages included, and Print sends that job.
    #[test]
    fn ctrl_p_opens_the_print_dialog_and_print_sends_its_settings() {
        let sent = std::rc::Rc::new(std::cell::RefCell::new(Vec::<usize>::new()));
        let seen = sent.clone();
        let services = Services {
            print: Some(Box::new(move |bytes: &[u8]| {
                seen.borrow_mut().push(bytes.len());
                Ok(())
            })),
            ..Default::default()
        };
        let mut a = WordApp::new(Session::new(wordcraft_doc::Document::from_text("one\u{c}two\u{c}three")), services);
        a.run("file.print", json!({})).unwrap();
        assert!(!a.ui.backstage, "a dialog, not the File › Print page");
        assert!(matches!(&a.dialog, Some(crate::dialogs::Dialog::Layout(d)) if d.name() == "print"));
        a.ui.print.reverse = true;
        a.ui.print.pages_per_sheet = "2".into();
        a.ui.print.order = "repeat".into();
        a.ui.print.lines = true;
        let p = a.ui.print.params();
        assert_eq!((p["reverse"].clone(), p["order"].clone(), p["lines"].clone()), (json!(true), json!("repeat"), json!(true)));
        let mut h = egui_kittest::Harness::builder().with_accessibility_check(false).with_size(egui::vec2(1440.0, 900.0)).build_ui_state(
            |ui, app: &mut WordApp| {
                app.logic(&ui.ctx().clone());
                app.ui(ui);
            },
            a,
        );
        h.run_steps(3);
        // The dialog's Print button: the last button of that name (drawn above the window).
        let buttons: Vec<_> = h.query_all(By::new().label("Print").role(egui::accesskit::Role::Button)).collect();
        buttons.last().expect("a Print button").click();
        h.run_steps(3);
        assert_eq!(sent.borrow().len(), 1, "Print sent the job");
        assert!(h.state().dialog.is_none(), "and closed the dialog");

        let mut b = WordApp::new(Session::new(wordcraft_doc::Document::new()), Services::default());
        b.ui.print_dialog = false;
        b.run("file.print", json!({})).unwrap();
        assert!(b.ui.backstage && b.ui.backstage_page == "print" && b.dialog.is_none());
    }
}
