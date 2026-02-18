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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyModifiers, KeyEvent, KeyEventKind, KeyEventState};

    #[test]
    fn test_select_field_navigation() {
        let mut select = SelectField {
            selected_idx: 0,
            label: "Test".to_string(),
            index: 0,
            items: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            transform: |s: &String| s.clone(),
        };

        let right_event = Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

        let left_event = Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

        select.handle_event(0, &right_event);
        assert_eq!(select.selected_idx, 1);

        select.handle_event(0, &right_event);
        assert_eq!(select.selected_idx, 2);

        // Test boundary (Right)
        select.handle_event(0, &right_event);
        assert_eq!(select.selected_idx, 2);

        select.handle_event(0, &left_event);
        assert_eq!(select.selected_idx, 1);

        select.handle_event(0, &left_event);
        assert_eq!(select.selected_idx, 0);

        // Test boundary (Left)
        select.handle_event(0, &left_event);
        assert_eq!(select.selected_idx, 0);
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

        assert_eq!(select.get_value(), "B");
    }
}
