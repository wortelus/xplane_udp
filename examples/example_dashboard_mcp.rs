use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::style::{Color};
use ratatui::layout::{Constraint, Direction, Layout};
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use crossterm::event;
use crossterm::event::{poll, KeyCode};
use ratatui::{init, restore};
use xplane_udp::command_handler::AlertMessage;
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

async fn execute_command(command: &str, session: &Session) -> io::Result<bool> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let cmd = parts[0];

    if cmd == "quit" {
        return Ok(true);
    }
    else if cmd == "alert" {
        let mut alert = AlertMessage::default();
        let msg = parts[1..].join(" ");

        alert.set_line("wortelus from xplane_udp is saying...", 0)?;
        alert.set_line(&msg, 1)?;
        session.alert(alert).await?;
    } else if cmd.starts_with("cmd") {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() != 2 {
            return Ok(false);
        }
        session.cmd(parts[1]).await?;
    }

    Ok(false)
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    // let mut session = Session::auto_discover_default(10000)?;
    let mut session = Session::manual(
        SocketAddr::from(([10, 0, 0, 10], 49000)),
        SocketAddr::from(([10, 0, 0, 10], 49001)),
    ).await?;

    session.run().await?;

    // Subscribe to datarefs in A and B fields
    for field in consts_b738x::MCP_A_FIELDS.iter().chain(consts_b738x::MCP_B_FIELDS.iter()) {
        let dr_type = match field.field_type {
            Digits => xplane_udp::dataref_type::DataRefType::Int,
            Gb => xplane_udp::dataref_type::DataRefType::Int,
        };
        session.subscribe(&field.dataref, 5, dr_type).await?;
    }

    let mut command_buffer = String::new();
    let mut terminal = init();

    loop {
        terminal.draw(|f| {
            let size = f.area();

            // Outer block with title and border
            let outer_block = Block::default()
                .title("Boeing 737 NG MCP")
                .borders(Borders::ALL);
            f.render_widget(&outer_block, size);

            let inner_area = outer_block.inner(size);
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Top row
                    Constraint::Length(3), // Second row
                    Constraint::Min(1),    // Remaining space
                    Constraint::Length(3), // Command row
                ])
                .split(inner_area);

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
                    Digits => f.render_widget(display_pg(&newest_value.to_string()), *block),
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
                    Digits => f.render_widget(display_pg(&newest_value.to_string()), *block),
                    Gb => f.render_widget(button_pg(field.name, newest_value == 1), *block),
                }
            }

            // Command box
            let command_box = Paragraph::new(format!(">{}", command_buffer))
                .alignment(Alignment::Left)
                .block(Block::default().borders(Borders::NONE).title("Command"));
            f.render_widget(command_box, main_layout[3]);
        })?;

        if poll(Duration::from_millis(100))? {
            if let event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char(ch) => {
                        // Append typed character to command buffer
                        command_buffer.push(ch);
                    }
                    KeyCode::Backspace => {
                        // Remove last character if exists
                        command_buffer.pop();
                    }
                    KeyCode::Enter => {
                        // Execute the command and clear the buffer
                        match execute_command(&command_buffer, &session).await {
                            Ok(true) => break,
                            _ => {}
                        }
                        command_buffer.clear();
                    }
                    _ => {}
                }
            }
        }
    }

    session.shutdown().await;
    restore();
    Ok(())
}
