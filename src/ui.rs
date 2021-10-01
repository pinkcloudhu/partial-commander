use std::error::Error;
use tui::{
    backend::Backend,
    Frame,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    layout::{Layout, Constraint, Direction, Margin, Alignment, Rect, Corner},
    text::{Span, Spans},
    style::{Color, Style, Modifier},
};
use crate::app::App;

pub struct UiData {
    parent_title: String,
    current_title: String,
    parent_list: Vec<String>,
    current_list: Vec<String>,
    current_last_selected: usize,
    child_list: Result<Vec<String>, Box<dyn Error>>,
    child_content: Option<Vec<String>>,
    child_is_folder: bool,
}
impl UiData {
    pub fn new() -> Self {
        UiData {
            parent_title: String::from(""),
            current_title: String::from(""),
            parent_list: vec!(),
            current_list: vec!(),
            current_last_selected: 0,
            child_list: Ok(vec!()),
            child_content: None,
            child_is_folder: true,
        }
    }
}

fn draw_empty_dir<B: Backend>(f: &mut Frame<B>, app: &mut App, rect: Rect, ) {
    let mut explanation = "";
    if app.is_dirs_only() {
        explanation = "Note that there may be files, but are hidden because of directory mode";
    }
    let s = vec![
        Spans::from(vec![
            Span::styled("Empty directory", Style::default().add_modifier(Modifier::ITALIC)),
        ]),
        Spans::from(vec![
            Span::styled(explanation, Style::default().fg(Color::DarkGray)),
        ]),
    ];
    let paragraph = Paragraph::new(s)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, rect);
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App, redraw_only: bool, ui_data: &mut UiData, current_directory_state: &mut ListState, parent_directory_state: &mut ListState) {
    if !redraw_only {
        ui_data.parent_title = app.parent_folder();
        ui_data.parent_list = app.parent_name();
        ui_data.current_title = app.current_folder();
        ui_data.current_list = app.list_cwd_child_names();
    }
    if let Some(idx) = current_directory_state.selected() {
        if (ui_data.current_last_selected != idx) | !redraw_only {
            ui_data.current_last_selected = idx;
            ui_data.child_is_folder = app.child_is_folder(idx);
            if ui_data.child_is_folder {
                ui_data.child_list =  app.list_cwd_nth_child_children_names(idx);
            } else {
                ui_data.child_content = app.read_child_file(idx);
            }
        }
    }

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
            Span::styled(ui_data.parent_title.clone(), Style::default().fg(Color::Green))
        )
        .borders(Borders::ALL);
    f.render_widget(block, chunks[0]);

    let folder_contents = ui_data.parent_list.clone();
    let items: Vec<ListItem> = folder_contents.iter().map(|f| ListItem::new(f.as_str())).collect();
    let list = List::new(items)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::White).bg(Color::DarkGray))
        .start_corner(Corner::TopRight)
        .highlight_symbol("> ");
    f.render_stateful_widget(list, parent_block, parent_directory_state);


    // current dir
    let block = Block::default()
        .title(
            Span::styled(ui_data.current_title.clone(), Style::default().fg(Color::Green))
        )
        .borders(Borders::ALL);
    f.render_widget(block, chunks[1]);

    let folder_contents = ui_data.current_list.clone();
    if folder_contents.len() == 0 {
        draw_empty_dir(f, app, current_block);
        return;
    } else {
        let items: Vec<ListItem> = folder_contents.iter().map(|f| ListItem::new(f.as_str())).collect();
        let list = List::new(items)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::White).bg(Color::Blue))
        .highlight_symbol("> ");
        f.render_stateful_widget(list, current_block, current_directory_state);
    }

    // child item/dir
    let block = Block::default()
        .borders(Borders::ALL);
    f.render_widget(block, chunks[2]);

    if let Some(_) = current_directory_state.selected() {
        if ui_data.child_is_folder {
            if let Ok(folder_contents) = &ui_data.child_list {
                if folder_contents.len() == 0 {
                    draw_empty_dir(f, app, contents_block);
                } else {
                    let items: Vec<ListItem> = folder_contents.iter().map(|f| ListItem::new(f.as_str())).collect();
                    let list = List::new(items)
                    .style(Style::default().fg(Color::Gray));
                    f.render_widget(list, contents_block);
                }
            }
        } else { // is a file
            if let Some(content) = ui_data.child_content.clone() {
                let s: Vec<Spans> = content.iter().map(|s| Spans::from(s.as_str())).collect();
                let paragraph = Paragraph::new(s)
                    .wrap(Wrap { trim: false });
                    f.render_widget(paragraph, contents_block);
            } else {
                let s = vec![
                    Spans::from(Span::styled("Can't display file content", Style::default().add_modifier(Modifier::ITALIC))),
                    Spans::from(Span::styled("Only text files are supported for preview, or missing filesystem permission", Style::default().fg(Color::DarkGray))),
                ];
                let paragraph = Paragraph::new(s)
                    .alignment(Alignment::Center)
                    .wrap(Wrap {trim: true});
                f.render_widget(paragraph, contents_block);
            }
        }
    }
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

    pub fn next(&mut self) {
        if self.items.len() == 0 { return }
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

    pub fn previous(&mut self) {
        if self.items.len() == 0 { return }
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

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn select(&mut self, idx: Option<usize>) {
        if let Some(idx) = idx {
            let i = if idx >= self.items.len() {
                self.items.len()
            } else if idx <= 0 { 0 } else { idx };
            self.state.select(Some(i));
        } else {
            self.unselect();
        }
    }
}
