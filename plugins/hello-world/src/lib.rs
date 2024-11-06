//! Hexerator hello world example plugin

use hexerator_plugin_api::{
    HexeratorHandle, MethodParam, MethodResult, Plugin, PluginMethod, PluginSourceProviderParams,
    Value, ValueTy,
};

struct HelloPlugin {
    i: usize,
}

impl Plugin for HelloPlugin {
    fn name(&self) -> &'static str {
        "Hello world plugin"
    }

    fn desc(&self) -> &'static str {
        "Hi! I'm an example plugin for Hexerator"
    }

    fn methods(&self) -> Vec<hexerator_plugin_api::PluginMethod> {
        vec![
            PluginMethod {
                method_name: "say_hello",
                human_name: Some("Say hello"),
                desc: "Write 'hello' to debug log.",
                params: &[],
            },
            PluginMethod {
                method_name: "fill_selection",
                human_name: Some("Fill selection"),
                desc: "Fills the selection with 0x42",
                params: &[],
            },
            PluginMethod {
                method_name: "sum_range",
                human_name: None,
                desc: "Sums up the values in the provided range",
                params: &[
                    MethodParam {
                        name: "from",
                        ty: ValueTy::U64,
                    },
                    MethodParam {
                        name: "to",
                        ty: ValueTy::U64,
                    },
                ],
            },
        ]
    }

    fn on_method_called(
        &mut self,
        name: &str,
        params: &[Option<Value>],
        hexerator: &mut dyn HexeratorHandle,
    ) -> MethodResult {
        match name {
            "say_hello" => {
                hexerator.debug_log("Hello world!");
                Ok(None)
            }
            "fill_selection" => match hexerator.selection_range() {
                Some((start, end)) => match hexerator.get_data_mut(start, end) {
                    Some(data) => {
                        data.fill(0x42);
                        Ok(None)
                    }
                    None => Err("Selection out of bounds".into()),
                },
                None => Err("Selection unavailable".into()),
            },
            "sum_range" => {
                let &[Some(Value::U64(from)), Some(Value::U64(to))] = params else {
                    return Err("Invalid params".into());
                };
                match hexerator.get_data_mut(from as usize, to as usize) {
                    Some(data) => {
                        let sum: u64 = data.iter().map(|b| *b as u64).sum();
                        Ok(Some(Value::U64(sum)))
                    }
                    None => Err("Out of bounds".into()),
                }
            }
            _ => Err(format!("Unknown method: {name}")),
        }
    }

    fn source_provider_params(&self) -> Option<PluginSourceProviderParams> {
        Some(PluginSourceProviderParams {
            human_name: "hello_world",
            // is_stream: true,
            is_stream: false,
            is_writable: false,
            is_savable: false,
            auto_reload_type: hexerator_plugin_api::AutoReloadType::OneShot,
        })
    }
    fn sp_read_contents(&mut self) -> std::io::Result<Vec<u8>> {
        let len = 100_000u32;
        let o = (0..len).map(|v| ((v + self.i as u32) % 100) as u8).collect();
        self.i += 1;
        Ok(o)
    }
    fn sp_read_exact(&mut self, lo: usize, buf: &mut [u8]) -> std::io::Result<()> {
        (lo..buf.len()).enumerate().for_each(|(i, v)| {
            buf[lo + i] = ((v + i + self.i) % 100) as u8;
        });
        self.i += 1;
        Ok(())
    }
    fn sp_read_stream(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        buf[0] = self.i as u8;
        if self.i == 100 {
            // Finished stream
            return Ok(0);
        }
        self.i += 1;
        Ok(1)
    }
}

#[no_mangle]
pub extern "Rust" fn hexerator_plugin_new() -> Box<dyn Plugin> {
    Box::new(HelloPlugin { i: 0 })
}
