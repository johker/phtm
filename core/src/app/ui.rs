use std::time::Duration;

use symbols::line;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Cell, LineGauge, Paragraph, Row, Table};
use tui::{symbols, Frame};
use tui_logger::TuiLoggerWidget;

use super::actions::Actions;
use super::state::AppState;
use crate::app::App;
use crate::pushr::push::state::PushState;
use crate::pushr::push::item::Item;

pub fn draw<B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let size = rect.size();
    check_size(&size);

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(10),
                Constraint::Length(3),
                Constraint::Length(12),
            ]
            .as_ref(),
        )
        .split(size);

    // Title
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    // Stacks 
    let stack_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Length(10), Constraint::Length(10), Constraint::Length(10)].as_ref())
        .split(chunks[1]);

    let body = draw_bool_stack(app.is_loading(), app.app_state(), app.push_state());
    rect.render_widget(body, stack_chunks[0]);

    let body = draw_int_stack(app.is_loading(), app.app_state(), app.push_state());
    rect.render_widget(body, stack_chunks[1]);

    let body = draw_float_stack(app.is_loading(), app.app_state(), app.push_state());
    rect.render_widget(body, stack_chunks[2]);

    let body = draw_exec_stack(app.is_loading(), app.app_state(), app.push_state());
    rect.render_widget(body, stack_chunks[3]);

    // Body & Help
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(32)].as_ref())
        .split(chunks[2]);

    let body = draw_body(app.is_loading(), app.app_state(), app.push_state());
    rect.render_widget(body, body_chunks[0]);


    let help = draw_help(app.actions());
    rect.render_widget(help, body_chunks[1]);

    // Duration LineGauge
    if let Some(duration) = app.app_state().duration() {
        let duration_block = draw_duration(duration);
        rect.render_widget(duration_block, chunks[3]);
    }

    // Logs
    let logs = draw_logs();
    rect.render_widget(logs, chunks[4]);
}

fn draw_title<'a>() -> Paragraph<'a> {
    Paragraph::new("Plop with TUI")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}

fn check_size(rect: &Rect) {
    if rect.width < 52 {
        panic!("Require width >= 52, (got {})", rect.width);
    }
    if rect.height < 28 {
        panic!("Require height >= 28, (got {})", rect.height);
    }
}

fn draw_body<'a>(loading: bool, app_state: &AppState, push_state: &PushState) -> Paragraph<'a> {
    let initialized_text = if app_state.is_initialized() {
        "Initialized"
    } else {
        "Not Initialized !"
    };
    let loading_text = if loading { "Loading..." } else { "" };
    let sleep_text = if let Some(sleeps) = app_state.count_sleep() {
        format!("Sleep count: {}", sleeps)
    } else {
        String::default()
    };
    let tick_text = if let Some(ticks) = app_state.count_tick() {
        format!("Tick count: {}", ticks)
    } else {
        String::default()
    };

    Paragraph::new(vec![
        Spans::from(Span::raw(initialized_text)),
        Spans::from(Span::raw(loading_text)),
        Spans::from(Span::raw(sleep_text)),
        Spans::from(Span::raw(tick_text)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
             .title("Body")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )

}
fn draw_item(item: &Item) -> String {
    match item {
        Item::List { items } =>return "( ... )".to_string(),
        _ => return item.to_string(),
    }
}

fn draw_exec_stack<'a>(loading: bool, app_state: &AppState, push_state: &PushState) -> Paragraph<'a>{


    let exec_pos1_text = if let Some(item) = push_state.exec_stack.get(0) {
        format!("1: {}", draw_item(item))
    } else {
        format!("1: ")
    };
    let exec_pos2_text = if let Some(item) = push_state.exec_stack.get(1) {
        format!("2: {}", draw_item(item))
    } else {
        format!("2: ")
    };
    let exec_pos3_text = if let Some(item) = push_state.exec_stack.get(2) {
        format!("3: {}", draw_item(item))
    } else {
        format!("3: ")
    };

    Paragraph::new(vec![
        Spans::from(Span::raw(exec_pos1_text)),
        Spans::from(Span::raw(exec_pos2_text)),
        Spans::from(Span::raw(exec_pos3_text)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
             .title("EXEC")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}


fn draw_bool_stack<'a>(loading: bool, app_state: &AppState, push_state: &PushState) -> Paragraph<'a>{

    let bool_pos1_text = if let Some(item) = push_state.bool_stack.get(0) {
        format!("1: {}", item)
    } else {
        format!("1: ")
    };
    let bool_pos2_text = if let Some(item) = push_state.bool_stack.get(1) {
        format!("2: {}", item)
    } else {
        format!("2: ")
    };
    let bool_pos3_text = if let Some(item) = push_state.bool_stack.get(2) {
        format!("3: {}", item)
    } else {
        format!("3: ")
    };

    Paragraph::new(vec![
        Spans::from(Span::raw(bool_pos1_text)),
        Spans::from(Span::raw(bool_pos2_text)),
        Spans::from(Span::raw(bool_pos3_text)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .title("BOOL")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

fn draw_float_stack<'a>(loading: bool, app_state: &AppState, push_state: &PushState) -> Paragraph<'a>{

    let float_pos1_text = if let Some(item) = push_state.float_stack.get(0) {
        format!("1: {}", item)
    } else {
        String::default()
    };
    let float_pos2_text = if let Some(item) = push_state.float_stack.get(1) {
        format!("2: {}", item)
    } else {
        String::default()
    };
    let float_pos3_text = if let Some(item) = push_state.float_stack.get(2) {
        format!("3: {}", item)
    } else {
        String::default()
    };

    Paragraph::new(vec![
        Spans::from(Span::raw(float_pos1_text)),
        Spans::from(Span::raw(float_pos2_text)),
        Spans::from(Span::raw(float_pos3_text)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .title("FLOAT")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

fn draw_int_stack<'a>(loading: bool, app_state: &AppState, push_state: &PushState) -> Paragraph<'a>{

    let int_pos1_text = if let Some(item) = push_state.int_stack.get(0) {
        format!("1: {}", item)
    } else {
        format!("1: ")
    };
    let int_pos2_text = if let Some(item) = push_state.int_stack.get(1) {
        format!("2: {}", item)
    } else {
        format!("2: ")
    };
    let int_pos3_text = if let Some(item) = push_state.int_stack.get(2) {
        format!("3: {}", item)
    } else {
        format!("3: ")
    };

    Paragraph::new(vec![
        Spans::from(Span::raw(int_pos1_text)),
        Spans::from(Span::raw(int_pos2_text)),
        Spans::from(Span::raw(int_pos3_text)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .title("INT")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

fn draw_duration(duration: &Duration) -> LineGauge {
    let sec = duration.as_secs();
    let label = format!("{}s", sec);
    let ratio = sec as f64 / 10.0;
    LineGauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sleep duration"),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .line_set(line::THICK)
        .label(label)
        .ratio(ratio)
}

fn draw_help(actions: &Actions) -> Table {
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    for action in actions.actions().iter() {
        let mut first = true;
        for key in action.keys() {
            let help = if first {
                first = false;
                action.to_string()
            } else {
                String::from("")
            };
            let row = Row::new(vec![
                Cell::from(Span::styled(key.to_string(), key_style)),
                Cell::from(Span::styled(help, help_style)),
            ]);
            rows.push(row);
        }
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Help"),
        )
        .widths(&[Constraint::Length(11), Constraint::Min(20)])
        .column_spacing(1)
}

fn draw_logs<'a>() -> TuiLoggerWidget<'a> {
    TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::Blue))
        .block(
            Block::default()
                .title("Logs")
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black))
}
