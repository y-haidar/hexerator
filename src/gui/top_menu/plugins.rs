use crate::{
    app::App,
    gui::{message_dialog::Icon, Gui},
    plugin::PluginContainer,
    shell::msg_fail,
};

pub fn ui(ui: &mut egui::Ui, gui: &mut Gui, app: &mut App) {
    let mut plugins = std::mem::take(&mut app.plugins);
    let mut reload = None;
    if plugins.is_empty() {
        ui.add_enabled(false, egui::Label::new("No plugins loaded"));
    }
    plugins.retain_mut(|plugin| {
        let mut retain = true;
        ui.horizontal(|ui| {
            ui.label(plugin.name).on_hover_text(plugin.desc);
            if ui.button("🗑").on_hover_text("Unload").clicked() {
                retain = false;
                ui.close_menu();
            }
            if ui.button("↺").on_hover_text("Reload").clicked() {
                retain = false;
                reload = Some(plugin.path.clone());
                ui.close_menu();
            }
        });
        for method in &plugin.methods {
            let name = if let Some(name) = method.human_name {
                name
            } else {
                method.method_name
            };
            let hover_ui = |ui: &mut egui::Ui| {
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing.x = 0.;
                    ui.label(
                        egui::RichText::new(method.method_name)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.label(egui::RichText::new("(").strong().color(egui::Color32::WHITE));
                    for param in method.params {
                        ui.label(format!("{}: {},", param.name, param.ty.label()));
                    }
                    ui.label(egui::RichText::new(")").strong().color(egui::Color32::WHITE));
                });
                ui.indent("indent", |ui| {
                    ui.label(method.desc);
                });
            };
            if ui.button(name).on_hover_ui(hover_ui).clicked() {
                ui.close_menu();
                match plugin.plugin.write().unwrap().on_method_called(method.method_name, &[], app)
                {
                    Ok(val) => {
                        if let Some(val) = val {
                            let strval = match val {
                                hexerator_plugin_api::Value::U64(n) => n.to_string(),
                                hexerator_plugin_api::Value::String(s) => s.to_string(),
                                hexerator_plugin_api::Value::F64(n) => n.to_string(),
                            };
                            gui.msg_dialog.open(
                                Icon::Info,
                                "Method call result",
                                format!("{}: {}", method.method_name, strval),
                            );
                        }
                    }
                    Err(e) => {
                        msg_fail(&e, "Method call failed", &mut gui.msg_dialog);
                    }
                }
            }
        }
        retain
    });
    if let Some(path) = reload {
        unsafe {
            match PluginContainer::new(path) {
                Ok(plugin) => {
                    plugins.push(plugin);
                }
                Err(e) => msg_fail(&e, "Failed to reload plugin", &mut gui.msg_dialog),
            }
        }
    }
    std::mem::swap(&mut app.plugins, &mut plugins);
}
