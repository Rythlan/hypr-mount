use crate::{
    app::{AppMode, MountApp, SelectedRow},
    core::drive_handle,
};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use std::io;

impl MountApp {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                match self.mode {
                    AppMode::MainTable | AppMode::DriveInfoPopup => {
                        self.handle_key_event(key_event)?
                    }
                    AppMode::ScriptPreview => self.script_view_key_event(key_event)?,
                    AppMode::ScriptPopup => self.script_popup_key_event(key_event)?,
                    AppMode::MessagePopup => self.message_popup_key_event(key_event)?,
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind != KeyEventKind::Press || self.row_length == 0 {
            return Ok(());
        }

        let mut scroll_offs = self.table_state.selected().unwrap_or(0);
        Self::handle_navigation_keys(&key_event, &mut scroll_offs, self.row_length);

        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Enter => {
                self.row_selection_handle();
            }
            KeyCode::Char('m') => {
                if !self.selected_rows.is_empty() {
                    self.mount_unmount_selected_drives();
                } else {
                    self.mount_unmount_selected_cursor();
                }
            }
            KeyCode::Char('c') => {
                self.clear_all_selection();
            }
            KeyCode::Char('p') => {
                if self.mode == AppMode::MainTable {
                    self.mode = AppMode::DriveInfoPopup;
                } else {
                    self.mode = AppMode::MainTable
                }
            }
            KeyCode::Char('x') => {
                self.generate_automount_config();
                self.mode = AppMode::MessagePopup;
            }
            KeyCode::Char('g') => {
                self.mode = AppMode::ScriptPreview;
            }
            KeyCode::Char('r') => match drive_handle::list_drives() {
                Ok(drives) => self.drives = drives,
                Err(err) => self.status_message = err.to_string(),
            },
            _ => {}
        }

        self.table_state.select(Some(scroll_offs));
        if let Some(drive_item) = self.drives.get(scroll_offs)
            && self.row_length > 0
        {
            self.selected_option = Some(SelectedRow {
                drive_name: drive_item.name.to_owned(),
                id: scroll_offs,
            });
        }
        Ok(())
    }

    fn handle_navigation_keys(
        key_event: &crossterm::event::KeyEvent,
        scroll_offset: &mut usize,
        rows_length: usize,
    ) -> bool {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if *scroll_offset > 0 {
                    *scroll_offset -= 1;
                } else {
                    *scroll_offset = rows_length - 1;
                }
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                *scroll_offset = (*scroll_offset + 1) % rows_length;
                true
            }
            _ => false,
        }
    }

    fn script_view_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Esc => self.mode = AppMode::MainTable,
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.script_view.script_view_height > 0 {
                        self.script_view.script_scroll =
                            self.script_view.script_scroll.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.script_view.script_view_height > 1
                        && self.script_view.script_scroll < self.script_view.script_view_height - 1
                    {
                        self.script_view.script_scroll += 1;
                    }
                }
                KeyCode::Char('x') => {
                    self.generate_automount_config();
                    self.mode = AppMode::MessagePopup;
                }
                KeyCode::Char('p') => {
                    self.mode = AppMode::ScriptPopup;
                }
                KeyCode::Char('g') => self.mode = AppMode::MainTable,
                _ => {}
            }
        }
        Ok(())
    }

    fn script_popup_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Esc => self.mode = AppMode::MainTable,
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.script_view.script_view_height > 0 {
                        self.script_view.script_scroll =
                            self.script_view.script_scroll.saturating_sub(1);
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.script_view.script_view_height > 1
                        && self.script_view.script_scroll < self.script_view.script_view_height - 1
                    {
                        self.script_view.script_scroll += 1;
                    }
                }
                KeyCode::Char('x') => {
                    self.generate_automount_config();
                }
                KeyCode::Char('p') => {
                    self.mode = AppMode::ScriptPopup;
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn message_popup_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => self.mode = AppMode::MainTable,
                _ => {}
            }
        }
        Ok(())
    }
}
