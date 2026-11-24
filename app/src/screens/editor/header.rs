//! The editor's section settings (name, tempo, time signature, bars) and
//! its list of staves.

use eframe::egui;
use model::{Segment, StaffKind, TimeSig};

use super::commands;
use super::cursor::Cursor;
use crate::screens::widgets::{
    Icon, NAME_FIELD_W, SECTION_NAME_HINT, SHORT_FIELD_W, TEMPO_LABEL, combo, danger, icon_button,
    menu, optional_bpm, toggle,
};
use crate::shortcuts;
use crate::status::{self, Kind};

pub fn section_settings(ui: &mut egui::Ui, seg: &mut Segment) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Name");
        let name = egui::TextEdit::singleline(&mut seg.name).hint_text(SECTION_NAME_HINT);
        ui.add(name.desired_width(NAME_FIELD_W));
        ui.separator();
        optional_bpm(ui, TEMPO_LABEL, &mut seg.tempo);
        ui.separator();
        // Combo boxes, not drag values: every change is applied at once and
        // cuts bars, so intermediate values while dragging would lose notes.
        ui.label("Time");
        let mut ts = seg.time_sig;
        let numbers = |values: &mut dyn Iterator<Item = u8>| -> Vec<(u8, String)> {
            values.map(|n| (n, n.to_string())).collect()
        };
        let numerators = numbers(&mut (1..=TimeSig::MAX_NUM));
        let denominators = numbers(&mut TimeSig::DENOMINATORS.iter().copied());
        combo(ui, "time_num", &mut ts.num, &numerators);
        ui.label("/");
        combo(ui, "time_den", &mut ts.den, &denominators);
        if ts != seg.time_sig {
            let shorter = ts.bar_ticks() < seg.time_sig.bar_ticks();
            commands::set_time_sig(seg, ts);
            if shorter {
                let undo = ui.ctx().format_shortcut(&shortcuts::UNDO);
                status::post(
                    ui.ctx(),
                    Kind::Warning,
                    format!("Bars were cut to fit; {undo} undoes this"),
                );
            }
        }
        ui.separator();
        ui.label(format!("Bars {}", seg.bar_count()));
        if icon_button(ui, Icon::Remove, "Remove bar")
            .on_hover_text("Remove the last bar")
            .clicked()
        {
            commands::remove_last_bar(seg);
        }
        if icon_button(ui, Icon::Add, "Add bar").on_hover_text("Add a bar at the end").clicked() {
            commands::add_bar(seg);
        }
    });
}

/// The staves as toggles (the one being edited is lit), with label
/// fields, remove buttons and an "Add staff" menu.
pub fn staves(ui: &mut egui::Ui, seg: &mut Segment, cursor: &mut Cursor) {
    let mut remove = None;
    ui.horizontal_wrapped(|ui| {
        for (i, staff) in seg.staves.iter_mut().enumerate() {
            if toggle(ui, cursor.staff == i, staff.kind.name())
                .on_hover_text("Edit this staff")
                .clicked()
            {
                cursor.staff = i;
                cursor.voice = 0;
            }
            let label =
                egui::TextEdit::singleline(&mut staff.label).hint_text(staff.kind.short_name());
            ui.add(label.desired_width(SHORT_FIELD_W))
                .on_hover_text("Staff label printed on the sheet");
            if danger(ui, Icon::Delete, "Remove staff").clicked() {
                remove = Some(i);
            }
            ui.separator();
        }
        menu(ui, Icon::Add, "Add staff", |ui| {
            for kind in StaffKind::ALL {
                if ui.button(kind.name()).clicked() {
                    commands::add_staff(seg, kind);
                    ui.close();
                }
            }
        });
    });
    if let Some(i) = remove {
        commands::remove_staff(seg, cursor, i);
    }
}
