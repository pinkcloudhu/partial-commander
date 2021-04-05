use std::{
    sync::mpsc,
    io::{stdout},
    time::{Duration, Instant},
    thread,
    error::Error,
};
use argh::FromArgs;
use tui::{
    Terminal, 
    backend::CrosstermBackend
};
use crossterm::{
    execute, 
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode
    },
    event::{self, Event as CEvent, KeyCode},
};

#[cfg(not(target_family = "windows"))]
use crossterm::event::DisableMouseCapture;

mod app;
mod ui;

// Events sent by the input handling thread
enum Event<I> {
    Input(I),
    Tick,
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
    /// time in ms between ticks
    #[argh(option, default = "250", short = 't')]
    tick_rate: u64,
    /// path to start in
    #[argh(positional)]
    path: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();

    // Setup terminal gui stuff
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup input handling thread
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(cli.tick_rate);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });


    terminal.clear()?;

    let mut app = app::App::new(cli.path);
    let mut current_directory = ui::Folder::new(app.list_folder());
    current_directory.set_items(app.list_folder());
    current_directory.select(Some(0));

    let mut parent_directory = ui::Folder::new(app.list_parent());
    parent_directory.set_items(app.list_parent());
    parent_directory.select(app.current_folder_parent_idx());

    loop {
        terminal.draw(|f| ui::draw(f, &mut app, &mut current_directory.state, &mut parent_directory.state))?;
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') | KeyCode::Esc => { break }
                KeyCode::Down => { current_directory.next() }
                KeyCode::Up => { current_directory.previous() }
                KeyCode::Left | KeyCode::Backspace => { 
                    app.up(current_directory.state.selected());
                    current_directory.set_items(app.list_folder());
                    current_directory.select(parent_directory.state.selected());
                    parent_directory.set_items(app.list_parent());
                    parent_directory.select(app.current_folder_parent_idx());
                }
                KeyCode::Right | KeyCode::Enter => {
                    if let Some(idx) = current_directory.state.selected() {
                        parent_directory.set_items(app.down(idx));
                        parent_directory.select(Some(idx));
                        current_directory.set_items(app.list_folder());
                        if let Some(idx) = app.pop_last_visited_idx() {
                            current_directory.select(Some(idx));
                        } else {
                            current_directory.select(Some(0));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {
                // TODO: pass to app, possibly redraw also
            }
        }
    }
    cleanup(&mut terminal)?;
    Ok(())
}

#[cfg(target_family = "unix")]
fn cleanup(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(target_family = "windows")]
fn cleanup(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}