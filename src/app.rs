use tui::{
    backend::CrosstermBackend,
    Frame,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction, Margin},
    text::{Span, Spans},
    style::{Color, Style, Modifier},
};

pub fn draw(f: &mut Frame<CrosstermBackend<std::io::Stdout>>) {
    let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ].as_ref())
    .split(f.size());

    let first_block = chunks[0].inner(&Margin { 
            horizontal: 3,
            vertical: 3,
    });
    
    let block = Block::default()
        .title(
            Span::styled("Partial Commander", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        )
        .borders(Borders::ALL);
    f.render_widget(block, chunks[0]);
    
    let block = Block::default()
        .title(
            Span::styled("This is a test", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK))
        )
        .borders(Borders::ALL);
    f.render_widget(block, chunks[1]);

    let block = Block::default()
        .title(
            Span::styled("Working example!", Style::default().fg(Color::LightRed).add_modifier(Modifier::ITALIC))
        )
        .borders(Borders::ALL);
    f.render_widget(block, chunks[2]);

    let text = vec![
        Spans::from(vec![
            Span::raw("Press q or escape to quit"),
        ])
    ];
    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, first_block);
}