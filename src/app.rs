use std::{
    io::{self},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event as CEvent, KeyCode};

use ratatui::{prelude::*, widgets::*};

use crate::commands::{get_all_packages, PackageVersionInfo};
use crate::ui::*;

#[derive(Clone)]
pub enum ListFilter {
    All,
    Explicit,
    Dependencies,
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

pub struct App {
    pub packages_list: StatefulList,
}

impl App {
    pub fn new() -> Self {
        Self {
            packages_list: StatefulList::new(),
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
        // Set up a thread and channel to listen for user input.
        // If no input is detected in 200ms then emit a tick.
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(200);
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("poll works") {
                    if let CEvent::Key(key) = event::read().expect("can read events") {
                        tx.send(UiEvent::Input(key)).expect("can send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = tx.send(UiEvent::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

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
                        MenuItem::PackageList => self.render_package_table(frame, chunks[1]),
                    }
                })
                .unwrap();

            // Input handling
            match rx.recv().unwrap() {
                UiEvent::Input(event) => match event.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('p') => active_menu_item = MenuItem::PackageList,
                    KeyCode::Up | KeyCode::Char('k') => self.packages_list.previous(),
                    KeyCode::Down | KeyCode::Char('j') => self.packages_list.next(),
                    KeyCode::Char('g') => self.go_top(),
                    KeyCode::Char('G') => self.go_bottom(),
                    KeyCode::Char('a') => self.change_filter(ListFilter::All),
                    KeyCode::Char('i') => self.change_filter(ListFilter::Explicit),
                    KeyCode::Char('d') => self.change_filter(ListFilter::Dependencies),
                    // TODO: Pressing the key brings up an input for the filter.
                    KeyCode::Char('f') => self.change_filter(ListFilter::Search("al".to_string())),
                    _ => {}
                },
                UiEvent::Tick => {}
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
                ListFilter::Explicit => !p.is_dependency,
                ListFilter::Dependencies => p.is_dependency,
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
