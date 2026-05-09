use crossterm::event::{Event as CEvent, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{layout::*, style::*, text::*, widgets::*, Frame};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::commands::{get_all_packages, PackageType, PackageVersionInfo};
use crate::config::Config;
use crate::shared::{centered_rect, join_vec, render_empty_list, StatefulList};

#[derive(Clone)]
pub enum ListFilter {
    All,
    Explicit,
    Orphans,
    Foreign,
    Search(String),
}

pub enum InputOutcome {
    Consumed,
    PassThrough,
}

pub struct PackageListView {
    pub packages_list: StatefulList<PackageVersionInfo>,
    pub filter_input: Input,
    pub filter_popup_open: bool,
    pub list_filter: ListFilter,
}

impl PackageListView {
    pub fn new() -> Self {
        Self {
            packages_list: StatefulList::new(),
            filter_input: Input::default(),
            filter_popup_open: false,
            list_filter: ListFilter::All,
        }
    }

    pub fn load(&mut self) {
        let packages = get_all_packages("pacman");
        self.packages_list.items = packages.clone();
        self.packages_list.filtered_items = packages;
    }

    pub fn render(&mut self, frame: &mut Frame<'_>, chunk: Rect, config: &Config) {
        if self.packages_list.filtered_items.is_empty() {
            render_empty_list(frame, chunk);
            return;
        }

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
            .split(chunk);

        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Packages")
            .border_type(BorderType::Plain);

        let items: Vec<_> = self
            .packages_list
            .filtered_items
            .iter()
            .map(|p| {
                let style = match p.package_type {
                    PackageType::Explicit => Style::default(),
                    PackageType::Orphan => Style::default()
                        .fg(config.theme.orphan_fg)
                        .bg(config.theme.orphan_bg),
                    PackageType::Foreign => Style::default()
                        .fg(config.theme.foreign_fg)
                        .bg(config.theme.foreign_bg),
                };

                ListItem::new(Line::from(vec![Span::styled(p.name.clone(), style)]))
            })
            .collect();

        let index = self.packages_list.state.selected().unwrap_or_default();

        let mut selected_package = self.packages_list.filtered_items[index].clone();

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

        let package_details = selected_package.get_details();
        let details_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title(package_details.name + " Details")
            .border_type(BorderType::Plain);

        let details_text = vec![
            Line::styled(
                "Version: ".to_owned() + &package_details.version.clone(),
                Style::default(),
            ),
            Line::styled(
                "Description: ".to_owned() + &package_details.description.clone(),
                Style::default(),
            ),
            Line::styled(
                "Depends On: ".to_owned() + &join_vec(package_details.depends_on.to_owned()),
                Style::default(),
            ),
            Line::styled(
                "Optional dependencies: ".to_owned()
                    + &join_vec(package_details.optional_dependencies).to_owned(),
                Style::default(),
            ),
            Line::styled(
                "Optional for: ".to_owned() + &join_vec(package_details.optional_for).to_owned(),
                Style::default(),
            ),
            Line::styled(
                "Installed size: ".to_owned() + &package_details.installed_size.clone(),
                Style::default(),
            ),
            Line::styled(
                "Install reason: ".to_owned() + &package_details.installed_reason.clone(),
                Style::default(),
            ),
        ];

        let details_display = Paragraph::new(details_text)
            .block(details_block)
            .wrap(Wrap { trim: false });

        frame.render_stateful_widget(list, layout[0], &mut self.packages_list.state);
        frame.render_widget(details_display, layout[1]);
    }

    pub fn render_popup(&mut self, frame: &mut Frame<'_>) {
        let block = Block::default()
            .title("Filter by name")
            .borders(Borders::ALL);
        let area = centered_rect(60, 20, frame.size());

        let input = Paragraph::new(self.filter_input.value())
            .style(Style::default())
            .block(block);

        let width = area.width.max(3) - 3;
        let scroll = self.filter_input.visual_scroll(width as usize);
        frame.set_cursor(
            area.x + (self.filter_input.visual_cursor().max(scroll) - scroll) as u16 + 1,
            area.y + 1,
        );

        frame.render_widget(Clear, area);
        frame.render_widget(input, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InputOutcome {
        if self.filter_popup_open {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        self.change_filter(ListFilter::Search(
                            self.filter_input.value().to_string(),
                        ));
                        self.filter_input.reset();
                        self.filter_popup_open = false;
                    }
                    KeyCode::Esc => {
                        self.filter_input.reset();
                        self.filter_popup_open = false;
                    }
                    _ => {
                        self.filter_input.handle_event(&CEvent::Key(key));
                    }
                }
            }
            return InputOutcome::Consumed;
        }

        match key.code {
            // TODO: Add removing packages from list.
            KeyCode::Up | KeyCode::Char('k') => self.packages_list.previous(),
            KeyCode::Down | KeyCode::Char('j') => self.packages_list.next(),
            KeyCode::Char('g') => self.packages_list.go_top(),
            KeyCode::Char('G') => self.packages_list.go_bottom(),
            KeyCode::Char('r') => self.change_filter(ListFilter::All),
            KeyCode::Char('e') => self.change_filter(ListFilter::Explicit),
            KeyCode::Char('o') => self.change_filter(ListFilter::Orphans),
            KeyCode::Char('f') => self.change_filter(ListFilter::Foreign),
            KeyCode::Char('s') => self.filter_popup_open = true,
            _ => return InputOutcome::PassThrough,
        }
        InputOutcome::PassThrough
    }

    fn change_filter(&mut self, filter: ListFilter) {
        self.list_filter = filter;
        self.packages_list.filtered_items = self
            .packages_list
            .items
            .clone()
            .into_iter()
            .filter(|p| match self.list_filter.clone() {
                ListFilter::All => true,
                ListFilter::Explicit => p.package_type == PackageType::Explicit,
                ListFilter::Orphans => p.package_type == PackageType::Orphan,
                ListFilter::Foreign => p.package_type == PackageType::Foreign,
                ListFilter::Search(s) => p.name.contains(s.as_str()),
            })
            .collect();

        self.packages_list.go_top();
    }
}
