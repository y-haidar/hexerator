pub trait Plugin {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn methods(&self) -> Vec<PluginMethod>;
    fn on_method_called(
        &mut self,
        name: &str,
        params: &[Option<Value>],
        hexerator: &mut dyn HexeratorHandle,
    ) -> MethodResult;
    // if `Some` then a sub menu `Open with...` with a sub item `PLUGIN_HUMAN_NAME` is added to `File` menu.
    fn source_provider_params(&self) -> Option<PluginSourceProviderParams>;
    // Look at `PluginSourceProvider` docs/source-code
    // fn sp_write(&mut self, lo: usize, buf: &mut [u8]) -> std::io::Result<()>;
    // fn sp_save(&mut self) -> std::io::Result<()>;
    fn sp_read_contents(&mut self) -> std::io::Result<Vec<u8>>;
    fn sp_read_exact(&mut self, lo: usize, buf: &mut [u8]) -> std::io::Result<()>;
    fn sp_read_stream(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

pub type MethodResult = Result<Option<Value>, String>;

pub struct PluginMethod {
    pub method_name: &'static str,
    pub human_name: Option<&'static str>,
    pub desc: &'static str,
    pub params: &'static [MethodParam],
}

pub struct MethodParam {
    pub name: &'static str,
    pub ty: ValueTy,
}

pub enum ValueTy {
    U64,
    String,
}

pub enum Value {
    U64(u64),
    F64(f64),
    String(String),
}

impl ValueTy {
    pub fn label(&self) -> &'static str {
        match self {
            ValueTy::U64 => "u64",
            ValueTy::String => "string",
        }
    }
}

pub struct PluginSourceProviderParams {
    pub human_name: &'static str,
    pub is_stream: bool,
    pub is_writable: bool,
    pub is_savable: bool,
    pub auto_reload_type: AutoReloadType,
}

pub enum AutoReloadType {
    Both,
    // It is not always possible to read everything, as for example, the plugin could be using a slow
    // network protocol like TCP. Another example, a remote gdb session.
    RegionOnly,
    // This can be the type of a plugin that acts as a network proxy and dumps its buffer
    // that was filled from an internal continuous stream
    OneShot, // or All
}

pub trait PluginSourceProvider {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    // to be honest, I didn't look at the source code for writing.
    // if the plugin doesn't support write, it should panic, as `can_write` should prevent
    // the call of this function
    fn write(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

pub trait HexeratorHandle {
    fn selection_range(&self) -> Option<(usize, usize)>;
    fn get_data(&self, start: usize, end: usize) -> Option<&[u8]>;
    fn get_data_mut(&mut self, start: usize, end: usize) -> Option<&mut [u8]>;
    fn debug_log(&self, msg: &str);
}
