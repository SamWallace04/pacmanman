use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::{env, fs};

use chrono::Utc;
use color_eyre::eyre::{eyre, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::*, style::*, text::*, widgets::*, Frame};
use serde::{Deserialize, Serialize};

use crate::commands::{search_packages, CloudPackage};
use crate::config::Config;
use crate::shared::StatefulList;

pub struct CloudView {
    pub cloud_packages_list: StatefulList<CloudPackage>,
}

#[derive(Clone, Serialize, Deserialize)]
struct CacheData {
    packages: Vec<CloudPackage>,
    epoch: String,
}

impl CloudView {
    pub fn new() -> Self {
        Self {
            cloud_packages_list: StatefulList::new(),
        }
    }

    pub fn load(&mut self) {
        let file_path = cache_file_path();
        let packages: Vec<CloudPackage>;

        if let Ok(cache) = read_file(&file_path) {
            packages = cache;
        } else {
            packages = search_packages("pacman", "");
            write_file(&file_path, &packages).unwrap_or_default();
        }

        self.cloud_packages_list.items = packages.clone();
        self.cloud_packages_list.filtered_items = packages.clone();
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

fn cache_file_path() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pacmanman")
        .join("packages.txt")
}

fn read_file(path: &Path) -> Result<Vec<CloudPackage>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // TODO: Read first line and check epoch. If it's ok continue to convert the rest from JSON.
    let cache: CacheData = serde_json::from_str(&contents)?;

    let now = Utc::now();
    let epoch = now.timestamp();
    if cache.epoch.parse::<i64>()? < epoch - 21600 {
        return Err(eyre!("Outdated cache"));
    }

    Ok(cache.packages)
}

fn write_file(path: &Path, packages: &[CloudPackage]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut write_file = File::create(path)?;

    let now = Utc::now();
    let epoch = now.timestamp();
    let data = CacheData {
        packages: packages.to_owned(),
        epoch: format!("{}", epoch),
    };

    let json = serde_json::to_string(&data)?;

    write_file.write_all(json.as_bytes())?;
    Ok(())
}
