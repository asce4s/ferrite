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
                    if self.selected_idx == 0 {
                        self.selected_idx = self.items.len().saturating_sub(1);
                    } else {
                        self.selected_idx -= 1;
                    }
                }
                KeyCode::Right => {
                    if !self.items.is_empty() {
                        self.selected_idx = (self.selected_idx + 1) % self.items.len();
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

        if self.items.len() > 1 {
            Paragraph::new("<").render(arrow_left, frame.buffer_mut());
        }

        let text = self
            .items
            .get(self.selected_idx)
            .map(|item| (self.transform)(item))
            .unwrap_or_else(|| String::from(""));

        Paragraph::new(text).render(value, frame.buffer_mut());

        if self.items.len() > 1 {
            Paragraph::new(">").render(arrow_right, frame.buffer_mut());
        }

        self.base_block()
            .style(style)
            .title(self.label.clone())
            .render(area, frame.buffer_mut());
    }

    fn get_value(&self) -> Option<T> {
        self.items.get(self.selected_idx).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_field_navigation() {
        let mut field = SelectField {
            selected_idx: 0,
            label: "Test".into(),
            index: 0,
            items: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            transform: |s: &String| s.clone(),
        };

        // Right
        field.handle_event(0, &Event::Key(KeyCode::Right.into()));
        assert_eq!(field.selected_idx, 1);
        field.handle_event(0, &Event::Key(KeyCode::Right.into()));
        assert_eq!(field.selected_idx, 2);

        // Wrap Right
        field.handle_event(0, &Event::Key(KeyCode::Right.into()));
        assert_eq!(field.selected_idx, 0);

        // Left
        field.handle_event(0, &Event::Key(KeyCode::Left.into()));
        assert_eq!(field.selected_idx, 2);
        field.handle_event(0, &Event::Key(KeyCode::Left.into()));
        assert_eq!(field.selected_idx, 1);
    }

    #[test]
    fn test_select_field_get_value() {
        let select = SelectField {
            selected_idx: 1,
            label: "Test".to_string(),
            index: 0,
            items: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            transform: |s: &String| s.clone(),
        };

        assert_eq!(select.get_value(), Some("B".to_string()));
    }
}
