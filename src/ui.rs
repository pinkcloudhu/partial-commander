use tui::{
    backend::Backend,
    Frame,
    widgets::{Block, Borders, Paragraph, List, ListItem, ListState},
    layout::{Layout, Constraint, Direction, Margin},
    text::{Span, Spans},
    style::{Color, Style, Modifier},
};
use crate::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App, current_directory_state: &mut ListState, parent_directory_state: &mut ListState) {
    let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ].as_ref())
    .split(f.size());

    let parent_block = chunks[0].inner(&Margin { 
            horizontal: 1,
            vertical: 1,
    });
    let current_block = chunks[1].inner(&Margin {
        horizontal: 1,
        vertical: 1,
    });
    let contents_block = chunks[2].inner(&Margin {
        horizontal: 1,
        vertical: 1,
    });
    
    // Parent dir
    let block = Block::default()
        .title(
            Span::styled(app.parent_folder(), Style::default().fg(Color::Green))
        )
        .borders(Borders::ALL);
    f.render_widget(block, chunks[0]);

    let folder_contents = app.list_parent();
    let items: Vec<ListItem> = folder_contents.iter().map(|f| ListItem::new(f.as_str())).collect();
    let list = List::new(items)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::White).bg(Color::Blue))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, parent_block, parent_directory_state);


    // current dir
    let block = Block::default()
        .title(
            Span::styled(app.current_folder(), Style::default().fg(Color::Green))
        )
        .borders(Borders::ALL);
    f.render_widget(block, chunks[1]);

    let folder_contents = app.list_folder();
    let items: Vec<ListItem> = folder_contents.iter().map(|f| ListItem::new(f.as_str())).collect();
    let list = List::new(items)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::White).bg(Color::Blue))
        .highlight_symbol("> ");
    f.render_stateful_widget(list, current_block, current_directory_state);

    let block = Block::default()
        .title(
            Span::styled("Working example!", Style::default().fg(Color::LightRed).add_modifier(Modifier::ITALIC))
        )
        .borders(Borders::ALL);
    f.render_widget(block, chunks[2]);
}

pub struct Folder {
    items: Vec<String>,
    pub state: ListState
}
impl Folder {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.state = ListState::default();
    }

    // Select the next item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // Select the previous item. This will not be reflected until the widget is drawn in the
    // `Terminal::draw` callback using `Frame::render_stateful_widget`.
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // Unselect the currently selected item if any. The implementation of `ListState` makes
    // sure that the stored offset is also reset.
    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}