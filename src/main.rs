use std::{
    sync::mpsc,
    io::{stdout},
    time::{Duration, Instant},
    thread,
    error::Error,
};
use argh::FromArgs;
use tui::{Terminal, backend::CrosstermBackend};
use crossterm::{
    execute, 
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode
    },
    event::{self, DisableMouseCapture, Event as CEvent, KeyCode},
};

mod app;

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
*/
#[derive(Debug, FromArgs)]
struct Cli {
    /// time in ms between ticks
    #[argh(option, default = "250")]
    tick_rate: u64,
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

    loop {
        
        terminal.draw(app::draw)?;
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') | KeyCode::Esc => { break }
                _ => {} // TODO: pass to app to process
            },
            Event::Tick => {
                // TODO: pass to app, possibly redraw also
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    return Ok(())
}
