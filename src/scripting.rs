use {
    crate::{
        app::App,
        gui::{dialogs::pattern_fill::parse_pattern_string, Gui},
        meta::{
            region::Region,
            value_type::{self, EndianedPrimitive as _, ValueType},
            Bookmark, NamedRegion,
        },
        slice_ext::SliceExt as _,
    },
    anyhow::Context,
    egui_sfml::sfml::graphics::Font,
    mlua::{ExternalError as _, ExternalResult as _, IntoLuaMulti, Lua, UserData},
};

pub struct LuaExecContext<'app, 'gui, 'font> {
    pub app: &'app mut App,
    pub gui: &'gui mut Gui,
    pub font: &'font Font,
}

pub(crate) trait Method {
    /// Name of the method
    const NAME: &'static str;
    /// Help text for the method
    const HELP: &'static str;
    /// Stringified API signature for help purposes
    const API_SIG: &'static str;
    /// Arguments the method takes when called
    type Args;
    /// Return type
    type Ret: IntoLuaMulti<'static>;
    /// The function that gets called
    fn call(lua: &Lua, exec: &mut LuaExecContext, args: Self::Args) -> mlua::Result<Self::Ret>;
}

macro_rules! def_method {
    ($help:literal $name:ident($exec:ident, $($argname:ident: $argty:ty),*) -> $ret:ty $block:block) => {
        #[allow(non_camel_case_types)] pub(crate) enum $name {}
        impl Method for $name {
            const NAME: &'static str = stringify!($name);
            const HELP: &'static str = $help;
            const API_SIG: &'static str = concat!(stringify!($name), "(", $(stringify!($argname), ": ", stringify!($argty), ", ",)* ")", " -> ", stringify!($ret));
            type Args = ($($argty,)*);
            type Ret = $ret;
            fn call(_lua: &Lua, $exec: &mut LuaExecContext, ($($argname,)*): ($($argty,)*)) -> mlua::Result<$ret> $block
        }
    };
}

def_method! {
    "Adds a region to the meta"
    add_region(exec, name: String, begin: usize, end: usize) -> () {
        exec.app.meta_state.meta.low.regions.insert(NamedRegion {
            name,
            desc: String::new(),
            region: Region { begin, end },
        });
        Ok(())
    }
}

def_method! {
    "Loads a file"
    load_file(exec, path: String) -> () {
        exec.app
            .load_file(path.into(), true, exec.font, &mut exec.gui.msg_dialog)
            .map_err(|e| e.into_lua_err())?;
        Ok(())
    }
}

def_method! {
    "Sets the value pointed to by the bookmark to an integer value"
    bookmark_set_int(exec, name: String, val: i64) -> () {
        let bm = exec
            .app
            .meta_state
            .meta
            .bookmark_by_name_mut(&name)
            .ok_or("no such bookmark".into_lua_err())?;
        bm.write_int(&mut exec.app.data[bm.offset..], val).map_err(|e| e.into_lua_err())?;
        Ok(())
    }
}

def_method! {
    "Fills a named region with a pattern"
    region_pattern_fill(exec, name: String, pattern: String) -> () {
        let reg = exec
            .app
            .meta_state
            .meta
            .region_by_name_mut(&name)
            .ok_or("no such region".into_lua_err())?;
        let pat = parse_pattern_string(&pattern).map_err(|e| e.into_lua_err())?;
        exec.app.data[reg.region.begin..=reg.region.end].pattern_fill(&pat);
        Ok(())
    }
}

def_method! {
    "Returns an array containing the offsets of the find results"
    find_result_offsets(exec,) -> Vec<usize> {
        Ok(exec.gui.find_dialog.results_vec.clone())
    }
}

def_method! {
    "Reads an unsigned 8 bit integer at `offset`"
    read_u8(exec, offset: usize) -> u8 {
        match exec.app.data.get(offset) {
            Some(byte) => Ok(*byte),
            None => Err("out of bounds".into_lua_err()),
        }
    }
}

def_method! {
    "Reads a little endian unsigned 32 bit integer at `offset`"
    read_u32_le(exec, offset: usize) -> u32 {
        match exec
        .app
        .data
        .get(offset..offset + 4)
    {
        Some(slice) => value_type::U32Le::from_byte_slice(slice)
            .ok_or_else(|| "Failed to convert".into_lua_err()),
        None => Err("out of bounds".into_lua_err()),
    }
    }
}

def_method! {
    "Fills a range from `start` to `end` with the value `fill`"
    fill_range(exec, start: usize, end: usize, fill: u8) -> () {
        match exec
              .app
              .data
              .get_mut(start..end) {
            Some(slice) => {
                slice.fill(fill);
                Ok(())
            }
            None => Err("out of bounds".into_lua_err()),
        }
    }
}

def_method! {
    "Sets the dirty region to `begin..=end`"
    set_dirty_region(exec, begin: usize, end: usize) -> () {
        exec.app.edit_state.dirty_region = Some(Region { begin, end });
        Ok(())
    }
}

def_method! {
    "Save the currently opened document (its dirty ranges)"
    save(exec,) -> () {
        exec.app.save(&mut exec.gui.msg_dialog).into_lua_err()?;
        Ok(())
    }
}

def_method! {
    "Returns the offset pointed to by the bookmark `name`"
    bookmark_offset(exec, name: String) -> usize {
        match exec
             .app
             .meta_state
             .meta
             .bookmark_by_name_mut(&name)
        {
            Some(bm) => Ok(bm.offset),
            None => Err(format!("no such bookmark: {name}").into_lua_err()),
        }
    }
}

def_method! {
    "Adds a bookmark with name `name`, pointing at `offset`"
    add_bookmark(exec, offset: usize, name: String) -> () {
        exec.app.meta_state.meta.bookmarks.push(Bookmark {
            offset,
            label: name,
            desc: String::new(),
            value_type: ValueType::None,
        });
        Ok(())
    }
}

def_method! {
    "Finds a hex string in the format '99 aa bb ...' format, and returns its offset"
    find_hex_string(exec, hex_string: String) -> Option<usize> {
        let mut offset = None;
        crate::gui::find_dialog::find_hex_string(&hex_string, &exec.app.data, |off| {
            offset = Some(off);
        }).into_lua_err()?;
        Ok(offset)
    }
}

def_method! {
    "Set the cursor to `offset`, center the view on the cursor, and flash the cursor"
    focus_cursor(exec, offset: usize) -> () {
        exec.app.search_focus(offset);
        Ok(())
    }
}

def_method! {
    "Reoffsets all bookmarks based on the difference between a bookmark's and the cursor's offsets"
    reoffset_bookmarks_cursor_diff(exec, bookmark_name: String) -> () {
        let bookmark = exec.app.meta_state.meta.bookmark_by_name_mut(&bookmark_name).context("No such bookmark").into_lua_err()?;
        let offset = bookmark.offset;
        exec.app.reoffset_bookmarks_cursor_diff(offset);
        Ok(())
    }
}

def_method! {
    "Prints to the debug log of Hexerator. You need to open the debug panel first (F12)"
    dbg(_exec, value: String) -> () {
        gamedebug_core::per!("lua: {value}");
        Ok(())
    }
}

impl<'app, 'gui, 'font> UserData for LuaExecContext<'app, 'gui, 'font> {
    fn add_methods<'lua, T: mlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
        forr::forr! {$t:ty in [
            add_region,
            load_file,
            bookmark_set_int,
            region_pattern_fill,
            find_result_offsets,
            read_u8,
            read_u32_le,
            fill_range,
            set_dirty_region,
            save,
            bookmark_offset,
            add_bookmark,
            find_hex_string,
            focus_cursor,
            reoffset_bookmarks_cursor_diff,
            dbg
            ] $* {
            methods.add_method_mut($t::NAME, $t::call);
        }};
    }
}

pub fn exec_lua(
    lua: &Lua,
    lua_script: &str,
    app: &mut App,
    gui: &mut Gui,
    font: &Font,
) -> Result<(), mlua::prelude::LuaError> {
    lua.scope(|scope| {
        let chunk = lua.load(lua_script);
        let fun = chunk.into_function()?;
        let app = scope.create_nonstatic_userdata(LuaExecContext {
            app: &mut *app,
            gui,
            font,
        })?;
        if let Some(env) = fun.environment() {
            env.set("hx", app)?;
        }
        fun.call(())?;
        Ok(())
    })
}