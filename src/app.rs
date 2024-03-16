use std::{
    io::{self},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event as CEvent, KeyCode};

use ratatui::{prelude::*, widgets::*};

use crate::commands::{get_explicit_packages, PackageVersionInfo};
use crate::ui::*;

pub struct StatefulList {
    pub state: ListState,
    pub packages: Vec<PackageVersionInfo>,
    pub last_selected: Option<usize>,
}

pub struct App {
    pub list: StatefulList,
}

impl App {
    pub fn new() -> Self {
        Self {
            list: StatefulList::new(),
        }
    }

    fn go_top(&mut self) {
        self.list.state.select(Some(0));
    }

    fn go_bottom(&mut self) {
        self.list.state.select(Some(self.list.packages.len() - 1));
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
                .draw(|rect| {
                    let size = rect.size();
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

                    let menu = render_menu(&menu_titles);

                    let tabs = render_tabs(menu, active_menu_item);
                    rect.render_widget(tabs, chunks[0]);

                    let footer = render_footer();
                    rect.render_widget(footer, chunks[2]);

                    match active_menu_item {
                        MenuItem::PackageList => {
                            let package_chunks = Layout::default()
                                .direction(Direction::Horizontal)
                                .constraints(
                                    [Constraint::Percentage(10), Constraint::Percentage(90)]
                                        .as_ref(),
                                )
                                .split(chunks[1]);
                            let (left, right) = render_package_table(self);
                            rect.render_stateful_widget(
                                left,
                                package_chunks[0],
                                &mut self.list.state,
                            );
                            rect.render_widget(right, package_chunks[1])
                        }
                    }
                })
                .unwrap();

            // Input handling
            match rx.recv().unwrap() {
                UiEvent::Input(event) => match event.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('p') => active_menu_item = MenuItem::PackageList,
                    KeyCode::Up | KeyCode::Char('k') => self.list.previous(),
                    KeyCode::Down | KeyCode::Char('j') => self.list.next(),
                    KeyCode::Char('g') => self.go_top(),
                    KeyCode::Char('G') => self.go_bottom(),
                    _ => {}
                },
                UiEvent::Tick => {}
            }
        }
    }
}

impl StatefulList {
    fn new() -> Self {
        StatefulList {
            state: ListState::default(),
            packages: get_explicit_packages(&"pacman"),
            last_selected: None,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.packages.len() - 1 {
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
                    self.packages.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.last_selected.unwrap_or(0),
        };
        self.state.select(Some(i));
    }
}
