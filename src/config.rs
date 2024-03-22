use std::{error::Error, str::FromStr};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ConfigFile {
    theme: ConfigTheme,
}

// TODO: Allow preset theme name or custom values?
#[derive(Serialize, Deserialize)]
struct ConfigTheme {
    fg: String,
    bg: String,
    orphan_fg: String,
    orphan_bg: String,
    foreign_fg: String,
    foreign_bg: String,
    selected_fg: String,
    selected_bg: String,
}

pub struct Config {
    pub theme: Theme,
}

pub struct Theme {
    pub fg: Color,
    pub bg: Color,
    pub orphan_fg: Color,
    pub orphan_bg: Color,
    pub foreign_fg: Color,
    pub foreign_bg: Color,
    pub selected_fg: Color,
    pub selected_bg: Color,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            theme: ConfigTheme::default(),
        }
    }
}

impl Default for ConfigTheme {
    fn default() -> Self {
        Self {
            fg: catppuccin::PALETTE.mocha.colors.text.hex.to_string(),
            bg: catppuccin::PALETTE.mocha.colors.base.hex.to_string(),
            orphan_fg: catppuccin::PALETTE.mocha.colors.crust.hex.to_string(),
            orphan_bg: catppuccin::PALETTE.mocha.colors.red.hex.to_string(),
            foreign_fg: catppuccin::PALETTE.mocha.colors.crust.hex.to_string(),
            foreign_bg: catppuccin::PALETTE.mocha.colors.yellow.hex.to_string(),
            selected_fg: catppuccin::PALETTE.mocha.colors.crust.hex.to_string(),
            selected_bg: catppuccin::PALETTE.mocha.colors.rosewater.hex.to_string(),
        }
    }
}

impl ConfigFile {
    pub fn parse(self) -> Result<Config, Box<dyn Error>> {
        // TODO: Find a better way to map??
        let theme = Theme {
            fg: Color::from_str(&self.theme.fg)?,
            bg: Color::from_str(&self.theme.bg)?,
            orphan_fg: Color::from_str(&self.theme.orphan_fg)?,
            orphan_bg: Color::from_str(&self.theme.orphan_bg)?,
            foreign_fg: Color::from_str(&self.theme.foreign_fg)?,
            foreign_bg: Color::from_str(&self.theme.foreign_bg)?,
            selected_fg: Color::from_str(&self.theme.selected_fg)?,
            selected_bg: Color::from_str(&self.theme.selected_bg)?,
        };

        Ok(Config { theme })
    }
}
