use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::style::{Color};
use ratatui::layout::{Constraint, Direction, Layout};
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use crossterm::event;
use crossterm::event::{poll, KeyCode, KeyEventKind};
use xplane_udp::session::Session;

mod consts_b738x;
use crate::consts_b738x::DisplayType::{Digits, Gb};

fn button_pg(label: &str, state: bool) -> Paragraph {
    let bg_color = if state { Color::Green } else { Color::Red };
    let block = Block::default().borders(Borders::ALL).bg(bg_color);
    let paragraph = Paragraph::new(label).centered().block(block);
    paragraph
}

fn display_pg(value: &str) -> Paragraph {
    Paragraph::new(vec![
        Line::from(""),
        Line::from(value),
        Line::from(""),
    ]).alignment(Alignment::Center).block(Block::default().borders(Borders::NONE))
}

fn main() -> io::Result<()> {
    // let mut session = Session::auto_discover_default(10000)?;
    let mut session = Session::manual(
        SocketAddr::from(([10, 0, 0, 10], 49000)),
        SocketAddr::from(([10, 0, 0, 10], 49001)),
    )?;

    session.connect()?;
    session.run();

    // Subscribe to datarefs in A and B fields
    for field in consts_b738x::MCP_A_FIELDS.iter().chain(consts_b738x::MCP_B_FIELDS.iter()) {
        let dr_type = match field.field_type {
            Digits => xplane_udp::dataref_type::DataRefType::Int,
            Gb => xplane_udp::dataref_type::DataRefType::Int,
        };
        session.subscribe(&field.dataref, 1, dr_type)?;
    }


    let mut terminal = ratatui::init();
    loop {
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
                let newest_value = match session.get_dataref(&field.dataref) {
                    Some(xplane_udp::dataref_type::DataRefValueType::Int(v)) => v,
                    _ => 0,
                };

                let display = &field.field_type;
                match display {
                    Digits => f.render_widget(display_pg(newest_value.to_string().as_str()), *block),
                    Gb => f.render_widget(button_pg(field.name, newest_value == 1), *block),
                }
            }

            // Render B fields
            for (block, field) in b_blocks.iter().zip(&consts_b738x::MCP_B_FIELDS) {
                let newest_value = match session.get_dataref(&field.dataref) {
                    Some(xplane_udp::dataref_type::DataRefValueType::Int(v)) => v,
                    _ => 0,
                };

                let display = &field.field_type;
                match display {
                    Digits => f.render_widget(display_pg(newest_value.to_string().as_str()), *block),
                    Gb => f.render_widget(button_pg(field.name, newest_value == 1), *block),
                }
            }
        })?;

        if poll(Duration::from_millis(100)).unwrap() {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}
