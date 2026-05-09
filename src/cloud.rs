use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::*, style::*, text::*, widgets::*, Frame};

use crate::commands::{search_packages, CloudPackage};
use crate::config::Config;
use crate::shared::StatefulList;

pub struct CloudView {
    pub cloud_packages_list: StatefulList<CloudPackage>,
}

impl CloudView {
    pub fn new() -> Self {
        Self {
            cloud_packages_list: StatefulList::new(),
        }
    }

    pub fn load(&mut self) {
        //TODO: Write to a file and refresh it every x hours (day?) to not hammer the pacman api
        let cloud_packages = search_packages("pacman", "");
        self.cloud_packages_list.items = cloud_packages.clone();
        self.cloud_packages_list.filtered_items = cloud_packages;
    }

    pub fn render(&mut self, frame: &mut Frame<'_>, chunk: Rect, config: &Config) {
        // TODO: Change this to a table
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunk);

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Packages")
            .border_type(BorderType::Plain);

        let items: Vec<_> = self
            .cloud_packages_list
            .filtered_items
            .iter()
            .map(|p| {
                let style = match p.is_installed {
                    true => Style::default(),
                    false => Style::default()
                        .fg(config.theme.foreign_fg)
                        .bg(config.theme.foreign_bg),
                };

                ListItem::new(Line::from(vec![Span::styled(p.name.clone(), style)]))
            })
            .collect();

        let index = self
            .cloud_packages_list
            .state
            .selected()
            .unwrap_or_default();

        let selected_package = self.cloud_packages_list.filtered_items[index].clone();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(config.theme.selected_fg)
                    .bg(config.theme.selected_bg)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        let details_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Details")
            .border_type(BorderType::Plain);

        let details_text = vec![
            Line::styled(
                "Version: ".to_owned() + &selected_package.version.clone(),
                Style::default(),
            ),
            Line::styled(
                "Description: ".to_owned() + &selected_package.description.clone(),
                Style::default(),
            ),
            Line::styled(
                "Source: ".to_owned() + &selected_package.source.clone(),
                Style::default(),
            ),
        ];

        let details_display = Paragraph::new(details_text)
            .block(details_block)
            .wrap(Wrap { trim: false });

        frame.render_stateful_widget(list, layout[0], &mut self.cloud_packages_list.state);
        frame.render_widget(details_display, layout[1]);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // TODO: Add sorting.
        // TODO: Add the ability to install packages from list.
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => self.cloud_packages_list.previous(),
            KeyCode::Down | KeyCode::Char('j') => self.cloud_packages_list.next(),
            KeyCode::Char('g') => self.cloud_packages_list.go_top(),
            KeyCode::Char('G') => self.cloud_packages_list.go_bottom(),
            _ => {}
        }
    }
}
