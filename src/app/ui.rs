use crate::app::{self};
use crate::app::{AppMode, MountApp, PopupPar};
use crate::core::DriveConfig;
use crate::core::mount;
use crate::core::mount::{driveconf_script_gen, to_automount_conf};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Line, Span, Style, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Cell, Clear, Paragraph, Row, Table, TableState};

impl MountApp {
    pub(crate) fn main_mount_table_draw(
        &mut self,
        frame: &mut Frame<'_>,
        area: Rect,
        areas: std::rc::Rc<[Rect]>,
    ) {
        let rows: Vec<ratatui::widgets::Row> = self
            .drives
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let status_text = if item.is_mounted {
                    Span::styled("MOUNTED", Style::new().fg(Color::Green).bold())
                } else {
                    Span::styled("UNMOUNTED", Style::new().fg(Color::Red))
                };

                let status_selected = if self.selected_rows.contains(&i) {
                    Span::styled(" ✔ ", Style::new().fg(Color::Green).bold())
                } else {
                    Span::styled(" · ", Style::new().fg(Color::DarkGray))
                };

                let mut uuid_res = String::new(); //maybe there better way of doing that
                if self.args.uncensor_uuid
                    && let Some(uuid) = &item.uuid
                {
                    uuid_res = uuid.to_string();
                } else if let Some(uuid) = &item.uuid {
                    uuid_res = format!("{}...", uuid.split_at(8).0);
                }

                Row::new(vec![
                    Cell::from(status_selected),
                    Cell::from(item.name.as_str()).style(if self.selected_rows.contains(&i) {
                        Style::default().fg(Color::White)
                    } else {
                        Style::default()
                    }),
                    Cell::from(item.device_path.as_str()),
                    Cell::from(item.fstype.as_str()),
                    Cell::from(item.size.as_str()),
                    Cell::from(status_text),
                    Cell::from(uuid_res),
                ])
            })
            .collect();
        self.row_length = rows.len();

        if areas.len() < 2 {
            return;
        }

        let table_area = areas[0];
        let status_area = areas[1];

        let mount_table = self.create_table_widget(&rows);

        let status_block = Block::default()
            .borders(ratatui::widgets::Borders::TOP)
            .border_style(Style::new().fg(Color::DarkGray)); // Dim line

        let status_text = Line::from(vec![
            Span::styled(
                " STATUS ",
                Style::new()
                    .fg(Color::Cyan)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::raw("│ "),
            Span::raw(&self.status_message),
        ]);

        frame.render_stateful_widget(&mount_table, table_area, &mut self.table_state);
        let status_widget = Paragraph::new(status_text)
            .block(status_block)
            .alignment(ratatui::layout::Alignment::Left);
        frame.render_widget(status_widget, status_area);

        match self.mode {
            AppMode::DriveInfoPopup => {
                if let Some(driveinfo_popup) = self.draw_drive_popup(area) {
                    frame.render_widget(Clear, driveinfo_popup.area);
                    frame.render_widget(driveinfo_popup.pop_block, driveinfo_popup.area);
                }
            }
            AppMode::ScriptPopup => {
                let drive_conf_auto = to_automount_conf(&self.drives, &self.selected_rows);
                let popup = self.draw_automount_popup(area, drive_conf_auto);
                frame.render_widget(Clear, popup.area);
                frame.render_widget(popup.pop_block, popup.area);
            }
            AppMode::ScriptPreview => {
                let drive_conf_auto = to_automount_conf(&self.drives, &self.selected_rows);
                let script_scroll = self.script_view.script_scroll;

                let script_content = if let Ok(script) = if self.args.uncensor_uuid {
                    mount::driveconf_to_string(drive_conf_auto)
                } else {
                    mount::driveconf_to_string_censored(drive_conf_auto)
                } {
                    script
                } else {
                    String::from("Could not generate preview.")
                };

                MountApp::automount_window_scr_preview(
                    frame,
                    area,
                    &mut self.table_state,
                    mount_table,
                    script_content,
                    script_scroll,
                    &mut self.script_view.script_view_height,
                );
            }
            AppMode::MessagePopup => {
                let popup = self.message_popup(area);
                frame.render_widget(Clear, popup.area);
                frame.render_widget(popup.pop_block, popup.area);
            }
            _ => {}
        }
    }

    fn message_popup(&mut self, area: Rect) -> PopupPar<'_> {
        let block = Block::bordered().title(" System ");
        let text = Paragraph::new(self.popup_msg.clone())
            .style(Style::new().fg(Color::Green).bold())
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        let area = app::popup_area(area, 50, 20);
        PopupPar {
            pop_block: text,
            area,
        }
    }

    fn create_table_widget<'a>(
        &self,
        rows: &'a [ratatui::widgets::Row<'a>],
    ) -> ratatui::widgets::Table<'a> {
        let widths = [
            Constraint::Length(5),  // " Sel "
            Constraint::Length(14), // "/dev/nvme0n1p1"
            Constraint::Fill(1),    // Mount Point
            Constraint::Length(10), // "fstype"
            Constraint::Length(10), // "100.5G"
            Constraint::Length(12), // "UNMOUNTED"
            Constraint::Fill(1),    // UUID
        ];

        let key_style = Style::new().bg(Color::Cyan).fg(Color::Black).bold();

        let desc_style = Style::new().fg(Color::Gray);

        let instructions = Line::from(vec![
            // Group 1: Selection
            Span::styled(" Enter ", key_style),
            Span::styled(" Select  ", desc_style),
            // Group 2: Actions
            Span::styled(" m ", key_style),
            Span::styled(" Mount  ", desc_style),
            Span::styled(" c ", key_style),
            Span::styled(" Clear  ", desc_style),
            Span::styled(" x ", key_style),
            Span::styled(" Generate  ", desc_style),
            // Group 3: Views
            Span::styled(" g ", key_style),
            Span::styled(" Preview  ", desc_style),
            Span::styled(" p ", key_style),
            Span::styled(" Info  ", desc_style),
            // Group 4: Quit
            Span::styled(" q ", key_style),
            Span::styled(" Quit ", desc_style),
        ]);

        let block = Block::bordered()
            .title(Line::from(" 💾 Hypr-Mount ").bold())
            .title_bottom(instructions)
            .border_set(border::ROUNDED)
            .border_style(Style::new().fg(Color::Cyan));

        Table::new(rows.iter().cloned(), widths)
            .column_spacing(1)
            .style(Style::new())
            .header(
                Row::new(vec![
                    "Sel",
                    "Name",
                    "Mount Point",
                    "Type",
                    "Size",
                    "Mounted",
                    "UUID",
                ])
                .style(Style::new().fg(Color::Cyan).add_modifier(
                    ratatui::style::Modifier::BOLD | ratatui::style::Modifier::UNDERLINED,
                ))
                .bottom_margin(1),
            )
            .block(block)
            .row_highlight_style(
                Style::new()
                    .fg(Color::Yellow)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .highlight_symbol(" ➤ ")
    }

    fn draw_drive_popup(&self, area: Rect) -> Option<PopupPar<'_>> {
        let block = Block::bordered().title("Drive info");
        let selected_drive = self.table_state.selected()?;

        let drive = self.drives.get(selected_drive)?;
        let drive_string = format!(
            "Drive: {}\nMounted as: {}\nType: {}\nSize: {}\nUUID: {}",
            drive.name,
            drive.device_path,
            drive.fstype,
            drive.size,
            drive.uuid.as_ref()?,
        );
        let content_text = Paragraph::new(drive_string).style(Style::default().fg(Color::Yellow));
        let popup = content_text.block(block);
        let area = app::popup_area(area, 45, 25);

        Some(PopupPar {
            pop_block: popup,
            area,
        })
    }
}

impl MountApp {
    fn automount_window_scr_preview(
        frame: &mut Frame<'_>,
        area: Rect,
        table_state: &mut TableState,
        mount_table: Table<'_>,
        script_content: String,
        script_scroll: u16,
        script_height: &mut u16,
    ) {
        frame.render_widget(Clear, area);
        let chunks =
            Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).split(area);
        let top_area = chunks[0];
        let bottom_area = chunks[1];

        let styled_lines: Vec<Line> = script_content
            .lines()
            .enumerate()
            .map(|(i, raw_line)| {
                let line_text = raw_line.to_string();

                // 1. Syntax Highlighting (JSON Style)
                let content_spans: Vec<Span> = if line_text.trim() == "{"
                    || line_text.trim() == "},"
                    || line_text.trim() == "}"
                {
                    // Brackets -> Magenta
                    vec![Span::styled(
                        line_text,
                        Style::new().fg(Color::Magenta).bold(),
                    )]
                } else if let Some((key, value)) = line_text.split_once(':') {
                    // "key": "value" -> Blue : Yellow
                    vec![
                        Span::styled(key.to_string(), Style::new().fg(Color::Blue)),
                        Span::raw(":"),
                        Span::styled(value.to_string(), Style::new().fg(Color::Yellow)),
                    ]
                } else {
                    // Fallback
                    vec![Span::raw(line_text)]
                };

                // 2. Add Line Number
                let mut full_line = vec![Span::styled(
                    format!(" {:02} │ ", i + 1),
                    Style::new().fg(Color::DarkGray),
                )];

                // 3. Combine
                full_line.extend(content_spans);
                Line::from(full_line)
            })
            .collect();

        *script_height = styled_lines.len() as u16;

        let script_widget = Paragraph::new(styled_lines)
            .block(
                Block::bordered()
                    .title(Line::from(" 📜 Config Preview ").bold()) // Changed title to match content
                    .border_set(border::ROUNDED)
                    .border_style(Style::new().fg(Color::LightGreen)),
            )
            .scroll((script_scroll, 0));

        frame.render_widget(script_widget, bottom_area);
        frame.render_stateful_widget(mount_table, top_area, table_state);
    }

    fn draw_automount_popup(&self, area: Rect, drive_conf_auto: Vec<DriveConfig>) -> PopupPar<'_> {
        let block = Block::bordered().title("Script info");
        let script_json = if self.args.uncensor_uuid {
            mount::driveconf_to_string(drive_conf_auto).unwrap_or_default()
        } else {
            mount::driveconf_to_string_censored(drive_conf_auto).unwrap_or_default()
        };

        let content_text = Paragraph::new(script_json).style(Style::default().fg(Color::Yellow));
        let popup = content_text.block(block);
        let area = app::popup_area(area, 60, 40);

        PopupPar {
            pop_block: popup,
            area,
        }
    }
    pub(crate) fn generate_automount_config(&mut self) {
        let drive_conf_auto = to_automount_conf(&self.drives, &self.selected_rows);
        self.popup_msg = match driveconf_script_gen(drive_conf_auto) {
            Ok(()) => String::from(
                "✅ Configuration generated successfully!\n\nSaved to: ~/.config/hypr-mount/automount.json",
            ),
            Err(err) => format!("Error: {}", err),
        };
    }
}
