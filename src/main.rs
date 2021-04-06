use std::{
    sync::mpsc,
    time::{Duration, Instant},
    thread,
    error::Error,
};
use argh::FromArgs;
use tui::{
    Terminal, 
    backend::{CrosstermBackend, Backend},
};
use crossterm::{
    execute, 
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode
    },
    event::{self, Event as CEvent, KeyCode},
};

#[cfg(windows)]
use std::io::{stdout, Stdout};

#[cfg(not(windows))]
use crossterm::event::DisableMouseCapture;

#[cfg(not(windows))]
use std::io::{stderr, Stderr};

mod app;
mod ui;
mod cwd;

// Events sent by the input handling thread
enum Event<I> {
    Input(I),
    Tick, // Needed to keep alive window resizing
}

/**
Partial Commander
    A simple console based directory tree navigator

Navigation keys:
    Q|ESC                   Quit the application
    Backspace|Left arrow    Move up a directory
    Enter|Right arrow       Move into selected directory
    Up|Down                 Movce within a directory
*/
#[derive(Debug, FromArgs)]
struct Cli {
    /// path to start in
    #[argh(positional)]
    path: Option<String>,
    /// time in ms between ticks for input handling
    #[argh(option, default = "250", short = 't')]
    tick_rate: u64,
    /// upon exiting, pass last directory to shell
    #[argh(switch, short = 'k')]
    keep: bool,
    /// display directories only
    #[argh(switch, short = 'd')]
    dirs: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();

    // Setup terminal gui stuff
    enable_raw_mode()?;
    let mut out = out();
    execute!(out, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    // Setup input handling thread
    let (tx, rx) = mpsc::channel();
    let (tx_stop_thread, rx_stop_thread) = mpsc::channel();
    let tick_rate = Duration::from_millis(cli.tick_rate);
    let input_thread_handle = thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
            if let Ok(_) = rx_stop_thread.recv_timeout(Duration::from_millis(1)) { return; }
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    if let Err(_) = tx.send(Event::Input(key)) {
                        return;
                    }
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });


    terminal.clear()?;

    let mut app = app::App::new(cli.path, cli.dirs);
    let mut current_directory = ui::Folder::new(app.list_folder_str());
    current_directory.set_items(app.list_folder_str());
    current_directory.select(Some(0));

    let mut parent_directory = ui::Folder::new(app.list_parent_str());
    parent_directory.set_items(app.list_parent_str());
    parent_directory.select(app.current_folder_parent_idx());

    loop {
        terminal.draw(|f| ui::draw(f, &mut app, &mut current_directory.state, &mut parent_directory.state))?;
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') | KeyCode::Esc => { break }
                KeyCode::Down => { current_directory.next() }
                KeyCode::Up => { current_directory.previous() }
                KeyCode::Left | KeyCode::Backspace => { 
                    if let Ok(()) = app.up(current_directory.state.selected()) {
                        current_directory.set_items(app.list_folder_str());
                        current_directory.select(parent_directory.state.selected());
                        parent_directory.set_items(app.list_parent_str());
                        parent_directory.select(app.current_folder_parent_idx());
                    };
                }
                KeyCode::Right | KeyCode::Enter => {
                    if let Some(idx) = current_directory.state.selected() {
                        if let Ok(items) = app.down(idx) {
                            parent_directory.set_items(items);
                            parent_directory.select(Some(idx));
                            current_directory.set_items(app.list_folder_str());
                            if let Some(idx) = app.pop_last_visited_idx() {
                                current_directory.select(Some(idx));
                            } else {
                                current_directory.select(Some(0));
                            }
                        }
                    }
                }
                _ => {}
            }
            _ => {}
        }
    }
    cleanup(&mut terminal)?;
    tx_stop_thread.send(())?;
    input_thread_handle.join().unwrap_or(());
    if cli.keep {
        crate::cwd::cwd_host(app.current_path())?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn cleanup<B: Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(windows)]
fn cleanup<B: Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

#[cfg(windows)]
fn out() -> Stdout { stdout() }

#[cfg(not(windows))]
fn out() -> Stderr { stderr() }