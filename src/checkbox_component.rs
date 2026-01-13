//! Checkbox component for the Logos TUI library

use crate::tui::{Component, UIEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent};

/// A checkbox component
pub struct Checkbox {
    id: String,
    label: String,
    position: (u16, u16),
    checked: bool,
    focused: bool,
}

impl Checkbox {
    pub fn new(id: &str, label: &str, x: u16, y: u16) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            position: (x, y),
            checked: false,
            focused: false,
        }
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }
}

impl Component for Checkbox {
    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Mouse(MouseEvent { kind: crossterm::event::MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
                let (x, y) = self.position;
                
                // Check if the click is on the checkbox (within a 2x1 area)
                if *column >= x && *column < x + 2 && *row == y {
                    self.checked = !self.checked;
                    true
                } else {
                    false
                }
            }
            UIEvent::Key(KeyEvent { code: KeyCode::Char(' '), .. }) => {
                self.checked = !self.checked;
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, frame: &mut Frame, _area: Rect) {
        let (x, y) = self.position;
        
        let checkbox_area = Rect {
            x,
            y,
            width: 2,
            height: 1,
        };
        
        let checkbox_char = if self.checked { "[x]" } else { "[ ]" };
        let checkbox_style = if self.focused {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        
        let checkbox = Paragraph::new(checkbox_char)
            .style(checkbox_style);
        
        frame.render_widget(checkbox, checkbox_area);
        
        // Render the label next to the checkbox
        let label_area = Rect {
            x: x + 4,
            y,
            width: self.label.len() as u16,
            height: 1,
        };
        
        let label = Paragraph::new(self.label.as_str());
        frame.render_widget(label, label_area);
    }

    fn id(&self) -> &str {
        &self.id
    }
}