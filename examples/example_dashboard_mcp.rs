use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::style::{Color};
use ratatui::layout::{Constraint, Direction, Layout};
use std::io;

mod consts_b738x;
use crate::consts_b738x::DisplayType::{Digits, Gb};

fn button_pg(label: &str, state: bool) -> Paragraph {
    let bg_color = if state { Color::Green } else { Color::Red };
    let block = Block::default().borders(Borders::ALL).bg(bg_color);
    let paragraph = Paragraph::new(label).centered().block(block);
    paragraph
}

fn display_pg(label: &str) -> Paragraph {
    let block = Block::default().borders(Borders::RIGHT | Borders::LEFT);
    let paragraph = Paragraph::new(label).centered().block(block);
    paragraph
}

fn main() -> io::Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.draw(|f| {
        let size = f.area();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Top row
                Constraint::Length(3), // Second row
                Constraint::Min(1),    // Remaining space
                Constraint::Length(3), // Command row
            ])
            .split(size);

        // Top row layout (A fields)
        let a_blocks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(consts_b738x::MCP_A_FIELDS.iter().map(|f| f.constraint).collect::<Vec<Constraint>>())
            .split(main_layout[0]);

        // Second row layout (B fields)
        let b_blocks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(consts_b738x::MCP_B_FIELDS.iter().map(|f| f.constraint).collect::<Vec<Constraint>>())
            .split(main_layout[1]);

        // Render A fields
        for (block, field) in a_blocks.iter().zip(&consts_b738x::MCP_A_FIELDS) {
            let display = &field.field_type;
            match display {
                Digits => f.render_widget(display_pg(field.name), *block),
                Gb => f.render_widget(button_pg(field.name, true), *block),
            }
        }

        // Render B fields
        for (block, field) in b_blocks.iter().zip(&consts_b738x::MCP_B_FIELDS) {
            let display = &field.field_type;
            match display {
                Digits => f.render_widget(display_pg(field.name), *block),
                Gb => f.render_widget(button_pg(field.name, true), *block),
            }
        }
    })?;

    Ok(())
}
