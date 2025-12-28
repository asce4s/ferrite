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

    fn get_value(&self) -> String {
        self.input.value().to_string()
    }
}
