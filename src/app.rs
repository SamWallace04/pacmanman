use std::io::{self};

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEventKind};

use ratatui::{prelude::*, widgets::*};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::commands::{
    get_all_packages, search_packages, CloudPackage, PackageType, PackageVersionInfo,
};
use crate::config::{Config, ConfigFile};
use crate::ui::*;

// TODO: Should the search be separate from other filters? Allowing for subsection filtering.
// eg: Explicit with a certain name.
#[derive(Clone)]
pub enum ListFilter {
    All,
    Explicit,
    Orphans,
    Foreign,
    Search(String),
}

pub trait ListItems {}

pub struct StatefulList<T: ListItems> {
    pub state: ListState,
    pub items: Vec<T>,
    pub filtered_items: Vec<T>,
    pub last_selected: Option<usize>,
    pub list_filter: ListFilter,
}

#[derive(Clone, PartialEq)]
pub enum Screens {
    DetailsList,
    FilterInput,
    CloudList,
}
pub struct App {
    pub packages_list: StatefulList<PackageVersionInfo>,
    pub cloud_packages_list: StatefulList<CloudPackage>,
    pub current_screen: Screens,
    pub previous_screen: Screens,
    pub filter_input: Input,
    pub config: Config,
}

impl App {
    pub fn new() -> Self {
        Self {
            packages_list: StatefulList::new(),
            cloud_packages_list: StatefulList::new(),
            current_screen: Screens::DetailsList,
            previous_screen: Screens::DetailsList,
            filter_input: Input::default(),
            // Try loading the config file, if there is an issue fallback on the hardcoded default.
            config: ConfigFile::parse(
                confy::load("pacmanman", None).unwrap_or(ConfigFile::default()),
            )
            .unwrap_or(ConfigFile::parse(ConfigFile::default()).unwrap()),
        }
    }

    pub fn load_packages(&mut self) {
        let packages = get_all_packages("pacman");
        self.packages_list.items = packages.clone();
        self.packages_list.filtered_items = packages.clone();

        //TODO: Write to a file and refresh it every x hours (day?) to not hammer the pacman api
        let cloud_packages = search_packages("pacman", "");
        self.cloud_packages_list.items = cloud_packages.clone();
        self.cloud_packages_list.filtered_items = cloud_packages.clone();
    }

    pub fn run(&mut self, mut terminal: Terminal<impl Backend>) -> io::Result<()> {
        let menu_titles = vec!["Packages", "All", "Quit"];
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
                            if !self.packages_list.filtered_items.is_empty() {
                                self.render_package_details(frame, chunks[1]);
                            } else {
                                render_empty_list(frame, chunks[1]);
                            }
                        }
                        MenuItem::Cloud => {
                            self.render_cloud_tab(frame, chunks[1]);
                        }
                    }

                    // Render any pop up screens after everything else has been rendered.
                    match self.current_screen {
                        Screens::FilterInput => self.render_filter_popup(frame),
                        Screens::DetailsList => {}
                        Screens::CloudList => {}
                    }
                })
                .unwrap();

            // Input handling
            if let CEvent::Key(key) = event::read().unwrap() {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }

                // App wide keybinds, don't use them when filtering.
                if self.current_screen != Screens::FilterInput {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('p') => {
                            active_menu_item = MenuItem::PackageList;
                            self.current_screen = Screens::DetailsList;
                        }
                        KeyCode::Char('a') => {
                            active_menu_item = MenuItem::Cloud;
                            self.current_screen = Screens::CloudList;
                        }
                        _ => {}
                    }
                }

                match self.current_screen {
                    Screens::DetailsList => match key.code {
                        KeyCode::Up | KeyCode::Char('k') => self.packages_list.previous(),
                        KeyCode::Down | KeyCode::Char('j') => self.packages_list.next(),
                        KeyCode::Char('g') => self.packages_list.go_top(),
                        KeyCode::Char('G') => self.packages_list.go_bottom(),
                        KeyCode::Char('r') => self.change_filter(ListFilter::All),
                        KeyCode::Char('e') => self.change_filter(ListFilter::Explicit),
                        KeyCode::Char('o') => self.change_filter(ListFilter::Orphans),
                        KeyCode::Char('f') => self.change_filter(ListFilter::Foreign),
                        KeyCode::Char('s') => {
                            self.previous_screen = self.current_screen.clone();
                            self.current_screen = Screens::FilterInput
                        }
                        _ => {}
                    },
                    Screens::CloudList => match key.code {
                        KeyCode::Up | KeyCode::Char('k') => self.cloud_packages_list.previous(),
                        KeyCode::Down | KeyCode::Char('j') => self.cloud_packages_list.next(),
                        KeyCode::Char('g') => self.cloud_packages_list.go_top(),
                        KeyCode::Char('G') => self.cloud_packages_list.go_bottom(),
                        _ => {}
                    },
                    Screens::FilterInput if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => {
                            self.change_filter(ListFilter::Search(
                                self.filter_input.value().to_string(),
                            ));
                            self.filter_input.reset();
                            self.current_screen = self.previous_screen.clone();
                        }
                        KeyCode::Esc => {
                            self.filter_input.reset();
                            self.current_screen = self.previous_screen.clone();
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
                ListFilter::Orphans => p.package_type == PackageType::Orphan,
                ListFilter::Foreign => p.package_type == PackageType::Foreign,
                ListFilter::Search(s) => p.name.contains(s.as_str()),
            })
            .collect();

        self.packages_list.go_top();
    }
}

impl<T: ListItems> StatefulList<T> {
    fn new() -> Self {
        StatefulList {
            state: ListState::default(),
            items: vec![],
            last_selected: None,
            list_filter: ListFilter::All,
            filtered_items: vec![],
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

    fn go_top(&mut self) {
        self.state.select(Some(0));
    }

    fn go_bottom(&mut self) {
        if !self.filtered_items.is_empty() {
            self.state.select(Some(self.filtered_items.len() - 1));
        }
    }
}
