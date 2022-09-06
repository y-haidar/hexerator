use egui_sfml::egui::{self, Ui};
use gamedebug_core::{Info, PerEntry, IMMEDIATE, PERSISTENT};

pub fn ui(ui: &mut Ui) {
    match IMMEDIATE.lock() {
        Ok(imm) => {
            egui::ScrollArea::vertical()
                .max_height(500.)
                .show(ui, |ui| {
                    for info in imm.iter() {
                        if let Info::Msg(msg) = info {
                            ui.label(msg);
                        }
                    }
                });
        }
        Err(e) => {
            ui.label(&format!("IMMEDIATE lock fail: {}", e));
        }
    }
    gamedebug_core::clear_immediates();
    ui.separator();
    match PERSISTENT.lock() {
        Ok(per) => {
            for PerEntry { frame, info } in per.iter() {
                if let Info::Msg(msg) = info {
                    ui.label(format!("{}: {}", frame, msg));
                }
            }
        }
        Err(e) => {
            ui.label(&format!("PERSISTENT lock fail: {}", e));
        }
    }
}