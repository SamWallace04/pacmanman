use std::io;

use crossterm::event::{self, Event as CEvent, KeyCode};

use ratatui::{prelude::*, widgets::*};

use crate::cloud::CloudView;
use crate::config::{Config, ConfigFile};
use crate::package_list::{InputOutcome, PackageListView};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MenuItem {
    PackageList,
    Cloud,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::PackageList => 0,
            MenuItem::Cloud => 1,
        }
    }
}

pub struct App {
    pub config: Config,
    pub active_menu: MenuItem,
    pub package_list: PackageListView,
    pub cloud: CloudView,
}

impl App {
    pub fn new() -> Self {
        Self {
            active_menu: MenuItem::PackageList,
            package_list: PackageListView::new(),
            cloud: CloudView::new(),
            // Try loading the config file, if there is an issue fallback on the hardcoded default.
            config: ConfigFile::parse(
                confy::load("pacmanman", None).unwrap_or(ConfigFile::default()),
            )
            .unwrap_or(ConfigFile::parse(ConfigFile::default()).unwrap()),
        }
    }

    pub fn load_packages(&mut self) {
        self.package_list.load();
        self.cloud.load();
    }

    pub fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        let menu_titles = vec!["Packages", "All", "Quit"];

        loop {
            terminal
                .draw(|frame| {
                    let size = frame.size();
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints(
                            [
                                Constraint::Length(3),
                                Constraint::Min(2),
                                Constraint::Length(3),
                            ]
                            .as_ref(),
                        )
                        .split(size);

                    let menu = create_menu(&menu_titles);
                    render_tabs(menu, self.active_menu, frame, chunks[0]);
                    render_footer(frame, chunks[2], self.active_menu);

                    match self.active_menu {
                        MenuItem::PackageList => {
                            self.package_list.render(frame, chunks[1], &self.config);
                            if self.package_list.filter_popup_open {
                                self.package_list.render_popup(frame);
                            }
                        }
                        MenuItem::Cloud => {
                            self.cloud.render(frame, chunks[1], &self.config);
                        }
                    }
                })
                .unwrap();

            if let CEvent::Key(key) = event::read().unwrap() {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                let outcome = match self.active_menu {
                    MenuItem::PackageList => self.package_list.handle_key(key),
                    MenuItem::Cloud => {
                        self.cloud.handle_key(key);
                        InputOutcome::PassThrough
                    }
                };

                if matches!(outcome, InputOutcome::PassThrough) {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('p') => self.active_menu = MenuItem::PackageList,
                        KeyCode::Char('a') => self.active_menu = MenuItem::Cloud,
                        _ => {}
                    }
                }
            }
        }
    }
}

fn create_menu<'a>(menu_titles: &Vec<&'a str>) -> Vec<Line<'a>> {
    menu_titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Line::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::White)),
            ])
        })
        .collect()
}

fn render_tabs<'a>(
    menu: Vec<Line<'a>>,
    active_menu_item: MenuItem,
    frame: &mut Frame<'_>,
    chunk: Rect,
) {
    let tabs = Tabs::new(menu)
        .select(active_menu_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"));

    frame.render_widget(tabs, chunk);
}

fn render_footer(frame: &mut Frame<'_>, chunk: Rect, current_window: MenuItem) {
    let footer = match current_window {
        MenuItem::PackageList => Paragraph::new("\nUse ↓/j and ↑/k to move, g/G to go top/bottom. e to show explicitly installed packages, o to show orphan packages, f to show foreign packages (AUR/manual install), s to search, r to reset the filter").centered(),
        MenuItem::Cloud => Paragraph::new("\nUse ↓/j and ↑/k to move, g/G to go top/bottom. s to search, r to reset the filter").centered(),
    };
    frame.render_widget(footer, chunk);
}
