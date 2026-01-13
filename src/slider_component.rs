//! Slider component for the Logos TUI library

use crate::tui::{Component, UIEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent};

/// A slider component for selecting numeric values
pub struct Slider {
    id: String,
    label: String,
    position: (u16, u16),
    size: (u16, u16),
    min_value: f64,
    max_value: f64,
    current_value: f64,
    focused: bool,
    step: f64,
}

impl Slider {
    pub fn new(id: &str, label: &str, x: u16, y: u16, width: u16, min: f64, max: f64) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            position: (x, y),
            size: (width, 1), // Sliders are typically horizontal
            min_value: min,
            max_value: max,
            current_value: min,
            focused: false,
            step: (max - min) / (width as f64 - 4.0), // Account for the brackets and handle
        }
    }

    pub fn set_value(&mut self, value: f64) {
        self.current_value = value.clamp(self.min_value, self.max_value);
    }

    pub fn get_value(&self) -> f64 {
        self.current_value
    }

    /// Calculate the position of the slider handle based on the current value
    fn get_handle_position(&self) -> u16 {
        let ratio = (self.current_value - self.min_value) / (self.max_value - self.min_value);
        let position = (ratio * (self.size.0 - 4) as f64).round() as u16; // Account for [ and ] chars
        position + 1 // Offset for the opening bracket
    }

    /// Calculate the value based on the handle position
    fn get_value_from_position(&self, pos: u16) -> f64 {
        let clamped_pos = pos.saturating_sub(1).min(self.size.0 - 4); // Account for offset and brackets
        let ratio = clamped_pos as f64 / (self.size.0 - 4) as f64;
        self.min_value + ratio * (self.max_value - self.min_value)
    }
}

impl Component for Slider {
    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Mouse(MouseEvent { kind: crossterm::event::MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
                let (x, y) = self.position;
                let (w, h) = self.size;
                
                if *column >= x && *column < x + w && *row == y {
                    self.focused = true;
                    // Set the value based on where the user clicked
                    self.current_value = self.get_value_from_position(*column - x);
                    true
                } else {
                    self.focused = false;
                    false
                }
            }
            UIEvent::Mouse(MouseEvent { kind: crossterm::event::MouseEventKind::Drag(MouseButton::Left), column, row, .. }) => {
                let (x, y) = self.position;
                let (w, h) = self.size;
                
                if *column >= x && *column < x + w && *row == y {
                    // Update the value as the user drags
                    self.current_value = self.get_value_from_position(*column - x);
                    true
                } else {
                    false
                }
            }
            UIEvent::Key(KeyEvent { code, modifiers, .. }) => {
                if !self.focused {
                    return false;
                }
                
                match code {
                    KeyCode::Left => {
                        self.current_value = (self.current_value - self.step).clamp(self.min_value, self.max_value);
                        true
                    }
                    KeyCode::Right => {
                        self.current_value = (self.current_value + self.step).clamp(self.min_value, self.max_value);
                        true
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        self.current_value = (self.current_value + self.step).clamp(self.min_value, self.max_value);
                        true
                    }
                    KeyCode::Char('-') => {
                        self.current_value = (self.current_value - self.step).clamp(self.min_value, self.max_value);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn render(&mut self, frame: &mut Frame, _area: Rect) {
        let (x, y) = self.position;
        let (w, h) = self.size;
        
        let slider_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };
        
        // Create the slider string
        let handle_pos = self.get_handle_position();
        let mut slider_str = String::new();
        slider_str.push('[');
        
        for i in 1..(w - 1) {
            if i == handle_pos {
                slider_str.push('●'); // Slider handle
            } else {
                slider_str.push('─'); // Slider track
            }
        }
        
        slider_str.push(']');
        
        // Add the value display
        let value_display = format!(" {}: {:.2}", self.label, self.current_value);
        slider_str.push_str(&value_display);
        
        let slider_style = if self.focused {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        
        let slider = Paragraph::new(slider_str)
            .style(slider_style);
        
        frame.render_widget(slider, slider_area);
    }

    fn id(&self) -> &str {
        &self.id
    }
}