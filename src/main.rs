mod app;
mod cloud;
mod commands;
mod config;
mod package_list;
mod shared;

use std::{error::Error, io::stdout};

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::prelude::*;
use ratatui::{backend::CrosstermBackend, Terminal};

use color_eyre::config::HookBuilder;

use crate::app::*;

fn main() -> Result<(), Box<dyn Error>> {
    init_error_hooks()?;

    // Load packages before switching the terminal into raw TUI mode. This keeps
    // file IO and pacman subprocesses from disturbing the interactive terminal.
    let mut app = App::new();
    app.load_packages();

    let terminal = init_terminal()?;
    let run_result = app.run(terminal);
    let restore_result = restore_terminal();

    run_result?;
    restore_result?;

    Ok(())
}

fn init_error_hooks() -> color_eyre::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}

fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal() -> color_eyre::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
