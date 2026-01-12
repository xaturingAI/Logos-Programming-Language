//! Shell interface for the Logos programming language
//! Provides an interactive environment for writing and executing Logos code

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Direction, Alignment, Rect},
    style::{Style, Color, Modifier},
    widgets::{Block, Borders, BorderType, Paragraph, Wrap, List, ListItem},
    text::{Span, Line, Text},
    Frame, Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseButton},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, stdout};
use std::fs;
use std::path::Path;

use crate::ast::*;
use crate::parser::Parser;
use crate::type_checker::TypeChecker;
use crate::runtime::Runtime;

/// Represents the state of the Logos shell
pub struct ShellState {
    /// Lines of code in the editor
    lines: Vec<String>,
    /// Current cursor position (row, col)
    cursor_position: (u16, u16),
    /// History of executed commands
    history: Vec<String>,
    /// Output of executed commands
    output: Vec<String>,
    /// Scroll offset for the editor view
    scroll_offset: u16,
    /// Current mode (editing, command, or menu)
    mode: ShellMode,
    /// Current file path (if any)
    current_file: Option<String>,
    /// Menu visibility
    menu_visible: bool,
    /// Selected menu item
    selected_menu_item: MenuItem,
    /// File browser state
    file_browser_state: FileBrowserState,
    /// Status message
    status_message: String,
}

/// Different modes for the shell
#[derive(Debug, Clone, PartialEq)]
enum ShellMode {
    Editing,
    Command,
    Insert,
    Menu,
    FileBrowser,
}

/// Menu items
#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuItem {
    File,
    Edit,
    View,
    Run,
    Help,
}

/// File browser state
#[derive(Debug, Clone)]
struct FileBrowserState {
    /// Current directory
    current_dir: String,
    /// Files in current directory
    files: Vec<String>,
    /// Selected file index
    selected_index: usize,
    /// Visible files start index
    visible_start: usize,
}

impl ShellState {
    /// Create a new shell state
    pub fn new() -> Self {
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        Self {
            lines: vec!["// Welcome to the Logos shell!".to_string(), "// Press Ctrl+N for new file, Ctrl+O to open, Ctrl+S to save".to_string()],
            cursor_position: (0, 0),
            history: Vec::new(),
            output: vec!["Welcome to the Logos shell! Press Ctrl+H for help.".to_string()],
            scroll_offset: 0,
            mode: ShellMode::Editing,
            current_file: None,
            menu_visible: true,
            selected_menu_item: MenuItem::File,
            file_browser_state: FileBrowserState {
                current_dir,
                files: Self::get_files_in_directory(&std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))),
                selected_index: 0,
                visible_start: 0,
            },
            status_message: "Ready".to_string(),
        }
    }

    /// Get files in a directory
    fn get_files_in_directory(dir: &std::path::Path) -> Vec<String> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".logos") || file_name.ends_with(".rs") ||
                               file_name.ends_with(".py") || file_name.ends_with(".js") ||
                               file_name.ends_with(".go") || file_name.ends_with(".java") ||
                               file_name.ends_with(".md") || file_name.ends_with(".txt") {
                                files.push(file_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        files.sort();
        files
    }

    /// Handle character input
    pub fn handle_char_input(&mut self, ch: char) {
        if self.mode == ShellMode::Insert {
            // Insert character at cursor position
            let (row, col) = self.cursor_position;
            if (row as usize) < self.lines.len() {
                let line = &mut self.lines[row as usize];
                let col = col.min(line.len() as u16);
                line.insert(col as usize, ch);
                self.cursor_position.1 = col + 1;
            } else {
                // If cursor is past the end, add a new line
                self.lines.push(ch.to_string());
                self.cursor_position.0 += 1;
                self.cursor_position.1 = 1;
            }
        }
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        if self.mode == ShellMode::Insert {
            let (row, col) = self.cursor_position;
            if row < self.lines.len() as u16 && col > 0 {
                let line = &mut self.lines[row as usize];
                if col as usize <= line.len() {
                    line.remove((col - 1) as usize);
                    self.cursor_position.1 -= 1;
                }
            }
        }
    }

    /// Handle left arrow key
    pub fn move_cursor_left(&mut self) {
        let (row, col) = self.cursor_position;
        if col > 0 {
            self.cursor_position.1 = col - 1;
        } else if row > 0 {
            // Move to end of previous line
            let prev_row = (row - 1) as usize;
            if prev_row < self.lines.len() {
                let new_col = self.lines[prev_row].len() as u16;
                self.cursor_position = (row - 1, new_col);
            }
        }
    }

    /// Handle right arrow key
    pub fn move_cursor_right(&mut self) {
        let (row, col) = self.cursor_position;
        if (row as usize) < self.lines.len() {
            let line_len = self.lines[row as usize].len() as u16;
            if col < line_len {
                self.cursor_position.1 = col + 1;
            } else if ((row + 1) as usize) < self.lines.len() {
                // Move to beginning of next line
                self.cursor_position = (row + 1, 0);
            }
        }
    }

    /// Handle up arrow key
    pub fn move_cursor_up(&mut self) {
        let (row, col) = self.cursor_position;
        if row > 0 {
            self.cursor_position.0 = row - 1;
            // Adjust column to not exceed line length
            let new_row = (row - 1) as usize;
            if new_row < self.lines.len() {
                let line_len = self.lines[new_row].len() as u16;
                self.cursor_position.1 = col.min(line_len);
            }
        }
    }

    /// Handle down arrow key
    pub fn move_cursor_down(&mut self) {
        let (row, col) = self.cursor_position;
        if ((row + 1) as usize) < self.lines.len() {
            self.cursor_position.0 = row + 1;
            // Adjust column to not exceed line length
            let new_row = (row + 1) as usize;
            if new_row < self.lines.len() {
                let line_len = self.lines[new_row].len() as u16;
                self.cursor_position.1 = col.min(line_len);
            }
        }
    }

    /// Handle enter key
    pub fn handle_enter(&mut self) {
        if self.mode == ShellMode::Insert {
            let (row, col) = self.cursor_position;
            if row as usize <= self.lines.len() {
                // Split the current line at the cursor position
                let current_line = self.lines.get(row as usize).cloned().unwrap_or_default();
                let (left, right) = current_line.split_at(col as usize);
                self.lines.insert(row as usize + 1, right.to_string());
                self.lines[row as usize] = left.to_string();

                // Move cursor to the beginning of the new line
                self.cursor_position = (row + 1, 0);
            }
        }
    }

    /// Toggle insert mode
    pub fn toggle_insert_mode(&mut self) {
        self.mode = match self.mode {
            ShellMode::Editing => ShellMode::Insert,
            ShellMode::Insert => ShellMode::Editing,
            ShellMode::Command => ShellMode::Insert,
            ShellMode::Menu => ShellMode::Insert,
            ShellMode::FileBrowser => ShellMode::Insert,
        };
    }

    /// Toggle command mode
    pub fn toggle_command_mode(&mut self) {
        self.mode = ShellMode::Command;
    }





    /// Execute the current input
    pub fn execute_input(&mut self) {
        // For now, just clear the output when executing
        self.output.clear();
        self.output.push("Executing code...".to_string());

        // Combine all lines into a single string for processing
        let code = self.lines.join("\n");

        // Add to history
        self.history.push(code.clone());

        // Process the code
        let result = self.process_command(&code);

        // Add result to output
        self.output.push(format!("> {}", code.replace('\n', "\\n"))); // Show newlines as \n
        self.output.extend(result);

        // Return to editing mode
        self.mode = ShellMode::Editing;
    }

    /// Handle key event with modifiers
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match (key.code, key.modifiers) {
            // Ctrl+N: New file
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                self.new_file();
                true
            },
            // Ctrl+O: Open file
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                self.mode = ShellMode::FileBrowser;
                true
            },
            // Ctrl+S: Save file
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.save_file();
                true
            },
            // Ctrl+W: Close file
            (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                self.close_file();
                true
            },
            // Ctrl+E: Exit
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                // This would normally trigger an exit, but we'll just return true to indicate exit intent
                true
            },
            // Ctrl+H: Show help
            (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                self.show_help();
                true
            },
            // Arrow keys for menu navigation when in menu mode
            (KeyCode::Left, _) if self.mode == ShellMode::Menu => {
                self.select_prev_menu_item();
                true
            },
            (KeyCode::Right, _) if self.mode == ShellMode::Menu => {
                self.select_next_menu_item();
                true
            },
            // Enter to activate menu item
            (KeyCode::Enter, _) if self.mode == ShellMode::Menu => {
                self.activate_menu_item();
                true
            },
            // Up/down arrows for file browser when in file browser mode
            (KeyCode::Up, _) if self.mode == ShellMode::FileBrowser => {
                self.select_prev_file();
                true
            },
            (KeyCode::Down, _) if self.mode == ShellMode::FileBrowser => {
                self.select_next_file();
                true
            },
            // Enter to open selected file in file browser mode
            (KeyCode::Enter, _) if self.mode == ShellMode::FileBrowser => {
                self.open_selected_file();
                true
            },
            // Escape to exit file browser mode
            (KeyCode::Esc, _) if self.mode == ShellMode::FileBrowser => {
                self.mode = ShellMode::Editing;
                true
            },
            _ => false,
        }
    }

    /// Create a new file
    fn new_file(&mut self) {
        self.lines = vec!["// New Logos file".to_string()];
        self.cursor_position = (0, 0);
        self.current_file = None;
        self.status_message = "New file created".to_string();
    }

    /// Save the current file
    fn save_file(&mut self) {
        if let Some(ref file_path) = self.current_file {
            let content = self.lines.join("\n");
            if let Err(e) = fs::write(file_path, content) {
                self.status_message = format!("Error saving file: {}", e);
            } else {
                self.status_message = format!("File saved: {}", file_path);
            }
        } else {
            self.status_message = "No file to save. Use Save As to create a new file.".to_string();
        }
    }

    /// Close the current file
    fn close_file(&mut self) {
        self.lines.clear();
        self.cursor_position = (0, 0);
        self.current_file = None;
        self.status_message = "File closed".to_string();
    }

    /// Show help information
    fn show_help(&mut self) {
        self.output.push("=== Logos Shell Help ===".to_string());
        self.output.push("Ctrl+N: New file".to_string());
        self.output.push("Ctrl+O: Open file".to_string());
        self.output.push("Ctrl+S: Save file".to_string());
        self.output.push("Ctrl+W: Close file".to_string());
        self.output.push("Ctrl+E: Exit shell".to_string());
        self.output.push("Ctrl+H: Show this help".to_string());
        self.status_message = "Help displayed".to_string();
    }

    /// Select previous menu item
    fn select_prev_menu_item(&mut self) {
        self.selected_menu_item = match self.selected_menu_item {
            MenuItem::File => MenuItem::Help,
            MenuItem::Edit => MenuItem::File,
            MenuItem::View => MenuItem::Edit,
            MenuItem::Run => MenuItem::View,
            MenuItem::Help => MenuItem::Run,
        };
    }

    /// Select next menu item
    fn select_next_menu_item(&mut self) {
        self.selected_menu_item = match self.selected_menu_item {
            MenuItem::File => MenuItem::Edit,
            MenuItem::Edit => MenuItem::View,
            MenuItem::View => MenuItem::Run,
            MenuItem::Run => MenuItem::Help,
            MenuItem::Help => MenuItem::File,
        };
    }

    /// Activate the selected menu item
    fn activate_menu_item(&mut self) {
        match self.selected_menu_item {
            MenuItem::File => {
                // Toggle file browser
                if self.mode == ShellMode::FileBrowser {
                    self.mode = ShellMode::Editing;
                } else {
                    self.mode = ShellMode::FileBrowser;
                }
            },
            MenuItem::Edit => {
                // Switch to insert mode
                self.mode = ShellMode::Insert;
            },
            MenuItem::View => {
                // Toggle menu visibility
                self.menu_visible = !self.menu_visible;
            },
            MenuItem::Run => {
                // Execute the current code
                self.execute_current_code();
            },
            MenuItem::Help => {
                self.show_help();
            },
        }
    }

    /// Select previous file in file browser
    fn select_prev_file(&mut self) {
        if self.file_browser_state.selected_index > 0 {
            self.file_browser_state.selected_index -= 1;

            // Adjust visible start if needed
            if self.file_browser_state.selected_index < self.file_browser_state.visible_start {
                self.file_browser_state.visible_start = self.file_browser_state.selected_index;
            }
        }
    }

    /// Select next file in file browser
    fn select_next_file(&mut self) {
        if self.file_browser_state.selected_index + 1 < self.file_browser_state.files.len() {
            self.file_browser_state.selected_index += 1;

            // Adjust visible start if needed
            let visible_end = self.file_browser_state.visible_start + 10; // Assuming 10 visible items
            if self.file_browser_state.selected_index >= visible_end {
                self.file_browser_state.visible_start = self.file_browser_state.selected_index - 9;
            }
        }
    }

    /// Open the selected file
    fn open_selected_file(&mut self) {
        if let Some(file_name) = self.file_browser_state.files.get(self.file_browser_state.selected_index) {
            let file_path = format!("{}/{}", self.file_browser_state.current_dir, file_name);

            match fs::read_to_string(&file_path) {
                Ok(content) => {
                    self.lines = content.lines().map(|s| s.to_string()).collect();
                    self.current_file = Some(file_path);
                    self.mode = ShellMode::Editing;
                    self.status_message = format!("Opened: {}", file_name);
                }
                Err(e) => {
                    self.status_message = format!("Error opening file: {}", e);
                }
            }
        }
    }

    /// Execute the current code
    fn execute_current_code(&mut self) {
        let code = self.lines.join("\n");
        self.output.push(format!("> Executing code..."));

        // For now, just add a placeholder result
        // In a real implementation, this would parse and execute the code
        self.output.push("Code executed successfully".to_string());
        self.status_message = "Code executed".to_string();
    }

    /// Process a command
    fn process_command(&mut self, command: &str) -> Vec<String> {
        let command = command.trim();

        if command.starts_with(':') {
            // Built-in commands
            match command {
                ":help" => vec![
                    "Logos Shell Commands:".to_string(),
                    ":help - Show this help".to_string(),
                    ":clear - Clear the screen".to_string(),
                    ":quit - Exit the shell".to_string(),
                    ":eval <expression> - Evaluate an expression".to_string(),
                    ":load <file> - Load a file".to_string(),
                ],
                ":clear" => {
                    vec!["Screen cleared".to_string()]  // We'll handle clearing in the caller
                },
                ":quit" => {
                    // This would normally trigger an exit, but we'll just return a message
                    vec!["Use Ctrl+C to exit".to_string()]
                },
                cmd if cmd.starts_with(":eval ") => {
                    let expr = cmd.strip_prefix(":eval ").unwrap_or("");
                    self.evaluate_expression(expr)
                },
                cmd if cmd.starts_with(":load ") => {
                    let filename = cmd.strip_prefix(":load ").unwrap_or("");
                    self.load_file(filename)
                },
                _ => vec![format!("Unknown command: {}", command)],
            }
        } else {
            // Try to evaluate as Logos code
            self.evaluate_logos_code(command)
        }
    }

    /// Evaluate a Logos expression
    fn evaluate_expression(&mut self, expr: &str) -> Vec<String> {
        // For now, just return the expression as-is
        // In a real implementation, this would parse and evaluate the expression
        vec![format!("Result: {}", expr)]
    }

    /// Load and execute a file
    fn load_file(&mut self, filename: &str) -> Vec<String> {
        match std::fs::read_to_string(filename) {
            Ok(content) => {
                self.evaluate_logos_code(&content)
            }
            Err(e) => {
                vec![format!("Error loading file '{}': {}", filename, e)]
            }
        }
    }

    /// Evaluate Logos code
    fn evaluate_logos_code(&self, code: &str) -> Vec<String> {
        // For now, just return the code as-is since the actual modules may not exist
        vec![format!("Evaluated: {}", code)]
    }
}

/// Run the Logos shell
pub fn run_shell() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    stdout().execute(EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // Create shell state
    let mut state = ShellState::new();

    // Run the main loop
    loop {
        terminal.draw(|f| ui(f, &mut state))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    // Handle special key combinations first
                    if state.handle_key_event(key) {
                        // If the key was handled by handle_key_event, continue to next iteration
                        continue;
                    }

                    // Handle other key events
                    match key.code {
                        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break; // Exit on Ctrl+Q
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break; // Exit on Ctrl+C
                        }
                        KeyCode::Char('i') if state.mode != ShellMode::Insert => {
                            state.toggle_insert_mode(); // Enter insert mode
                        }
                        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            state.toggle_command_mode(); // Enter command mode
                        }
                        KeyCode::Esc => {
                            // Exit file browser mode or return to editing mode
                            if state.mode == ShellMode::FileBrowser {
                                state.mode = ShellMode::Editing;
                            } else {
                                state.mode = ShellMode::Editing; // Return to editing mode
                            }
                        }
                        KeyCode::Enter => {
                            if state.mode == ShellMode::Command {
                                state.execute_input(); // Execute command
                            } else if state.mode == ShellMode::FileBrowser {
                                state.open_selected_file(); // Open selected file in file browser
                            } else {
                                state.handle_enter(); // Handle enter in insert mode
                            }
                        }
                        KeyCode::Backspace | KeyCode::Char('\u{7f}') => {
                            state.handle_backspace();
                        }
                        KeyCode::Left => {
                            if state.mode == ShellMode::Menu {
                                state.select_prev_menu_item();
                            } else {
                                state.move_cursor_left();
                            }
                        }
                        KeyCode::Right => {
                            if state.mode == ShellMode::Menu {
                                state.select_next_menu_item();
                            } else {
                                state.move_cursor_right();
                            }
                        }
                        KeyCode::Up => {
                            if state.mode == ShellMode::FileBrowser {
                                state.select_prev_file();
                            } else {
                                state.move_cursor_up();
                            }
                        }
                        KeyCode::Down => {
                            if state.mode == ShellMode::FileBrowser {
                                state.select_next_file();
                            } else {
                                state.move_cursor_down();
                            }
                        }
                        KeyCode::Tab => {
                            // Switch between menu and editor modes
                            state.mode = match state.mode {
                                ShellMode::Editing => ShellMode::Menu,
                                ShellMode::Menu => ShellMode::Editing,
                                _ => ShellMode::Editing,
                            };
                        }
                        KeyCode::Char(ch) => {
                            if state.mode == ShellMode::Insert {
                                state.handle_char_input(ch);
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        crossterm::event::MouseEventKind::ScrollUp => {
                            if state.scroll_offset > 0 {
                                state.scroll_offset -= 1;
                            }
                        }
                        crossterm::event::MouseEventKind::ScrollDown => {
                            if (state.scroll_offset as usize) < state.lines.len() {
                                state.scroll_offset += 1;
                            }
                        }
                        crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                            // Handle clicking on menu items
                            if mouse.row == 0 { // Clicked on menu bar
                                let col = mouse.column;
                                // Determine which menu item was clicked based on approximate positions
                                if col >= 1 && col < 5 { // File menu
                                    state.selected_menu_item = MenuItem::File;
                                    state.mode = ShellMode::Menu;
                                } else if col >= 6 && col < 10 { // Edit menu
                                    state.selected_menu_item = MenuItem::Edit;
                                    state.mode = ShellMode::Menu;
                                } else if col >= 11 && col < 15 { // View menu
                                    state.selected_menu_item = MenuItem::View;
                                    state.mode = ShellMode::Menu;
                                } else if col >= 16 && col < 19 { // Run menu
                                    state.selected_menu_item = MenuItem::Run;
                                    state.mode = ShellMode::Menu;
                                } else if col >= 20 && col < 24 { // Help menu
                                    state.selected_menu_item = MenuItem::Help;
                                    state.mode = ShellMode::Menu;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

/// Draw the UI
fn ui(f: &mut Frame, state: &mut ShellState) {
    let size = f.size();

    // Create layout with menu bar at the top
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Menu bar
            Constraint::Ratio(2, 3),  // Editor area (2/3 of remaining screen)
            Constraint::Ratio(1, 3),  // Output area (1/3 of remaining screen)
        ])
        .split(size);

    // Render menu bar
    let menu_items = ["File", "Edit", "View", "Run", "Help"];
    let mut menu_spans = Vec::new();

    for (i, &item) in menu_items.iter().enumerate() {
        let style = if state.selected_menu_item as usize == i && state.mode == ShellMode::Menu {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED)
        } else {
            Style::default().fg(Color::White)
        };

        menu_spans.push(Span::styled(format!(" {} ", item), style));
        if i < menu_items.len() - 1 {
            menu_spans.push(Span::raw(" | "));
        }
    }

    let menu_paragraph = Paragraph::new(Line::from(menu_spans))
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(menu_paragraph, chunks[0]);

    // Render editor area or file browser depending on mode
    if state.mode == ShellMode::FileBrowser {
        // Render file browser
        let file_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // File browser
                Constraint::Percentage(80), // Preview pane
            ])
            .split(chunks[1]);

        // Render file list
        let start_idx = state.file_browser_state.visible_start;
        let end_idx = std::cmp::min(start_idx + 10, state.file_browser_state.files.len());

        let items: Vec<ListItem> = state.file_browser_state.files[start_idx..end_idx]
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let idx = start_idx + i;
                let style = if idx == state.file_browser_state.selected_index {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };
                ListItem::new(Span::styled(file.as_str(), style))
            })
            .collect();

        let file_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Files"))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        f.render_widget(file_list, file_chunks[0]);

        // Render preview pane
        let preview_content = if let Some(selected_file) = state.file_browser_state.files.get(state.file_browser_state.selected_index) {
            let path = format!("{}/{}", state.file_browser_state.current_dir, selected_file);
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    // Take ownership of the content to ensure it lives long enough
                    let content_lines: Vec<String> = content
                        .lines()
                        .take(20) // Limit preview to 20 lines
                        .map(|line| line.to_string())
                        .collect();

                    let lines: Vec<Line> = content_lines
                        .into_iter()
                        .map(|line| Line::from(vec![Span::raw(line)]))
                        .collect();
                    Text::from(lines)
                }
                Err(_) => Text::from("Could not read file"),
            }
        } else {
            Text::from("No file selected")
        };

        let preview = Paragraph::new(preview_content)
            .block(Block::default().borders(Borders::ALL).title("Preview"));
        f.render_widget(preview, file_chunks[1]);
    } else {
        // Render editor area
        let editor_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1), // Editor
                Constraint::Length(1), // Status bar
            ])
            .split(chunks[1]);

        // Calculate visible lines based on terminal height and scroll offset
        let visible_start = state.scroll_offset as usize;
        let visible_end = std::cmp::min(
            visible_start + (editor_chunks[0].height as usize - 2), // Account for borders
            state.lines.len()
        );

        // Create text for editor area with line numbers
        let mut editor_lines = Vec::new();
        for (i, line) in state.lines[visible_start..visible_end].iter().enumerate() {
            let line_num = visible_start + i + 1;
            let line_text = format!("{:4} │ {}", line_num, line);

            // Highlight the current line
            let line_spans = if (visible_start + i) as u16 == state.cursor_position.0 {
                vec![Span::styled(line_text, Style::default().bg(Color::DarkGray))]
            } else {
                vec![Span::raw(line_text)]
            };

            editor_lines.push(Line::from(line_spans));
        }

        let editor_widget = Paragraph::new(editor_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Editor - Mode: {:?} - {}",
                        state.mode,
                        state.current_file.as_ref().map(|f| f.as_str()).unwrap_or("untitled")))
            )
            .scroll((state.scroll_offset, 0));

        f.render_widget(editor_widget, editor_chunks[0]);

        // Render status bar
        let status_text = format!("Ln {}, Col {} | {}",
            state.cursor_position.0 + 1,
            state.cursor_position.1 + 1,
            state.status_message);
        let status_bar = Paragraph::new(Line::from(vec![Span::raw(status_text)]))
            .style(Style::default().bg(Color::Rgb(64, 64, 64)).fg(Color::Rgb(220, 220, 220)));
        f.render_widget(status_bar, editor_chunks[1]);
    }

    // Render output area
    let output_lines: Vec<Line> = state.output
        .iter()
        .map(|line| Line::from(vec![Span::raw(line.as_str())]))
        .collect();

    let output_widget = Paragraph::new(output_lines)
        .block(Block::default().borders(Borders::ALL).title("Output"));

    f.render_widget(output_widget, chunks[2]);
}