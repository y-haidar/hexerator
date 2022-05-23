use std::ffi::OsString;

use sfml::graphics::Vertex;

use crate::{input::Input, EditTarget, FindDialog, InteractMode, Region};

/// The hexerator application state
pub struct App {
    pub rows: usize,
    // Number of columns in the view
    pub cols: usize,
    // Maximum number of visible hex columns that can be shown on screen.
    // ascii is double this amount.
    pub max_visible_cols: usize,
    /// Path to the file we're editing
    pub path: OsString,
    pub dirty: bool,
    pub data: Vec<u8>,
    pub show_debug_panel: bool,
    pub col_width: u8,
    // The editing byte offset
    pub cursor: usize,
    // The byte offset in the data from which the view starts viewing data from
    pub starting_offset: usize,
    pub vertices: Vec<Vertex>,
    pub input: Input,
    pub interact_mode: InteractMode,
    pub top_gap: i64,
    pub view_x: i64,
    pub view_y: i64,
    // The amount scrolled per frame in view mode
    pub scroll_speed: i64,
    pub colorize: bool,
    // The value of the cursor on the previous frame. Used to determine when the cursor changes
    pub cursor_prev_frame: usize,
    pub edit_target: EditTarget,
    pub row_height: u8,
    pub show_text: bool,
    // The half digit when the user begins to type into a hex view
    pub hex_edit_half_digit: Option<u8>,
    pub u8_buf: String,
    pub find_dialog: FindDialog,
    pub selection: Option<Region>,
    pub select_begin: Option<usize>,
    pub fill_text: String,
    pub backup_path: OsString,
}

pub enum CursorViewStatus {
    Inside,
    Before,
    After,
}

impl App {
    pub fn new(path: OsString) -> Self {
        let data = std::fs::read(&path).unwrap();
        let top_gap = 30;
        let cursor = 0;
        Self {
            rows: 67,
            cols: 48,
            max_visible_cols: 75,
            path: path.clone(),
            dirty: false,
            data,
            show_debug_panel: false,
            col_width: 26,
            cursor,
            starting_offset: 0,
            vertices: Vec::new(),
            input: Input::default(),
            interact_mode: InteractMode::View,
            // The top part where the top panel is. You should try to position stuff so it's not overdrawn
            // by the top panel
            top_gap,
            // The x pixel offset of the scrollable view
            view_x: 0,
            // The y pixel offset of the scrollable view
            view_y: -top_gap,
            // The amount scrolled per frame in view mode
            scroll_speed: 4,
            colorize: true,
            // The value of the cursor on the previous frame. Used to determine when the cursor changes
            cursor_prev_frame: cursor,
            edit_target: EditTarget::Hex,
            row_height: 16,
            show_text: true,
            // The half digit when the user begins to type into a hex view
            hex_edit_half_digit: None,
            u8_buf: String::new(),
            find_dialog: FindDialog::default(),
            selection: None,
            select_begin: None,
            fill_text: String::new(),
            backup_path: {
                let mut new = path;
                new.push(".hexerator_bak");
                new
            },
        }
    }
    pub fn reload(&mut self) {
        self.data = std::fs::read(&self.path).unwrap();
        self.dirty = false;
    }
    pub fn save(&mut self) {
        std::fs::write(&self.path, &self.data).unwrap();
        self.dirty = false;
    }
    pub fn toggle_debug(&mut self) {
        self.show_debug_panel ^= true;
        gamedebug_core::toggle();
    }
    pub fn ascii_display_x_offset(&self) -> i64 {
        self.cols as i64 * i64::from(self.col_width) + 12
    }
    pub fn cursor_view_status(
        cursor: usize,
        starting_offset: usize,
        rows: usize,
        cols: usize,
    ) -> CursorViewStatus {
        if cursor < starting_offset {
            CursorViewStatus::Before
        } else if cursor > starting_offset + rows * cols {
            CursorViewStatus::After
        } else {
            CursorViewStatus::Inside
        }
    }
    pub fn search_focus(
        cursor: &mut usize,
        starting_offset: &mut usize,
        off: usize,
        rows: usize,
        cols: usize,
    ) {
        // Focus the search result in the hex view
        *cursor = off;
        match Self::cursor_view_status(*cursor, *starting_offset, rows, cols) {
            CursorViewStatus::Before => {
                *starting_offset = off.saturating_sub((rows - 1) * (cols - 1))
            }
            CursorViewStatus::After => *starting_offset = off - (rows + cols),
            CursorViewStatus::Inside => {}
        }
    }
}