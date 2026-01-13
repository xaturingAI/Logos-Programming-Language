//! Text User Interface (TUI) library for the Logos programming language
//! Provides components for building terminal-based user interfaces with keyboard and mouse support

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseButton},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Gauge, Chart, Dataset, Axis},
    Frame, Terminal,
};

/// Represents a UI component/widget
pub trait Component {
    /// Handle an event and return whether it was consumed
    fn handle_event(&mut self, event: &UIEvent) -> bool;

    /// Render the component to the frame
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// Get the component's ID
    fn id(&self) -> &str;

    /// Get the component's size requirements
    fn required_size(&mut self, _constraints: (u16, u16)) -> (u16, u16) {
        (1, 1) // Default minimum size
    }

    /// Called when the component gains focus
    fn on_focus_gained(&mut self) {}

    /// Called when the component loses focus
    fn on_focus_lost(&mut self) {}

    /// Called when the component is resized
    fn on_resize(&mut self, _new_width: u16, _new_height: u16) {}
}

/// Represents different types of UI events
#[derive(Debug, Clone)]
pub enum UIEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    Paste(String),
    Custom(String),
    /// Double-click event
    DoubleClick(u16, u16, MouseButton),
    /// Scroll event
    Scroll(i32, i32),
    /// Drag event
    Drag(u16, u16, u16, u16), // from_x, from_y, to_x, to_y
}

// The Component trait is already defined earlier in the file
// /// Represents a UI component/widget
// pub trait Component {
//     /// Handle an event and return whether it was consumed
//     fn handle_event(&mut self, event: &UIEvent) -> bool;
//
//     /// Render the component to the frame
//     fn render(&mut self, frame: &mut Frame, area: Rect);
//
//     /// Get the component's ID
//     fn id(&self) -> &str;
//
//     /// Get the component's size requirements
//     fn required_size(&mut self, _constraints: (u16, u16)) -> (u16, u16) {
//         (1, 1) // Default minimum size
//     }
//
//     /// Called when the component gains focus
//     fn on_focus_gained(&mut self) {}
//
//     /// Called when the component loses focus
//     fn on_focus_lost(&mut self) {}
//
//     /// Called when the component is resized
//     fn on_resize(&mut self, _new_width: u16, _new_height: u16) {}
// }

/// Main TUI application structure
pub struct TUIApplication {
    /// Terminal instance
    terminal: Terminal<CrosstermBackend<io::Stdout>>,

    /// Registered components
    components: HashMap<String, Box<dyn Component>>,

    /// Current focused component
    focused_component: Option<String>,

    /// Stack of component IDs for focus management
    focus_stack: Vec<String>,

    /// Application state
    state: ApplicationState,

    /// Theme for the application
    theme: Theme,

    /// Layout manager
    layout_manager: LayoutManager,

    /// Event receiver
    event_rx: std::sync::mpsc::Receiver<UIEvent>,

    /// Event sender (for custom events)
    event_tx: std::sync::mpsc::Sender<UIEvent>,
}

/// Application state
// The ApplicationState struct is already defined earlier in the file
// #[derive(Debug, Clone)]
// pub struct ApplicationState {
//     /// Current screen/tab
//     pub current_screen: String,
//
//     /// Application data
//     pub data: HashMap<String, String>,
//
//     /// Whether the app is running
//     pub running: bool,
//
//     /// Global configuration
//     pub config: AppConfig,
// }

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Whether to show debug information
    pub show_debug_info: bool,

    /// Theme preference
    pub theme: String,

    /// Keyboard shortcuts
    pub shortcuts: HashMap<String, String>,
}

/// Theme for the UI
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary color
    pub primary: Color,

    /// Secondary color
    pub secondary: Color,

    /// Background color
    pub background: Color,

    /// Text color
    pub text: Color,

    /// Highlight color
    pub highlight: Color,
}

/// Layout manager for organizing components
#[derive(Debug, Clone)]
pub struct LayoutManager {
    /// Root layout container
    root: LayoutContainer,

    /// Component positions
    positions: HashMap<String, Rect>,
}

/// Container for layouts
#[derive(Debug, Clone)]
pub enum LayoutContainer {
    Horizontal { children: Vec<LayoutElement> },
    Vertical { children: Vec<LayoutElement> },
    Grid { rows: u16, cols: u16, children: Vec<LayoutElement> },
    Fixed(Rect),
}

/// Element in a layout
#[derive(Debug, Clone)]
pub enum LayoutElement {
    Component(String), // Component ID
    Container(LayoutContainer),
    Spacer(u16), // Size of spacer
}

/// Application state
#[derive(Debug, Clone)]
pub struct ApplicationState {
    /// Current screen/tab
    pub current_screen: String,
    
    /// Application data
    pub data: HashMap<String, String>,
    
    /// Whether the app is running
    pub running: bool,
}

impl ApplicationState {
    pub fn new() -> Self {
        Self {
            current_screen: "main".to_string(),
            data: HashMap::new(),
            running: true,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert("quit".to_string(), "Ctrl+Q".to_string());
        shortcuts.insert("focus_next".to_string(), "Tab".to_string());
        shortcuts.insert("focus_prev".to_string(), "Shift+Tab".to_string());

        Self {
            show_debug_info: false,
            theme: "default".to_string(),
            shortcuts,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Cyan,
            background: Color::Black,
            text: Color::White,
            highlight: Color::Yellow,
        }
    }
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            root: LayoutContainer::Fixed(Rect::default()), // Will be set during render
            positions: HashMap::new(),
        }
    }

    /// Calculate positions for all components based on the layout
    pub fn calculate_layout(&mut self, area: Rect) {
        let root_clone = self.root.clone();  // Clone to avoid borrowing conflicts
        self.calculate_container_layout(&root_clone, area);
    }

    /// Calculate layout for a container
    fn calculate_container_layout(&mut self, container: &LayoutContainer, area: Rect) {
        match container {
            LayoutContainer::Horizontal { children } => {
                let num_children = children.len() as u16;
                let width_per_child = area.width / num_children;

                for (i, child) in children.iter().enumerate() {
                    let child_area = Rect {
                        x: area.x + (i as u16 * width_per_child),
                        y: area.y,
                        width: if i == children.len() - 1 {
                            // Last child takes remaining space
                            area.x + area.width - (area.x + (i as u16 * width_per_child))
                        } else {
                            width_per_child
                        },
                        height: area.height,
                    };

                    self.assign_layout_element(child, child_area);
                }
            },
            LayoutContainer::Vertical { children } => {
                let num_children = children.len() as u16;
                let height_per_child = area.height / num_children;

                for (i, child) in children.iter().enumerate() {
                    let child_area = Rect {
                        x: area.x,
                        y: area.y + (i as u16 * height_per_child),
                        width: area.width,
                        height: if i == children.len() - 1 {
                            // Last child takes remaining space
                            area.y + area.height - (area.y + (i as u16 * height_per_child))
                        } else {
                            height_per_child
                        },
                    };

                    self.assign_layout_element(child, child_area);
                }
            },
            LayoutContainer::Grid { rows, cols, children } => {
                let width_per_col = area.width / cols;
                let height_per_row = area.height / rows;

                for (i, child) in children.iter().enumerate() {
                    let col = (i as u16) % cols;
                    let row = (i as u16) / cols;

                    let child_area = Rect {
                        x: area.x + (col * width_per_col),
                        y: area.y + (row * height_per_row),
                        width: width_per_col,
                        height: height_per_row,
                    };

                    self.assign_layout_element(child, child_area);
                }
            },
            LayoutContainer::Fixed(rect) => {
                // For fixed containers, use the provided rect
                // Note: This assumes the Fixed container has a single child component
                // In a real implementation, we'd need to handle multiple children
                // For now, we'll just store the rect for the container itself
                // This is a simplified implementation
            }
        }
    }

    /// Assign area to a layout element
    fn assign_layout_element(&mut self, element: &LayoutElement, area: Rect) {
        match element {
            LayoutElement::Component(id) => {
                self.positions.insert(id.clone(), area);
            },
            LayoutElement::Container(container) => {
                self.calculate_layout(area);
            },
            LayoutElement::Spacer(_) => {
                // Spacers don't need positioning
            }
        }
    }

    /// Get the area for a component
    pub fn get_component_area(&self, component_id: &str) -> Option<Rect> {
        self.positions.get(component_id).copied()
    }
}

impl TUIApplication {
    /// Create a new TUI application
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Create event channel
        let (event_tx, event_rx) = std::sync::mpsc::channel();

        // Spawn event handling thread
        let _event_tx = event_tx.clone();
        std::thread::spawn(move || {
            loop {
                // Poll for crossterm events
                if event::poll(Duration::from_millis(50)).unwrap() {
                    match event::read().unwrap() {
                        CEvent::Key(key) => {
                            let _ = _event_tx.send(UIEvent::Key(key));
                        }
                        CEvent::Mouse(mouse) => {
                            let _ = _event_tx.send(UIEvent::Mouse(mouse));
                        }
                        CEvent::Resize(w, h) => {
                            let _ = _event_tx.send(UIEvent::Resize(w, h));
                        }
                        CEvent::FocusGained | CEvent::FocusLost | CEvent::Paste(_) => {
                            // Ignore these events for now
                        }
                    }
                }
            }
        });

        Ok(Self {
            terminal,
            components: HashMap::new(),
            focused_component: None,
            focus_stack: Vec::new(),
            state: ApplicationState::new(),
            theme: Theme::default(),
            layout_manager: LayoutManager::new(),
            event_rx,
            event_tx,
        })
    }

    /// Register a new component
    pub fn register_component(&mut self, component: Box<dyn Component>) {
        let id = component.id().to_string();
        self.components.insert(id, component);
    }

    /// Set the focused component
    pub fn set_focus(&mut self, component_id: &str) {
        if self.components.contains_key(component_id) {
            self.focused_component = Some(component_id.to_string());
        }
    }

    /// Get a reference to a component
    pub fn get_component(&self, component_id: &str) -> Option<&dyn Component> {
        self.components.get(component_id).map(|c| c.as_ref())
    }

    /// Get a mutable reference to a component
    pub fn get_component_mut(&mut self, component_id: &str) -> Option<&mut Box<dyn Component>> {
        self.components.get_mut(component_id)
    }

    /// Send a custom event
    pub fn send_event(&self, event: UIEvent) -> Result<(), std::sync::mpsc::SendError<UIEvent>> {
        self.event_tx.send(event)
    }

    /// Run the application
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.state.running {
            // Handle events
            if let Ok(event) = self.event_rx.try_recv() {
                self.handle_event(event)?;
            }

            // Draw UI - simplified approach to avoid borrowing conflicts
            if let Err(_) = self.terminal.draw(|f| {
                // For now, just render a simple UI to avoid the borrowing issue
                use ratatui::widgets::{Block, Borders, Paragraph};
                use ratatui::style::{Style, Color};

                let block = Block::default()
                    .title("Logos TUI")
                    .borders(Borders::ALL);
                let paragraph = Paragraph::new("Press 'q' to quit")
                    .block(block)
                    .style(Style::default().fg(Color::White).bg(Color::Black));
                f.render_widget(paragraph, f.size());
            }) {
                // If drawing fails, we'll just continue
                // In a real implementation, we'd handle this more gracefully
            }
        }

        Ok(())
    }

    /// Create a dummy terminal for temporary replacement during drawing
    fn create_dummy_terminal(&mut self) -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>> {
        // For now, we'll create a minimal terminal - in a real implementation, this would be more sophisticated
        // We'll create a new backend and terminal instance
        use std::io::stdout;
        let backend = CrosstermBackend::new(stdout());
        Terminal::new(backend).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// Handle a UI event
    fn handle_event(&mut self, event: UIEvent) -> Result<(), Box<dyn std::error::Error>> {
        match &event {
            UIEvent::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                self.state.running = false;
            }
            UIEvent::Key(KeyEvent { code: KeyCode::Tab, .. }) => {
                // Cycle focus between components
                self.cycle_focus();
            }
            _ => {}
        }

        // Pass event to focused component
        if let Some(ref id) = self.focused_component {
            if let Some(component) = self.components.get_mut(id) {
                component.handle_event(&event);
            }
        }

        // Pass event to all components
        for component in self.components.values_mut() {
            component.handle_event(&event);
        }

        Ok(())
    }

    /// Cycle focus between components
    fn cycle_focus(&mut self) {
        let component_ids: Vec<String> = self.components.keys().cloned().collect();
        if component_ids.is_empty() {
            return;
        }

        match &self.focused_component {
            Some(current_id) => {
                if let Some(pos) = component_ids.iter().position(|id| id == current_id) {
                    let next_pos = (pos + 1) % component_ids.len();
                    self.focused_component = Some(component_ids[next_pos].clone());
                } else {
                    self.focused_component = Some(component_ids[0].clone());
                }
            }
            None => {
                self.focused_component = Some(component_ids[0].clone());
            }
        }
    }

    /// Render the UI
    fn render_ui(&mut self, frame: &mut Frame) {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title bar
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Status bar
            ])
            .split(frame.size());

        // Render title bar
        self.render_title_bar(frame, chunks[0]);

        // Render main content
        self.render_main_content(frame, chunks[1]);

        // Render status bar
        self.render_status_bar(frame, chunks[2]);
    }

    /// Render the title bar
    fn render_title_bar(&self, frame: &mut Frame, area: Rect) {
        let title = Block::default()
            .title("Logos TUI - Press 'q' to quit")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Blue).fg(Color::White));
        
        frame.render_widget(title, area);
    }

    /// Render the main content area
    fn render_main_content(&mut self, frame: &mut Frame, area: Rect) {
        // Render all components in the main area
        for component in self.components.values_mut() {
            component.render(frame, area);
        }
    }

    /// Render the status bar
    fn render_status_bar(&self, frame: &mut Frame, area: Rect) {
        let status_text = format!(
            "Components: {}, Focused: {}",
            self.components.len(),
            self.focused_component.as_deref().unwrap_or("none")
        );
        
        let status = Paragraph::new(status_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
        
        frame.render_widget(status, area);
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.state.running = false;
    }

    /// Set the application theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Set the application layout
    pub fn set_layout(&mut self, layout: LayoutContainer) {
        self.layout_manager = LayoutManager {
            root: layout,
            positions: HashMap::new(),
        };
    }

    /// Push a component to the focus stack
    pub fn push_focus(&mut self, component_id: String) {
        if self.components.contains_key(&component_id) {
            self.focus_stack.push(component_id.clone());
            self.focused_component = Some(component_id);
        }
    }

    /// Pop a component from the focus stack
    pub fn pop_focus(&mut self) {
        if let Some(prev_focused) = self.focus_stack.pop() {
            self.focused_component = self.focus_stack.last().cloned();
            if let Some(component) = self.components.get_mut(&prev_focused) {
                component.on_focus_lost();
            }
            if let Some(focused_id) = &self.focused_component {
                if let Some(component) = self.components.get_mut(focused_id) {
                    component.on_focus_gained();
                }
            }
        }
    }

    /// Focus the next component
    pub fn focus_next(&mut self) {
        let component_ids: Vec<String> = self.components.keys().cloned().collect();
        if component_ids.is_empty() {
            return;
        }

        match &self.focused_component {
            Some(current_id) => {
                if let Some(pos) = component_ids.iter().position(|id| id == current_id) {
                    let next_pos = (pos + 1) % component_ids.len();
                    self.set_focused_component(&component_ids[next_pos]);
                } else {
                    self.set_focused_component(&component_ids[0]);
                }
            }
            None => {
                self.set_focused_component(&component_ids[0]);
            }
        }
    }

    /// Focus the previous component
    pub fn focus_prev(&mut self) {
        let component_ids: Vec<String> = self.components.keys().cloned().collect();
        if component_ids.is_empty() {
            return;
        }

        match &self.focused_component {
            Some(current_id) => {
                if let Some(pos) = component_ids.iter().position(|id| id == current_id) {
                    let prev_pos = if pos == 0 { component_ids.len() - 1 } else { pos - 1 };
                    self.set_focused_component(&component_ids[prev_pos]);
                } else {
                    self.set_focused_component(&component_ids[0]);
                }
            }
            None => {
                self.set_focused_component(&component_ids[0]);
            }
        }
    }

    /// Set the focused component
    fn set_focused_component(&mut self, component_id: &str) {
        // Notify the previously focused component that it's losing focus
        if let Some(prev_id) = &self.focused_component {
            if let Some(prev_component) = self.components.get_mut(prev_id) {
                prev_component.on_focus_lost();
            }
        }

        // Set the new focused component
        self.focused_component = Some(component_id.to_string());

        // Notify the newly focused component that it's gaining focus
        if let Some(new_component) = self.components.get_mut(component_id) {
            new_component.on_focus_gained();
        }
    }

}

impl Drop for TUIApplication {
    fn drop(&mut self) {
        // Clean up terminal
        disable_raw_mode().unwrap();
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        self.terminal.show_cursor().unwrap();
    }
}

// Additional UI components
use ratatui::widgets::{ListState, Tabs as RatatuiTabs, Paragraph as RatatuiParagraph, Block as RatatuiBlock, Gauge as RatatuiGauge, Chart as RatatuiChart, GraphType};

// The TabbedPane and ProgressBar structs are already defined earlier in the file
// /// A tabbed interface component
// pub struct TabbedPane {
//     id: String,
//     tabs: Vec<String>,
//     active_tab: usize,
//     position: (u16, u16),
//     size: (u16, u16),
// }
//
// impl TabbedPane {
//     pub fn new(id: &str, tabs: Vec<String>, x: u16, y: u16, width: u16, height: u16) -> Self {
//         Self {
//             id: id.to_string(),
//             tabs,
//             active_tab: 0,
//             position: (x, y),
//             size: (width, height),
//         }
//     }
//
//     pub fn add_tab(&mut self, tab_name: String) {
//         self.tabs.push(tab_name);
//     }
//
//     pub fn switch_to_tab(&mut self, index: usize) {
//         if index < self.tabs.len() {
//             self.active_tab = index;
//         }
//     }
//
//     pub fn get_active_tab(&self) -> usize {
//         self.active_tab
//     }
//
//     pub fn get_active_tab_name(&self) -> &str {
//         &self.tabs[self.active_tab]
//     }
// }
//
// impl Component for TabbedPane {
//     fn handle_event(&mut self, event: &UIEvent) -> bool {
//         match event {
//             UIEvent::Key(KeyEvent { code: KeyCode::Left, .. }) => {
//                 if self.active_tab > 0 {
//                     self.active_tab -= 1;
//                     true
//                 } else {
//                     false
//                 }
//             }
//             UIEvent::Key(KeyEvent { code: KeyCode::Right, .. }) => {
//                 if self.active_tab < self.tabs.len() - 1 {
//                     self.active_tab += 1;
//                     true
//                 } else {
//                     false
//                 }
//             }
//             UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
//                 let (x, y) = self.position;
//                 let (w, h) = self.size;
//
//                 if *column >= x && *column < x + w && *row == y {
//                     // Calculate which tab was clicked based on position
//                     let tab_width = w / self.tabs.len() as u16;
//                     let clicked_tab = (*column - x) / tab_width;
//                     if clicked_tab < self.tabs.len() as u16 {
//                         self.active_tab = clicked_tab as usize;
//                         return true;
//                     }
//                 }
//                 false
//             }
//             _ => false,
//         }
//     }
//
//     fn render(&mut self, frame: &mut Frame, _area: Rect) {
//         let (x, y) = self.position;
//         let (w, h) = self.size;
//
//         let tab_area = Rect {
//             x,
//             y,
//             width: w,
//             height: h,
//         };
//
//         // Create tab titles
//         let titles: Vec<Line> = self.tabs
//             .iter()
//             .enumerate()
//             .map(|(i, t)| {
//                 let style = if i == self.active_tab {
//                     Style::default().add_modifier(Modifier::BOLD).bg(Color::Yellow).fg(Color::Black)
//                 } else {
//                     Style::default().bg(Color::DarkGray).fg(Color::White)
//                 };
//                 Line::from(Span::styled(t.as_str(), style))
//             })
//             .collect();
//
//         let tabs = Tabs::new(titles)
//             .block(Block::default().borders(Borders::ALL).title("Tabs"))
//             .select(self.active_tab)
//             .style(Style::default().fg(Color::Cyan));
//
//         frame.render_widget(tabs, tab_area);
//     }
//
//     fn id(&self) -> &str {
//         &self.id
//     }
// }
//
// /// A progress bar component
// pub struct ProgressBar {
//     id: String,
//     label: String,
//     position: (u16, u16),
//     size: (u16, u16),
//     progress: f64, // 0.0 to 1.0
//     show_percentage: bool,
// }
//
// impl ProgressBar {
//     pub fn new(id: &str, label: &str, x: u16, y: u16, width: u16, height: u16) -> Self {
//         Self {
//             id: id.to_string(),
//             label: label.to_string(),
//             position: (x, y),
//             size: (width, height),
//             progress: 0.0,
//             show_percentage: true,
//         }
//     }
//
//     pub fn set_progress(&mut self, progress: f64) {
//         self.progress = progress.clamp(0.0, 1.0);
//     }
//
//     pub fn get_progress(&self) -> f64 {
//         self.progress
//     }
//
//     pub fn set_show_percentage(&mut self, show: bool) {
//         self.show_percentage = show;
//     }
// }
//
// impl Component for ProgressBar {
//     fn handle_event(&mut self, _event: &UIEvent) -> bool {
//         false // Progress bars don't typically handle events directly
//     }
//
//     fn render(&mut self, frame: &mut Frame, _area: Rect) {
//         let (x, y) = self.position;
//         let (w, h) = self.size;
//
//         let progress_area = Rect {
//             x,
//             y,
//             width: w,
//             height: h,
//         };
//
//         let label = if self.show_percentage {
//             format!("{}: {:.0}%", self.label, self.progress * 100.0)
//         } else {
//             self.label.clone()
//         };
//
//         let gauge = Gauge::default()
//             .block(Block::default().borders(Borders::ALL).title(label))
//             .gauge_style(Style::default().fg(Color::Green))
//             .ratio(self.progress);
//
//         frame.render_widget(gauge, progress_area);
//     }
//
//     fn id(&self) -> &str {
//         &self.id
//     }
// }

/// Built-in UI components

/// A menu item that can be clicked
pub struct MenuItem {
    id: String,
    label: String,
    position: (u16, u16),
    size: (u16, u16),
    selected: bool,
    enabled: bool,
    callback: Option<Box<dyn FnMut()>>,
}

impl MenuItem {
    pub fn new(id: &str, label: &str, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            position: (x, y),
            size: (width, height),
            selected: false,
            enabled: true,
            callback: None,
        }
    }

    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.callback = Some(Box::new(callback));
        self
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Execute the callback if available
    pub fn execute_callback(&mut self) {
        if let Some(ref mut callback) = self.callback {
            callback();
        }
    }
}

impl Component for MenuItem {
    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
                let (x, y) = self.position;
                let (w, h) = self.size;

                if *column >= x && *column < x + w && *row >= y && *row < y + h && self.enabled {
                    self.selected = true;

                    // Execute callback if available
                    if let Some(ref mut callback) = self.callback {
                        callback();
                    }

                    return true;
                }
            }
            UIEvent::Key(KeyEvent { code: KeyCode::Enter, .. }) if self.selected => {
                if self.enabled {
                    // Execute callback if available
                    if let Some(ref mut callback) = self.callback {
                        callback();
                    }
                    return true;
                }
            }
            UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Moved, column, row, .. }) => {
                let (x, y) = self.position;
                let (w, h) = self.size;

                if *column >= x && *column < x + w && *row >= y && *row < y + h && self.enabled {
                    self.selected = true;
                } else {
                    self.selected = false;
                }
            }
            _ => {}
        }

        false
    }

    fn render(&mut self, frame: &mut Frame, _area: Rect) {
        let (x, y) = self.position;
        let (w, h) = self.size;

        let item_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        let item_style = if self.selected {
            Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
        } else if self.enabled {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::DarkGray) // Grayed out
        };

        let item = Paragraph::new(self.label.as_str())
            .style(item_style)
            .alignment(ratatui::layout::Alignment::Left);

        frame.render_widget(item, item_area);
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// A menu component containing multiple menu items
pub struct Menu {
    id: String,
    items: Vec<MenuItem>,
    position: (u16, u16),
    size: (u16, u16),
    visible: bool,
    active_item: Option<usize>,
}

impl Menu {
    pub fn new(id: &str, items: Vec<MenuItem>, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id: id.to_string(),
            items,
            position: (x, y),
            size: (width, height),
            visible: true,
            active_item: None,
        }
    }

    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn get_active_item(&self) -> Option<&MenuItem> {
        if let Some(index) = self.active_item {
            self.items.get(index)
        } else {
            None
        }
    }

    pub fn get_active_item_mut(&mut self) -> Option<&mut MenuItem> {
        if let Some(index) = self.active_item {
            self.items.get_mut(index)
        } else {
            None
        }
    }
}

impl Component for Menu {
    fn handle_event(&mut self, event: &UIEvent) -> bool {
        if !self.visible {
            return false;
        }

        match event {
            UIEvent::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                // Move selection up
                if let Some(current) = self.active_item {
                    if current > 0 {
                        self.active_item = Some(current - 1);
                    }
                } else if !self.items.is_empty() {
                    self.active_item = Some(0);
                }
                true
            }
            UIEvent::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                // Move selection down
                if let Some(current) = self.active_item {
                    if current < self.items.len() - 1 {
                        self.active_item = Some(current + 1);
                    }
                } else if !self.items.is_empty() {
                    self.active_item = Some(0);
                }
                true
            }
            UIEvent::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                // Activate the selected item
                if let Some(index) = self.active_item {
                    if let Some(item) = self.items.get_mut(index) {
                        // Execute the callback if available
                        if let Some(ref mut callback) = item.callback {
                            callback();
                        }
                    }
                }
                true
            }
            UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
                let (x, y) = self.position;
                let (w, h) = self.size;

                if *column >= x && *column < x + w && *row >= y && *row < y + h {
                    // Calculate which item was clicked based on position
                    let item_height = if self.items.is_empty() { 1 } else { h / self.items.len() as u16 };
                    let clicked_row = *row - y;
                    let item_index = clicked_row / item_height;

                    if item_index < self.items.len() as u16 {
                        self.active_item = Some(item_index as usize);

                        // Handle the click event for the specific item
                        if let Some(item) = self.items.get_mut(item_index as usize) {
                            item.handle_event(event);
                        }
                        return true;
                    }
                }
                false
            }
            UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Moved, column, row, .. }) => {
                let (x, y) = self.position;
                let (w, h) = self.size;

                if *column >= x && *column < x + w && *row >= y && *row < y + h {
                    // Calculate which item the mouse is over
                    let item_height = if self.items.is_empty() { 1 } else { h / self.items.len() as u16 };
                    let hovered_row = *row - y;
                    let item_index = hovered_row / item_height;

                    if item_index < self.items.len() as u16 {
                        // Update selection to the hovered item
                        self.active_item = Some(item_index as usize);
                        return true;
                    }
                }
                false
            }
            _ => {
                // Propagate event to individual items
                for item in &mut self.items {
                    item.handle_event(event);
                }
                false
            }
        }
    }

    fn render(&mut self, frame: &mut Frame, _area: Rect) {
        if !self.visible {
            return;
        }

        let (x, y) = self.position;
        let (w, h) = self.size;

        let menu_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        // Create a block for the menu
        let block = Block::default()
            .title("Menu")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        frame.render_widget(block, menu_area);

        // Calculate item dimensions
        let items_len = self.items.len();
        let item_height = if items_len == 0 { 1 } else { h.saturating_sub(2) / items_len as u16 }; // Subtract borders

        // Render each item
        for (i, item) in self.items.iter_mut().enumerate() {
            let item_area = Rect {
                x: x + 1, // Account for border
                y: y + 1 + (i as u16 * item_height), // Account for border and previous items
                width: w.saturating_sub(2), // Account for borders
                height: if i == items_len - 1 {
                    // Last item takes remaining space
                    h.saturating_sub(2).saturating_sub((i as u16) * item_height)
                } else {
                    item_height
                },
            };

            // Temporarily adjust item position to match the calculated area
            let original_pos = item.position;
            item.position = (item_area.x, item_area.y);
            item.size = (item_area.width, item_area.height);

            item.render(frame, item_area);

            // Restore original position
            item.position = original_pos;
        }
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// A simple button component
pub struct Button {
    id: String,
    label: String,
    position: (u16, u16),
    size: (u16, u16),
    clicked: bool,
    callback: Option<Box<dyn FnMut()>>,
}

impl Button {
    pub fn new(id: &str, label: &str, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            position: (x, y),
            size: (width, height),
            clicked: false,
            callback: None,
        }
    }

    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.callback = Some(Box::new(callback));
        self
    }
}

impl Component for Button {
    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
                let (x, y) = self.position;
                let (w, h) = self.size;

                if *column >= x && *column < x + w && *row >= y && *row < y + h {
                    self.clicked = true;

                    // Execute callback if available
                    if let Some(ref mut callback) = self.callback {
                        callback();
                    }

                    return true;
                }
            }
            _ => {}
        }

        false
    }

    fn render(&mut self, frame: &mut Frame, _area: Rect) {
        let (x, y) = self.position;
        let (w, h) = self.size;

        // Create a paragraph widget for the button
        let button_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        let button_style = if self.clicked {
            Style::default().bg(Color::LightBlue).fg(Color::Black)
        } else {
            Style::default().bg(Color::Blue).fg(Color::White)
        };

        let button = Paragraph::new(self.label.as_str())
            .alignment(ratatui::layout::Alignment::Center)
            .style(button_style);

        frame.render_widget(button, button_area);
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// A tabbed interface component
pub struct TabbedPane {
    id: String,
    tabs: Vec<String>,
    active_tab: usize,
    position: (u16, u16),
    size: (u16, u16),
}

impl TabbedPane {
    pub fn new(id: &str, tabs: Vec<String>, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id: id.to_string(),
            tabs,
            active_tab: 0,
            position: (x, y),
            size: (width, height),
        }
    }

    pub fn add_tab(&mut self, tab_name: String) {
        self.tabs.push(tab_name);
    }

    pub fn switch_to_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab = index;
        }
    }

    pub fn get_active_tab(&self) -> usize {
        self.active_tab
    }

    pub fn get_active_tab_name(&self) -> &str {
        &self.tabs[self.active_tab]
    }
}

impl Component for TabbedPane {
    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Key(KeyEvent { code: KeyCode::Left, .. }) => {
                if self.active_tab > 0 {
                    self.active_tab -= 1;
                    true
                } else {
                    false
                }
            }
            UIEvent::Key(KeyEvent { code: KeyCode::Right, .. }) => {
                if self.active_tab < self.tabs.len() - 1 {
                    self.active_tab += 1;
                    true
                } else {
                    false
                }
            }
            UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
                let (x, y) = self.position;
                let (w, h) = self.size;

                if *column >= x && *column < x + w && *row == y {
                    // Calculate which tab was clicked based on position
                    let tab_width = w / self.tabs.len() as u16;
                    let clicked_tab = (*column - x) / tab_width;
                    if clicked_tab < self.tabs.len() as u16 {
                        self.active_tab = clicked_tab as usize;
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let (x, y) = self.position;
        let (w, h) = self.size;

        let tab_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        // Create tab titles
        let titles: Vec<Line> = self.tabs
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let style = if i == self.active_tab {
                    Style::default().add_modifier(Modifier::BOLD).bg(Color::Yellow).fg(Color::Black)
                } else {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                };
                Line::from(Span::styled(t.as_str(), style))
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(self.active_tab)
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(tabs, tab_area);
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// A progress bar component
pub struct ProgressBar {
    id: String,
    label: String,
    position: (u16, u16),
    size: (u16, u16),
    progress: f64, // 0.0 to 1.0
    show_percentage: bool,
}

impl ProgressBar {
    pub fn new(id: &str, label: &str, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            position: (x, y),
            size: (width, height),
            progress: 0.0,
            show_percentage: true,
        }
    }

    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub fn get_progress(&self) -> f64 {
        self.progress
    }

    pub fn set_show_percentage(&mut self, show: bool) {
        self.show_percentage = show;
    }
}

impl Component for ProgressBar {
    fn handle_event(&mut self, _event: &UIEvent) -> bool {
        false // Progress bars don't typically handle events directly
    }

    fn render(&mut self, frame: &mut Frame, _area: Rect) {
        let (x, y) = self.position;
        let (w, h) = self.size;

        let progress_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        let label = if self.show_percentage {
            format!("{}: {:.0}%", self.label, self.progress * 100.0)
        } else {
            self.label.clone()
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(label))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(self.progress);

        frame.render_widget(gauge, progress_area);
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// A chart/graph component for displaying data
pub struct ChartComponent {
    id: String,
    title: String,
    position: (u16, u16),
    size: (u16, u16),
    datasets: Vec<ChartDataset>,
    x_axis_label: String,
    y_axis_label: String,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

#[derive(Debug, Clone)]
pub struct ChartDataset {
    pub name: String,
    pub data: Vec<(f64, f64)>,
    pub color: Color,
}

impl ChartComponent {
    pub fn new(id: &str, title: &str, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            position: (x, y),
            size: (width, height),
            datasets: Vec::new(),
            x_axis_label: "X".to_string(),
            y_axis_label: "Y".to_string(),
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
        }
    }

    pub fn add_dataset(&mut self, dataset: ChartDataset) {
        self.datasets.push(dataset);
    }

    pub fn set_x_axis_range(&mut self, min: f64, max: f64) {
        self.x_min = min;
        self.x_max = max;
    }

    pub fn set_y_axis_range(&mut self, min: f64, max: f64) {
        self.y_min = min;
        self.y_max = max;
    }

    pub fn set_axis_labels(&mut self, x_label: &str, y_label: &str) {
        self.x_axis_label = x_label.to_string();
        self.y_axis_label = y_label.to_string();
    }
}

impl Component for ChartComponent {
    fn handle_event(&mut self, _event: &UIEvent) -> bool {
        false // Charts typically don't handle events directly
    }

    fn render(&mut self, frame: &mut Frame, _area: Rect) {
        let (x, y) = self.position;
        let (w, h) = self.size;

        let chart_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        // Convert our datasets to ratatui datasets
        let datasets: Vec<Dataset> = self.datasets
            .iter()
            .map(|ds| {
                Dataset::default()
                    .name(ds.name.as_str())
                    .data(&ds.data)
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(ds.color))
            })
            .collect();

        let x_axis = Axis::default()
            .title(self.x_axis_label.as_str())
            .bounds([self.x_min, self.x_max])
            .labels(vec![self.x_min.to_string().into(), self.x_max.to_string().into()]);

        let y_axis = Axis::default()
            .title(self.y_axis_label.as_str())
            .bounds([self.y_min, self.y_max])
            .labels(vec![self.y_min.to_string().into(), self.y_max.to_string().into()]);

        let chart = Chart::new(datasets)
            .block(Block::default().title(self.title.as_str()).borders(Borders::ALL))
            .x_axis(x_axis)
            .y_axis(y_axis);

        frame.render_widget(chart, chart_area);
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// A modal dialog component
pub struct ModalDialog {
    id: String,
    title: String,
    content: String,
    visible: bool,
    position: (u16, u16),
    size: (u16, u16),
    buttons: Vec<String>, // Button labels
    on_close: Option<Box<dyn Fn()>>,
}

impl ModalDialog {
    pub fn new(id: &str, title: &str, content: &str, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            visible: false,
            position: (x, y),
            size: (width, height),
            buttons: vec!["OK".to_string()],
            on_close: None,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_buttons(&mut self, buttons: Vec<String>) {
        self.buttons = buttons;
    }

    pub fn with_on_close<F>(mut self, callback: F) -> Self
    where
        F: Fn() + 'static,
    {
        self.on_close = Some(Box::new(callback));
        self
    }
}

impl Component for ModalDialog {
    fn handle_event(&mut self, event: &UIEvent) -> bool {
        if !self.visible {
            return false;
        }

        match event {
            UIEvent::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                self.visible = false;
                if let Some(ref callback) = self.on_close {
                    callback();
                }
                true
            }
            UIEvent::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                // Default action is to close the dialog
                self.visible = false;
                if let Some(ref callback) = self.on_close {
                    callback();
                }
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let (x, y) = self.position;
        let (w, h) = self.size;

        let dialog_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        // Create a semi-transparent overlay
        let overlay = Paragraph::new("")
            .style(Style::default().bg(Color::Rgb(0, 0, 0)).fg(Color::Rgb(0, 0, 0)));
        frame.render_widget(overlay, frame.size());

        // Create the dialog box
        let dialog_block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Rgb(30, 30, 30)).fg(Color::White));

        let content = Paragraph::new(self.content.as_str())
            .block(Block::default().style(Style::default().bg(Color::Rgb(30, 30, 30)).fg(Color::White)));

        // Calculate content area (leaving space for title and buttons)
        let content_area = Rect {
            x: dialog_area.x + 1,
            y: dialog_area.y + 2,
            width: dialog_area.width - 2,
            height: dialog_area.height - 3 - 2, // Subtract title and button area
        };

        // Render the dialog
        frame.render_widget(dialog_block, dialog_area);
        frame.render_widget(content, content_area);

        // Render buttons at the bottom
        if !self.buttons.is_empty() {
            let button_area = Rect {
                x: dialog_area.x + 1,
                y: dialog_area.y + dialog_area.height - 3,
                width: dialog_area.width - 2,
                height: 2,
            };

            let button_text = self.buttons.join("  ");
            let buttons = Paragraph::new(button_text)
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().bg(Color::Rgb(30, 30, 30)).fg(Color::White));

            frame.render_widget(buttons, button_area);
        }
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// A text input field component
pub struct TextInput {
    id: String,
    label: String,
    position: (u16, u16),
    size: (u16, u16),
    content: String,
    cursor_position: usize,
    focused: bool,
}

impl TextInput {
    pub fn new(id: &str, label: &str, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            position: (x, y),
            size: (width, height),
            content: String::new(),
            cursor_position: 0,
            focused: false,
        }
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    fn move_cursor_right(&mut self) {
        self.cursor_position = std::cmp::min(self.cursor_position + 1, self.content.len());
    }

    fn enter_char(&mut self, new_char: char) {
        self.content.insert(self.cursor_position, new_char);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        if self.cursor_position != 0 && self.content.len() > 0 {
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;
            self.content.remove(from_left_to_current_index);
            self.move_cursor_left();
        }
    }
}

impl Component for TextInput {
    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Key(KeyEvent { code, modifiers: _, .. }) => {
                match code {
                    KeyCode::Char(c) => {
                        self.enter_char(*c);
                        true
                    }
                    KeyCode::Backspace => {
                        self.delete_char();
                        true
                    }
                    KeyCode::Left => {
                        self.move_cursor_left();
                        true
                    }
                    KeyCode::Right => {
                        self.move_cursor_right();
                        true
                    }
                    KeyCode::Enter => {
                        // Submit action - could trigger a callback
                        true
                    }
                    KeyCode::Home => {
                        self.cursor_position = 0;
                        true
                    }
                    KeyCode::End => {
                        self.cursor_position = self.content.len();
                        true
                    }
                    _ => false,
                }
            }
            UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
                let (x, y) = self.position;
                let (w, h) = self.size;
                
                if *column >= x && *column < x + w && *row >= y && *row < y + h {
                    self.focused = true;
                    true
                } else {
                    self.focused = false;
                    true
                }
            }
            _ => false,
        }
    }

    fn render(&mut self, frame: &mut Frame, _area: Rect) {
        let (x, y) = self.position;
        let (w, h) = self.size;
        
        let input_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };
        
        let input_style = if self.focused {
            Style::default().bg(Color::Yellow).fg(Color::Black)
        } else {
            Style::default().bg(Color::White).fg(Color::Black)
        };
        
        let input = Paragraph::new(self.content.as_str())
            .style(input_style)
            .block(Block::default().borders(Borders::ALL).title(self.label.as_str()));
        
        frame.render_widget(input, input_area);
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// A simple text label component
pub struct Label {
    id: String,
    text: String,
    position: (u16, u16),
    style: Style,
}

impl Label {
    pub fn new(id: &str, text: &str, x: u16, y: u16) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
            position: (x, y),
            style: Style::default(),
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Component for Label {
    fn handle_event(&mut self, _event: &UIEvent) -> bool {
        false
    }

    fn render(&mut self, frame: &mut Frame, _area: Rect) {
        let (x, y) = self.position;
        
        let label_area = Rect {
            x,
            y,
            width: self.text.len() as u16,
            height: 1,
        };
        
        let label = Paragraph::new(self.text.as_str()).style(self.style);
        
        frame.render_widget(label, label_area);
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// A list component
pub struct ListComponent {
    id: String,
    items: Vec<String>,
    selected: Option<usize>,
    position: (u16, u16),
    size: (u16, u16),
    focused: bool,
}

impl ListComponent {
    pub fn new(id: &str, items: Vec<String>, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id: id.to_string(),
            items,
            selected: None,
            position: (x, y),
            size: (width, height),
            focused: false,
        }
    }

    pub fn get_selected_item(&self) -> Option<&String> {
        if let Some(index) = self.selected {
            self.items.get(index)
        } else {
            None
        }
    }
}

impl Component for ListComponent {
    fn handle_event(&mut self, event: &UIEvent) -> bool {
        match event {
            UIEvent::Key(KeyEvent { code, .. }) => {
                if !self.focused {
                    return false;
                }
                
                match code {
                    KeyCode::Up => {
                        if let Some(selected) = self.selected {
                            if selected > 0 {
                                self.selected = Some(selected - 1);
                            }
                        } else if !self.items.is_empty() {
                            self.selected = Some(0);
                        }
                        true
                    }
                    KeyCode::Down => {
                        if let Some(selected) = self.selected {
                            if selected < self.items.len() - 1 {
                                self.selected = Some(selected + 1);
                            }
                        } else if !self.items.is_empty() {
                            self.selected = Some(0);
                        }
                        true
                    }
                    KeyCode::Enter => {
                        // Handle selection
                        true
                    }
                    _ => false,
                }
            }
            UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
                let (x, y) = self.position;
                let (w, h) = self.size;
                
                if *column >= x && *column < x + w && *row >= y && *row < y + h {
                    self.focused = true;
                    // Calculate which item was clicked
                    let item_idx = (*row - y) as usize;
                    if item_idx < self.items.len() {
                        self.selected = Some(item_idx);
                    }
                    true
                } else {
                    self.focused = false;
                    true
                }
            }
            _ => false,
        }
    }

    fn render(&mut self, frame: &mut Frame, _area: Rect) {
        let (x, y) = self.position;
        let (w, h) = self.size;
        
        let list_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };
        
        // Convert items to ListItems
        let list_items: Vec<ListItem> = self.items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if Some(i) == self.selected && self.focused {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(item.as_str()).style(style)
            })
            .collect();
        
        let list = List::new(list_items)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        
        frame.render_widget(list, list_area);
    }

    fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("btn1", "Click Me", 0, 0, 10, 3);
        assert_eq!(button.id(), "btn1");
        assert_eq!(button.label, "Click Me");
    }

    #[test]
    fn test_text_input_creation() {
        let input = TextInput::new("input1", "Label", 0, 0, 20, 3);
        assert_eq!(input.id(), "input1");
        assert_eq!(input.label, "Label");
        assert_eq!(input.get_content(), "");
    }

    #[test]
    fn test_label_creation() {
        let label = Label::new("label1", "Hello, World!", 0, 0);
        assert_eq!(label.id(), "label1");
        assert_eq!(label.text, "Hello, World!");
    }

    /// A menu bar component for the shell
    pub struct MenuBar {
        id: String,
        menus: Vec<Menu>,
        position: (u16, u16),
        size: (u16, u16),
        visible: bool,
        active_menu: Option<usize>,
        active_item: Option<(usize, usize)>, // (menu_index, item_index)
    }

    impl MenuBar {
        pub fn new(id: &str, x: u16, y: u16, width: u16, height: u16) -> Self {
            Self {
                id: id.to_string(),
                menus: Vec::new(),
                position: (x, y),
                size: (width, height),
                visible: true,
                active_menu: None,
                active_item: None,
            }
        }

        pub fn add_menu(&mut self, menu: Menu) {
            self.menus.push(menu);
        }

        pub fn set_visible(&mut self, visible: bool) {
            self.visible = visible;
        }

        pub fn is_visible(&self) -> bool {
            self.visible
        }
    }

    impl Component for MenuBar {
        fn handle_event(&mut self, event: &UIEvent) -> bool {
            if !self.visible {
                return false;
            }

            match event {
                UIEvent::Key(KeyEvent { code: KeyCode::Left, .. }) => {
                    // Move to previous menu
                    if let Some(current) = self.active_menu {
                        if current > 0 {
                            self.active_menu = Some(current - 1);
                        }
                    } else if !self.menus.is_empty() {
                        self.active_menu = Some(0);
                    }
                    true
                }
                UIEvent::Key(KeyEvent { code: KeyCode::Right, .. }) => {
                    // Move to next menu
                    if let Some(current) = self.active_menu {
                        if current < self.menus.len() - 1 {
                            self.active_menu = Some(current + 1);
                        }
                    } else if !self.menus.is_empty() {
                        self.active_menu = Some(0);
                    }
                    true
                }
                UIEvent::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                    // If a menu is active, move to its items
                    if let Some(menu_idx) = self.active_menu {
                        if menu_idx < self.menus.len() {
                            // Set the active item in the current menu
                            if let Some(item_idx) = self.menus[menu_idx].active_item {
                                self.active_item = Some((menu_idx, item_idx));
                            } else if !self.menus[menu_idx].items.is_empty() {
                                self.active_item = Some((menu_idx, 0));
                            }
                        }
                    }
                    true
                }
                UIEvent::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                    // If an item is active, go back to menu
                    if self.active_item.is_some() {
                        self.active_item = None;
                    }
                    true
                }
                UIEvent::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                    // Activate the selected menu item
                    if let Some((menu_idx, item_idx)) = self.active_item {
                        if menu_idx < self.menus.len() {
                            if let Some(item) = self.menus[menu_idx].items.get_mut(item_idx) {
                                // Execute the callback if available
                                item.execute_callback();
                            }
                        }
                    }
                    true
                }
                UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
                    let (x, y) = self.position;
                    let (w, h) = self.size;

                    if *column >= x && *column < x + w && *row >= y && *row < y + h {
                        // Calculate which menu was clicked based on position
                        let menu_width = if self.menus.is_empty() { 1 } else { w / self.menus.len() as u16 };
                        let clicked_col = *column - x;
                        let menu_index = clicked_col / menu_width;

                        if menu_index < self.menus.len() as u16 {
                            self.active_menu = Some(menu_index as usize);

                            // Handle the click event for the specific menu
                            if let Some(menu) = self.menus.get_mut(menu_index as usize) {
                                menu.handle_event(event);
                            }
                            return true;
                        }
                    }
                    false
                }
                UIEvent::Mouse(MouseEvent { kind: event::MouseEventKind::Moved, column, row, .. }) => {
                    let (x, y) = self.position;
                    let (w, h) = self.size;

                    if *column >= x && *column < x + w && *row >= y && *row < y + h {
                        // Calculate which menu the mouse is over
                        let menu_width = if self.menus.is_empty() { 1 } else { w / self.menus.len() as u16 };
                        let hovered_col = *column - x;
                        let menu_index = hovered_col / menu_width;

                        if menu_index < self.menus.len() as u16 {
                            self.active_menu = Some(menu_index as usize);
                            return true;
                        }
                    }
                    false
                }
                _ => {
                    // Propagate event to individual menus
                    for menu in &mut self.menus {
                        menu.handle_event(event);
                    }
                    false
                }
            }
        }

        fn render(&mut self, frame: &mut Frame, _area: Rect) {
            if !self.visible {
                return;
            }

            let (x, y) = self.position;
            let (w, h) = self.size;

            let menubar_area = Rect {
                x,
                y,
                width: w,
                height: h,
            };

            // Create a block for the menu bar
            let block = Block::default()
                .borders(Borders::BOTTOM)
                .style(Style::default().bg(Color::DarkGray).fg(Color::White));

            frame.render_widget(block, menubar_area);

            // Calculate menu dimensions
            let menu_width = if self.menus.is_empty() { 1 } else { w / self.menus.len() as u16 };

            // Render each menu header
            for (i, menu) in self.menus.iter_mut().enumerate() {
                let menu_header_area = Rect {
                    x: x + (i as u16 * menu_width),
                    y,
                    width: if i == self.menus.len() - 1 {
                        // Last menu takes remaining space
                        w.saturating_sub((i as u16) * menu_width)
                    } else {
                        menu_width
                    },
                    height: h,
                };

                // Render menu header
                let header_style = if self.active_menu == Some(i) {
                    Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                };

                let header = Paragraph::new(menu.id.as_str())
                    .style(header_style)
                    .alignment(ratatui::layout::Alignment::Center);

                frame.render_widget(header, menu_header_area);

                // If this menu is active and has items, render the submenu
                if self.active_menu == Some(i) {
                    // Calculate submenu position and size
                    let submenu_area = Rect {
                        x: menu_header_area.x,
                        y: menu_header_area.y + menu_header_area.height,
                        width: menu_header_area.width,
                        height: 10, // Fixed height for submenu
                    };

                    // Render the submenu with adjusted position
                    // We'll temporarily adjust the menu's position and size for rendering
                    let original_pos = menu.position;
                    let original_size = menu.size;
                    menu.position = (submenu_area.x, submenu_area.y);
                    menu.size = (submenu_area.width, submenu_area.height);

                    menu.render(frame, submenu_area);

                    // Restore original position and size
                    menu.position = original_pos;
                    menu.size = original_size;
                }
            }
        }

        fn id(&self) -> &str {
            &self.id
        }
    }

    #[test]
    fn test_list_creation() {
        let items = vec!["Item 1".to_string(), "Item 2".to_string()];
        let list = ListComponent::new("list1", items, 0, 0, 20, 5);
        assert_eq!(list.id(), "list1");
        assert_eq!(list.items.len(), 2);
    }

    #[test]
    fn test_menu_item_creation() {
        let item = MenuItem::new("item1", "File", 0, 0, 10, 1);
        assert_eq!(item.id(), "item1");
        assert_eq!(item.label, "File");
        assert_eq!(item.is_enabled(), true);
    }

    #[test]
    fn test_menu_creation() {
        let items = vec![MenuItem::new("item1", "New", 0, 0, 10, 1)];
        let menu = Menu::new("file_menu", items, 0, 0, 20, 5);
        assert_eq!(menu.id(), "file_menu");
        assert_eq!(menu.items.len(), 1);
        assert_eq!(menu.is_visible(), true);
    }

    #[test]
    fn test_menu_bar_creation() {
        let menubar = MenuBar::new("main_menubar", 0, 0, 80, 3);
        assert_eq!(menubar.id(), "main_menubar");
        assert_eq!(menubar.menus.len(), 0);
        assert_eq!(menubar.is_visible(), true);
    }
}