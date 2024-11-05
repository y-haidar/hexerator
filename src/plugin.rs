use {
    crate::app::App,
    hexerator_plugin_api::{HexeratorHandle, Plugin, PluginMethod, PluginSourceProviderParams},
    std::{
        self,
        path::PathBuf,
        sync::{Arc, RwLock},
    },
};

pub struct PluginContainer {
    pub path: PathBuf,
    // pub plugin: Box<dyn Plugin>,
    pub plugin: Arc<RwLock<Box<dyn Plugin>>>,
    pub name: &'static str,
    pub desc: &'static str,
    pub methods: Vec<PluginMethod>,
    pub sp_params: Option<PluginSourceProviderParams>,
    // Safety: Must be last, fields are dropped in decl order.
    pub _lib: libloading::Library,
}

impl std::fmt::Debug for PluginContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginContainer")
            .field("path", &self.path)
            .field("plugin", &"REDACTED")
            .field("methods", &"REDACTED")
            .field("_lib", &"REDACTED")
            .finish()
    }
}

impl HexeratorHandle for App {
    fn debug_log(&self, msg: &str) {
        gamedebug_core::per!("{msg}");
    }

    fn get_data(&self, start: usize, end: usize) -> Option<&[u8]> {
        self.data.get(start..=end)
    }

    fn get_data_mut(&mut self, start: usize, end: usize) -> Option<&mut [u8]> {
        self.data.get_mut(start..=end)
    }

    fn selection_range(&self) -> Option<(usize, usize)> {
        self.hex_ui.selection().map(|sel| (sel.begin, sel.end))
    }
}

impl PluginContainer {
    pub unsafe fn new(path: PathBuf) -> anyhow::Result<Self> {
        unsafe {
            let lib = libloading::Library::new(&path)?;
            let plugin_init = lib.get::<fn() -> Box<dyn Plugin>>(b"hexerator_plugin_new")?;
            let plugin = plugin_init();
            let name = plugin.name();
            let desc = plugin.desc();
            let methods = plugin.methods();
            let sp_params = plugin.source_provider_params();
            let plugin = Arc::new(RwLock::new(plugin_init()));
            Ok(Self {
                path,
                plugin,
                name,
                desc,
                sp_params,
                methods,
                _lib: lib,
            })
        }
    }
}
