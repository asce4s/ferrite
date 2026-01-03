use ratatui::{
    Frame,
    crossterm::event::{Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
};

use crate::widgets::widget::InputField;

#[derive(Debug, Default)]
pub struct SelectField<T, F> {
    pub selected_idx: usize,
    pub label: String,
    pub index: u8,
    pub items: Vec<T>,
    pub transform: F,
}

impl<T, F> InputField<T> for SelectField<T, F>
where
    T: Clone,
    F: Fn(&T) -> String,
{
    fn handle_event(&mut self, focus_index: u8, event: &Event) {
        if self.index != focus_index {
            return;
        }
        if let Event::Key(key) = &event {
            match key.code {
                KeyCode::Left => {
                    self.selected_idx = self.selected_idx.saturating_sub(1);
                }
                KeyCode::Right => {
                    if self.items.len() > (self.selected_idx + 1) {
                        self.selected_idx += 1;
                    }
                }
                _ => {}
            }
        };
    }

    fn render(&self, frame: &mut Frame, focus_index: &u8, area: Rect) {
        let is_focused = self.index == *focus_index;
        let style = self.base_style(is_focused);
        let [arrow_left, value, arrow_right] = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .margin(1)
        .areas(area);

        Paragraph::new("<").render(arrow_left, frame.buffer_mut());

        let text = self
            .items
            .get(self.selected_idx)
            .map(|item| (self.transform)(item))
            .unwrap_or_else(|| String::from(""));

        Paragraph::new(text).render(value, frame.buffer_mut());

        Paragraph::new(">").render(arrow_right, frame.buffer_mut());

        self.base_block()
            .style(style)
            .title(self.label.clone())
            .render(area, frame.buffer_mut());
    }

    fn get_value(&self) -> T {
        self.items.get(self.selected_idx).unwrap().clone()
    }
}
