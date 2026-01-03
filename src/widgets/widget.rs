use ratatui::{
    Frame,
    crossterm::event::Event,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType},
};

pub trait InputField<T> {
    fn handle_event(&mut self, focus_index: u8, event: &Event);
    fn render(&self, frame: &mut Frame, focus_index: &u8, area: Rect);
    fn get_value(&self) -> T;

    fn base_style(&self, is_focused: bool) -> Style {
        if is_focused {
            return Style::default().fg(Color::Yellow);
        }
        Style::default()
    }

    fn base_block<'a>(&self) -> Block<'a> {
        Block::bordered().border_type(BorderType::Plain)
    }
}
