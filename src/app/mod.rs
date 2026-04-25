mod events;
mod ui;

use std::collections::HashSet;

use crate::core::{DriveItem, drive_handle};
use clap::Parser;
use ratatui::layout::Flex;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, TableState},
};

pub struct MountApp {
    pub exit: bool,
    pub table_state: TableState,
    pub row_length: usize,
    pub selected_option: Option<SelectedRow>,
    pub drives: Vec<DriveItem>,
    pub selected_rows: HashSet<usize>,
    pub status_message: String,
    pub popup_msg: String,
    pub script_view: ScriptView,
    pub mode: AppMode,
    pub args: CliArgs,
}

impl MountApp {
    pub fn new(drives: Vec<DriveItem>, args: CliArgs) -> Self {
        MountApp {
            exit: false,
            table_state: TableState::default(),
            row_length: 0,
            selected_option: None,
            drives,
            selected_rows: HashSet::new(),
            status_message: "".to_string(),
            popup_msg: "".to_string(),
            script_view: ScriptView {
                script_scroll: 0,
                script_view_height: 0,
            },
            mode: AppMode::MainTable,
            args,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum AppMode {
    MainTable,
    DriveInfoPopup,
    ScriptPreview,
    ScriptPopup,
    MessagePopup,
}

pub struct SelectedRow {
    drive_name: String,
    id: usize,
}

pub struct ScriptView {
    pub script_scroll: u16,
    pub script_view_height: u16,
}

struct PopupPar<'a> {
    pop_block: Paragraph<'a>,
    area: Rect,
}

#[derive(Parser, Debug)]
#[command(name = "Hypr-Mount")]
#[command(version = "b1.0.1")]
#[command(about = "A TUI drive mounter", long_about = None)]
pub struct CliArgs {
    #[arg(long, group = "mode")]
    pub auto_mount: bool,
    #[arg(long, group = "mode")]
    pub generate_service: bool,
    #[arg(long, group = "mode")]
    pub show_censor_uuid: bool,
}

impl MountApp {
    fn draw(&mut self, frame: &mut Frame) {
        if let Some(sel_op) = &mut self.selected_option
            && let Some(idx) = self.table_state.selected()
            && let Some(drive) = self.drives.get(idx)
        {
            sel_op.drive_name = drive.name.to_string();
        }
        let vertical_layout = Layout::vertical([Constraint::Min(0), Constraint::Length(2)]);
        let area = frame.area();
        let areas = vertical_layout.split(area);
        self.main_mount_table_draw(frame, area, areas);
    }
}

impl MountApp {
    fn row_selection_handle(&mut self) {
        if let Some(selected) = &self.selected_option
            && self.row_length > 0
        {
            if self.selected_rows.contains(&selected.id) {
                self.selected_rows.remove(&selected.id);
            } else {
                self.selected_rows.insert(selected.id);
            }
            self.status_message = format!("Selected {} drive(s)", self.selected_rows.len());
        }
    }
    fn mount_unmount_selected_cursor(&mut self) {
        if self.selected_rows.is_empty()
            && let Some(idx) = self.table_state.selected()
            && let Some(drive) = self.drives.get_mut(idx)
            && let Some(uuid) = &drive.uuid
        {
            if !drive.is_mounted {
                match drive_handle::mount_drive(uuid) {
                    Ok(()) => {
                        drive.is_mounted = true;
                    }
                    Err(err) => {
                        self.status_message = err.to_string();
                    }
                }
                return;
            }

            match drive_handle::unmount_drive(uuid) {
                Ok(()) => {
                    drive.is_mounted = false;
                }
                Err(err) => {
                    self.status_message = err.to_string();
                }
            }
        }
    }
    fn mount_unmount_selected_drives(&mut self) {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut error_msg = String::new();

        for (idx, drive) in self.drives.iter_mut().enumerate() {
            if !self.selected_rows.contains(&idx) {
                continue; // if drives not selected just skip
            }

            if let Some(ref uuid) = drive.uuid {
                if drive.is_mounted {
                    match drive_handle::unmount_drive(uuid) {
                        Ok(()) => {
                            drive.is_mounted = !drive.is_mounted;
                            self.selected_rows.remove(&idx);
                            success_count += 1;
                        }
                        Err(err) => {
                            error_msg = err.to_string();
                            error_count += 1;
                        }
                    }
                } else {
                    match drive_handle::mount_drive(uuid) {
                        Ok(()) => {
                            drive.is_mounted = !drive.is_mounted;
                            self.selected_rows.remove(&idx);
                            success_count += 1;
                        }
                        Err(err) => {
                            error_msg = err.to_string();
                            error_count += 1;
                        }
                    }
                }
            } else {
                error_msg = String::from("Drive has no UUID");
                error_count += 1;
            }
        }

        if success_count > 0 && error_count == 0 {
            self.status_message = format!("Successfully processed {} drive(s)", success_count);
        } else if error_count > 0 {
            self.status_message = format!(
                "Error processing {} of {} drive(s): {}",
                error_count,
                success_count + error_count,
                error_msg
            );
        } else if success_count == 0 {
            self.status_message = String::from("No drives selected for mounting/unmounting");
        }
    }

    fn clear_all_selection(&mut self) {
        self.selected_rows.clear();
        self.status_message = String::from("All drive selections cleared")
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
