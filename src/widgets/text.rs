use ratatui::{Frame, crossterm::event::Event, layout::Rect, widgets::Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use crate::widgets::widget::InputField;

#[derive(Debug, Default)]
pub struct TextField {
    pub input: Input,
    pub label: String,
    pub index: u8,
    pub mask: Option<String>,
}

impl InputField<String> for TextField {
    fn handle_event(&mut self, focus_index: u8, event: &Event) {
        if self.index == focus_index {
            self.input.handle_event(event);
        }
    }

    fn render(&self, frame: &mut Frame, focus_index: &u8, area: Rect) {
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let is_focused = self.index == *focus_index;
        let style = self.base_style(is_focused);

        let value: String = match self.mask.as_deref() {
            Some(mask_chr) => mask_chr.repeat(self.input.value().chars().count()),
            None => self.input.value().to_string(),
        };

        let input = Paragraph::new(value)
            .style(style)
            .scroll((0, scroll as u16))
            .block(self.base_block().title(self.label.clone()));

        frame.render_widget(input, area);

        if is_focused {
            let x = self.input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((area.x + x as u16, area.y + 1))
        };
    }

    fn get_value(&self) -> Option<String> {
        Some(self.input.value().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    #[test]
    fn test_text_field_handle_event() {
        let mut text_field = TextField {
            input: Input::default(),
            label: "Test".to_string(),
            index: 0,
            mask: None,
        };

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

        text_field.handle_event(0, &event);
        assert_eq!(text_field.get_value(), Some("a".to_string()));

        let event_b = Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });

        text_field.handle_event(0, &event_b);
        assert_eq!(text_field.get_value(), Some("ab".to_string()));
    }

    #[test]
    fn test_text_field_get_value() {
        let text_field = TextField {
            input: Input::new("hello".to_string()),
            label: "Test".to_string(),
            index: 0,
            mask: None,
        };

        assert_eq!(text_field.get_value(), Some("hello".to_string()));
    }
}
