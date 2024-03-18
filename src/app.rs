use std::io::{self};

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};

use ratatui::{prelude::*, widgets::*};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::commands::{get_all_packages, PackageType, PackageVersionInfo};
use crate::ui::*;

// TODO: Should the search be separate from other filters? Allowing for subsection filtering.
// eg: Explicit with a certain name.
#[derive(Clone)]
pub enum ListFilter {
    All,
    Explicit,
    Dependencies,
    Foreign,
    Search(String),
}

// TODO: Turn into a generic??
pub struct StatefulList {
    pub state: ListState,
    pub items: Vec<PackageVersionInfo>,
    pub filtered_items: Vec<PackageVersionInfo>,
    pub last_selected: Option<usize>,
    pub list_filter: ListFilter,
}

#[derive(PartialEq)]
pub enum Screens {
    DetailsList,
    FilterInput,
}
pub struct App {
    pub packages_list: StatefulList,
    pub current_screen: Screens,
    pub filter_input: Input,
}

impl App {
    pub fn new() -> Self {
        Self {
            packages_list: StatefulList::new(),
            current_screen: Screens::DetailsList,
            filter_input: Input::default(),
        }
    }

    fn go_top(&mut self) {
        self.packages_list.state.select(Some(0));
    }

    fn go_bottom(&mut self) {
        self.packages_list
            .state
            .select(Some(self.packages_list.filtered_items.len() - 1));
    }
}

impl App {
    pub fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        let menu_titles = vec!["Packages", "Quit"];
        let mut active_menu_item = MenuItem::PackageList;

        // Render loop
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

                    render_tabs(menu, active_menu_item, frame, chunks[0]);

                    render_footer(frame, chunks[2]);

                    match active_menu_item {
                        MenuItem::PackageList => {
                            if self.packages_list.filtered_items.len() > 0 {
                                self.render_package_details(frame, chunks[1]);
                            } else {
                                render_empty_list(frame, chunks[1]);
                            }
                        }
                    }

                    if self.current_screen == Screens::FilterInput {
                        self.render_filter_popup(frame);
                    }
                })
                .unwrap();

            // Input handling
            if let CEvent::Key(key) = event::read().unwrap() {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                match self.current_screen {
                    Screens::DetailsList => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('p') => active_menu_item = MenuItem::PackageList,
                        KeyCode::Up | KeyCode::Char('k') => self.packages_list.previous(),
                        KeyCode::Down | KeyCode::Char('j') => self.packages_list.next(),
                        KeyCode::Char('g') => self.go_top(),
                        KeyCode::Char('G') => self.go_bottom(),
                        KeyCode::Char('a') => self.change_filter(ListFilter::All),
                        KeyCode::Char('e') => self.change_filter(ListFilter::Explicit),
                        KeyCode::Char('d') => self.change_filter(ListFilter::Dependencies),
                        KeyCode::Char('f') => self.change_filter(ListFilter::Foreign),
                        KeyCode::Char('s') => self.current_screen = Screens::FilterInput,
                        _ => {}
                    },
                    Screens::FilterInput if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => {
                            self.change_filter(ListFilter::Search(
                                self.filter_input.value().to_string(),
                            ));
                            self.filter_input.reset();
                            self.current_screen = Screens::DetailsList;
                        }
                        _ => {
                            self.filter_input.handle_event(&CEvent::Key(key));
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    fn change_filter(&mut self, filter: ListFilter) {
        self.packages_list.list_filter = filter;
        self.packages_list.filtered_items = self
            .packages_list
            .items
            .clone()
            .into_iter()
            .filter(|p| match self.packages_list.list_filter.clone() {
                ListFilter::All => true,
                ListFilter::Explicit => p.package_type == PackageType::Explicit,
                ListFilter::Dependencies => p.package_type == PackageType::Dependency,
                ListFilter::Foreign => p.package_type == PackageType::Foreign,
                // TODO: Make the search a bit smarter??
                ListFilter::Search(s) => p.name.contains(s.as_str()),
            })
            .collect();

        self.go_top();
    }
}

impl StatefulList {
    fn new() -> Self {
        let packages = get_all_packages(&"pacman");
        StatefulList {
            state: ListState::default(),
            items: packages.clone(),
            last_selected: None,
            list_filter: ListFilter::All,
            filtered_items: packages.clone(),
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }
}
