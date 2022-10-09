use {
    crate::{
        app::App,
        damage_region::DamageRegion,
        gui::{
            message_dialog::{Icon, MessageDialog},
            Dialog,
        },
        slice_ext::SliceExt,
    },
    egui,
    rlua::Lua,
};

#[derive(Debug, Default)]
pub struct PatternFillDialog {
    pattern_string: String,
    just_opened: bool,
}

impl Dialog for PatternFillDialog {
    fn title(&self) -> &str {
        "Selection pattern fill"
    }

    fn on_open(&mut self) {
        self.just_opened = true;
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        app: &mut App,
        msg: &mut MessageDialog,
        _lua: &Lua,
    ) -> bool {
        let Some(sel) = app.hex_ui.selection() else {
            ui.heading("No active selection");
            return true;
        };
        let re = ui.text_edit_singleline(&mut self.pattern_string);
        if self.just_opened {
            re.request_focus();
        }
        self.just_opened = false;
        if ui.input().key_pressed(egui::Key::Enter) {
            let values: Result<Vec<u8>, _> = self
                .pattern_string
                .split(' ')
                .map(|token| u8::from_str_radix(token, 16))
                .collect();
            match values {
                Ok(values) => {
                    let range = sel.begin..=sel.end;
                    app.data[range.clone()].pattern_fill(&values);
                    app.edit_state
                        .widen_dirty_region(DamageRegion::RangeInclusive(range));
                    false
                }
                Err(e) => {
                    msg.open(Icon::Error, "Fill parse error", e.to_string());
                    true
                }
            }
        } else {
            true
        }
    }
}
