use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseButton},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Direction, Alignment, Rect},
    style::{Style, Color, Modifier},
    widgets::{Block, Borders, BorderType, Paragraph, Wrap, List, ListItem},
    text::{Span, Line, Text},
    Frame, Terminal,
};
use std::io::{self, stdout};

#[derive(Debug, Clone, PartialEq)]
pub enum ShellMode {
    Editing,
    Insert,
    Command,
    Menu,
    FileBrowser,
    CommandPrompt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem {
    File,
    Edit,
    View,
    Run,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowsingMode {
    Files,
    Directories,
}

#[derive(Debug, Clone)]
pub struct FileBrowserState {
    pub current_dir: String,
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub selected_index: usize,
    pub visible_start: usize,
    pub browsing_mode: BrowsingMode,
}

impl Default for FileBrowserState {
    fn default() -> Self {
        Self {
            current_dir: std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
            files: Vec::new(),
            directories: Vec::new(),
            selected_index: 0,
            visible_start: 0,
            browsing_mode: BrowsingMode::Files,
        }
    }
}

pub struct ShellState {
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub scroll_offset: u16,
    pub mode: ShellMode,
    pub selected_menu_item: MenuItem,
    pub output: Vec<String>,
    pub status_message: String,
    pub file_browser_state: FileBrowserState,
    pub command_buffer: String,
    pub command_cursor: usize,
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            lines: vec!["// Welcome to Logos Shell".to_string(), "// Press 'i' to enter insert mode".to_string()],
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset: 0,
            mode: ShellMode::Editing,
            selected_menu_item: MenuItem::File,
            output: vec!["Logos Shell initialized".to_string()],
            status_message: "Press 'i' to enter insert mode".to_string(),
            file_browser_state: FileBrowserState::default(),
            command_buffer: String::new(),
            command_cursor: 0,
        }
    }

    pub fn refresh_file_list(&mut self) {
        self.file_browser_state.files.clear();
        self.file_browser_state.directories.clear();
        
        if let Ok(entries) = std::fs::read_dir(&self.file_browser_state.current_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if file_type.is_file() {
                        self.file_browser_state.files.push(name);
                    } else if file_type.is_dir() {
                        self.file_browser_state.directories.push(name);
                    }
                }
            }
        }
        
        self.file_browser_state.files.sort();
        self.file_browser_state.directories.sort();
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        // Handle special key combinations
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('s') => {
                    self.save_file();
                    return true;
                }
                KeyCode::Char('o') => {
                    self.toggle_file_browser();
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    pub fn save_file(&mut self) {
        // For now, just update the status message
        self.status_message = "File saved".to_string();
    }

    pub fn toggle_file_browser(&mut self) {
        self.mode = match self.mode {
            ShellMode::FileBrowser => ShellMode::Editing,
            _ => ShellMode::FileBrowser,
        };
        
        if self.mode == ShellMode::FileBrowser {
            self.refresh_file_list();
        }
    }

    pub fn toggle_insert_mode(&mut self) {
        self.mode = match self.mode {
            ShellMode::Insert => ShellMode::Editing,
            _ => ShellMode::Insert,
        };
        
        self.status_message = match self.mode {
            ShellMode::Insert => "INSERT MODE".to_string(),
            _ => "NORMAL MODE".to_string(),
        };
    }

    pub fn toggle_command_mode(&mut self) {
        self.mode = match self.mode {
            ShellMode::Command => ShellMode::Editing,
            _ => ShellMode::Command,
        };
        
        self.status_message = match self.mode {
            ShellMode::Command => "COMMAND MODE".to_string(),
            _ => "NORMAL MODE".to_string(),
        };
    }

    pub fn execute_input(&mut self) {
        let input = self.lines.join("\n");
        let code = self.lines.join("\n");
        self.output.push(format!("> Executing code..."));

        // For now, just add a placeholder result
        // In a real implementation, this would parse and execute the code
        self.output.push("Code executed successfully".to_string());
        self.status_message = "Code executed".to_string();
    }

    pub fn enter_command_prompt(&mut self) {
        self.mode = ShellMode::CommandPrompt;
        self.command_buffer.clear();
        self.command_cursor = 0;
        self.status_message = "Enter command (type 'help' for help)".to_string();
    }

    pub fn exit_command_prompt(&mut self) {
        self.mode = ShellMode::Editing;
        self.command_buffer.clear();
        self.command_cursor = 0;
    }

    pub fn handle_command_in_prompt(&mut self) {
        let input = self.command_buffer.clone();
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            self.exit_command_prompt();
            return;
        }

        match parts[0] {
            "cd" => {
                if parts.len() > 1 {
                    self.change_directory(parts[1]);
                } else {
                    self.status_message = "Usage: cd <directory>".to_string();
                }
            },
            "ls" | "dir" => {
                let contents = self.list_directory_contents();
                self.output.push(contents);
            },
            "pwd" => {
                self.output.push(format!("Current directory: {}", self.file_browser_state.current_dir));
            },
            "clear" => {
                self.output.clear();
            },
            "help" => {
                self.output.push("Available commands:".to_string());
                self.output.push("  cd <directory> - Change directory".to_string());
                self.output.push("  ls/dir - List directory contents".to_string());
                self.output.push("  pwd - Print working directory".to_string());
                self.output.push("  clear - Clear output".to_string());
                self.output.push("  help - Show this help".to_string());
            },
            _ => {
                self.status_message = format!("Unknown command: {}", parts[0]);
            }
        }

        self.exit_command_prompt();
    }

    pub fn move_command_cursor_left(&mut self) {
        if self.command_cursor > 0 {
            self.command_cursor -= 1;
        }
    }

    pub fn move_command_cursor_right(&mut self) {
        if self.command_cursor < self.command_buffer.len() {
            self.command_cursor += 1;
        }
    }

    pub fn add_char_to_command(&mut self, ch: char) {
        self.command_buffer.insert(self.command_cursor, ch);
        self.command_cursor += 1;
    }

    pub fn remove_char_from_command(&mut self) {
        if self.command_cursor > 0 {
            self.command_buffer.remove(self.command_cursor - 1);
            self.command_cursor -= 1;
        }
    }

    pub fn change_directory(&mut self, path: &str) {
        let new_path = std::path::Path::new(path);
        if new_path.is_dir() {
            match std::fs::canonicalize(new_path) {
                Ok(canonical_path) => {
                    self.file_browser_state.current_dir = canonical_path.to_string_lossy().to_string();
                    self.refresh_file_list();
                    self.status_message = format!("Changed directory to: {}", self.file_browser_state.current_dir);
                }
                Err(e) => {
                    self.status_message = format!("Error changing directory: {}", e);
                }
            }
        } else {
            self.status_message = format!("Error: '{}' is not a directory", path);
        }
    }

    pub fn list_directory_contents(&self) -> String {
        match std::fs::read_dir(&self.file_browser_state.current_dir) {
            Ok(entries) => {
                let mut contents = Vec::new();
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        contents.push(format!("F  {}", name_str));
                    } else {
                        contents.push(format!("D  {}", name_str));
                    }
                }
                contents.join("\n")
            }
            Err(e) => format!("Error reading directory: {}", e),
        }
    }

    pub fn process_command(&mut self, command: &str) -> Vec<String> {
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

    pub fn evaluate_expression(&mut self, expr: &str) -> Vec<String> {
        // For now, just return the expression as-is
        // In a real implementation, this would parse and evaluate the expression
        vec![format!("Result: {}", expr)]
    }

    pub fn load_file(&mut self, filename: &str) -> Vec<String> {
        match std::fs::read_to_string(filename) {
            Ok(content) => {
                self.evaluate_logos_code(&content)
            }
            Err(e) => {
                vec![format!("Error loading file '{}': {}", filename, e)]
            }
        }
    }

    pub fn evaluate_logos_code(&self, code: &str) -> Vec<String> {
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
                            if state.mode == ShellMode::CommandPrompt {
                                state.handle_command_in_prompt(); // Execute command in command prompt mode
                            } else {
                                break; // Exit on Ctrl+C in other modes
                            }
                        }
                        KeyCode::Char('i') if state.mode != ShellMode::Insert => {
                            state.toggle_insert_mode(); // Enter insert mode
                        }
                        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            state.toggle_command_mode(); // Enter command mode
                        }
                        KeyCode::Esc => {
                            // Exit different modes appropriately
                            match state.mode {
                                ShellMode::FileBrowser => {
                                    state.mode = ShellMode::Editing;
                                },
                                ShellMode::CommandPrompt => {
                                    state.exit_command_prompt(); // Exit command prompt mode
                                },
                                _ => {
                                    state.mode = ShellMode::Editing; // Return to editing mode
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if state.mode == ShellMode::Command {
                                state.execute_input(); // Execute command
                            } else if state.mode == ShellMode::FileBrowser {
                                state.open_selected_file(); // Open selected file in file browser
                            } else if state.mode == ShellMode::CommandPrompt {
                                state.handle_command_in_prompt(); // Execute command in command prompt mode
                            } else {
                                state.handle_enter(); // Handle enter in insert mode
                            }
                        }
                        KeyCode::Backspace => {
                            if state.mode == ShellMode::CommandPrompt {
                                state.remove_char_from_command();
                            } else {
                                state.handle_backspace();
                            }
                        }
                        KeyCode::Char('\u{7f}') => {
                            // Delete key
                            if state.mode == ShellMode::CommandPrompt {
                                state.remove_char_from_command();
                            } else {
                                state.handle_backspace();
                            }
                        }
                        KeyCode::Left => {
                            if state.mode == ShellMode::Menu {
                                state.select_prev_menu_item();
                            } else if state.mode == ShellMode::CommandPrompt {
                                state.move_command_cursor_left();
                            } else {
                                state.move_cursor_left();
                            }
                        }
                        KeyCode::Right => {
                            if state.mode == ShellMode::Menu {
                                state.select_next_menu_item();
                            } else if state.mode == ShellMode::CommandPrompt {
                                state.move_command_cursor_right();
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
                            match state.mode {
                                ShellMode::Insert => {
                                    state.handle_char_input(ch);
                                },
                                ShellMode::CommandPrompt => {
                                    state.add_char_to_command(ch);
                                },
                                _ => {
                                    // In other modes, we might want to handle character input differently
                                    // For now, ignore character input in other modes
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        crossterm::event::MouseEventKind::ScrollUp => {
                            if state.mode == ShellMode::FileBrowser {
                                // Handle scrolling in file browser
                                if state.file_browser_state.visible_start > 0 {
                                    state.file_browser_state.visible_start -= 1;
                                }
                            } else {
                                // Handle scrolling in editor
                                if state.scroll_offset > 0 {
                                    state.scroll_offset -= 1;
                                }
                            }
                        }
                        crossterm::event::MouseEventKind::ScrollDown => {
                            if state.mode == ShellMode::FileBrowser {
                                // Handle scrolling in file browser
                                let max_visible = if state.file_browser_state.browsing_mode == BrowsingMode::Files {
                                    state.file_browser_state.files.len()
                                } else {
                                    state.file_browser_state.directories.len()
                                };

                                if state.file_browser_state.visible_start + 10 < max_visible {
                                    state.file_browser_state.visible_start += 1;
                                }
                            } else {
                                // Handle scrolling in editor
                                if (state.scroll_offset as usize) < state.lines.len() {
                                    state.scroll_offset += 1;
                                }
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
                            } else if state.mode == ShellMode::FileBrowser {
                                // Handle clicking on file/directory in file browser
                                // Calculate the dimensions for file browser area
                                let size = terminal.size()?;
                                let chunks = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints([
                                        Constraint::Length(1),  // Menu bar
                                        Constraint::Ratio(2, 3),  // Editor area
                                        Constraint::Ratio(1, 3),  // Output area
                                    ])
                                    .split(size);

                                let file_chunks = Layout::default()
                                    .direction(Direction::Horizontal)
                                    .constraints([
                                        Constraint::Percentage(20), // File browser
                                        Constraint::Percentage(80), // Preview pane
                                    ])
                                    .split(chunks[1]);

                                // Check if click is in the file browser area
                                if mouse.column >= file_chunks[0].x && mouse.column < file_chunks[0].x + file_chunks[0].width {
                                    // Calculate which item was clicked
                                    let item_index = (mouse.row - file_chunks[0].y - 1) as usize + state.file_browser_state.visible_start; // -1 for border

                                    if state.file_browser_state.browsing_mode == BrowsingMode::Files {
                                        if item_index < state.file_browser_state.files.len() {
                                            state.file_browser_state.selected_index = item_index;
                                            state.open_selected_file(); // Open the clicked file
                                        }
                                    } else if state.file_browser_state.browsing_mode == BrowsingMode::Directories {
                                        if item_index < state.file_browser_state.directories.len() {
                                            state.file_browser_state.selected_index = item_index;
                                            state.open_selected_file(); // Navigate to the clicked directory
                                        }
                                    }
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
        let style = if state.selected_menu_item.clone() as usize == i && state.mode == ShellMode::Menu {
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

        // Render file or directory list based on browsing mode
        let start_idx = state.file_browser_state.visible_start;
        let (items, title) = match state.file_browser_state.browsing_mode {
            BrowsingMode::Files => {
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
                (items, "Files")
            },
            BrowsingMode::Directories => {
                let end_idx = std::cmp::min(start_idx + 10, state.file_browser_state.directories.len());
                let items: Vec<ListItem> = state.file_browser_state.directories[start_idx..end_idx]
                    .iter()
                    .enumerate()
                    .map(|(i, dir)| {
                        let idx = start_idx + i;
                        let style = if idx == state.file_browser_state.selected_index {
                            Style::default().bg(Color::Blue).fg(Color::White)
                        } else {
                            Style::default().fg(Color::Cyan) // Use cyan for directories
                        };
                        // Add a directory indicator
                        let text = format!("[{}] {}", dir, if idx == state.file_browser_state.selected_index { " ←" } else { "" });
                        ListItem::new(Span::styled(text, style))
                    })
                    .collect();
                (items, "Directories")
            },
        };

        let file_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
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

        // Calculate visible lines based on terminal height
        let visible_start = state.scroll_offset as usize;
        let visible_end = std::cmp::min(
            visible_start + (editor_chunks[0].height as usize) - 2, // Account for borders
            state.lines.len()
        );

        // Create text content
        let mut spans = Vec::new();
        for (idx, line) in state.lines[visible_start..visible_end].iter().enumerate() {
            let line_num = visible_start + idx + 1;
            let mut line_spans = Vec::new();
            
            // Add line number
            line_spans.push(Span::styled(
                format!("{:4} │ ", line_num),
                Style::default().fg(Color::DarkGray)
            ));
            
            // Add line content
            line_spans.push(Span::raw(line));
            
            spans.push(Line::from(line_spans));
        }

        let text = Text::from(spans);
        let editor_paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Editor"))
            .wrap(Wrap { trim: true });

        f.render_widget(editor_paragraph, editor_chunks[0]);

        // Render status bar
        let status_text = match state.mode {
            ShellMode::Insert => Span::styled("INSERT", Style::default().bg(Color::Green).fg(Color::Black)),
            ShellMode::Command => Span::styled("COMMAND", Style::default().bg(Color::Blue).fg(Color::White)),
            ShellMode::Menu => Span::styled("MENU", Style::default().bg(Color::Yellow).fg(Color::Black)),
            ShellMode::FileBrowser => Span::styled("FILE BROWSER", Style::default().bg(Color::Cyan).fg(Color::Black)),
            ShellMode::CommandPrompt => Span::styled("CMD PROMPT", Style::default().bg(Color::Magenta).fg(Color::White)),
            _ => Span::styled("NORMAL", Style::default().bg(Color::Gray).fg(Color::White)),
        };

        let status_bar = Paragraph::new(Line::from(vec![
            status_text,
            Span::raw(format!("  {}", state.status_message)),
        ]))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(status_bar, editor_chunks[1]);
    }

    // Render output area
    let output_text: Vec<Line> = state.output
        .iter()
        .rev() // Reverse to show newest at the bottom
        .take(chunks[2].height as usize - 2) // Account for borders
        .map(|line| Line::from(Span::raw(line)))
        .collect();

    let output_paragraph = Paragraph::new(Text::from(output_text))
        .block(Block::default().borders(Borders::ALL).title("Output"))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);
    f.render_widget(output_paragraph, chunks[2]);
}

// Additional helper methods for ShellState
impl ShellState {
    pub fn handle_enter(&mut self) {
        if self.mode == ShellMode::Insert {
            // Insert a new line at cursor position
            self.lines.insert(self.cursor_y + 1, String::new());
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.mode == ShellMode::Insert && self.cursor_x > 0 {
            if let Some(line) = self.lines.get_mut(self.cursor_y) {
                if !line.is_empty() && self.cursor_x <= line.len() {
                    line.remove(self.cursor_x - 1);
                    self.cursor_x -= 1;
                }
            }
        }
    }

    pub fn handle_char_input(&mut self, ch: char) {
        if self.mode == ShellMode::Insert {
            if let Some(line) = self.lines.get_mut(self.cursor_y) {
                line.insert(self.cursor_x, ch);
                self.cursor_x += 1;
            } else {
                // If we're past the end of lines, add a new line
                self.lines.push(ch.to_string());
                self.cursor_y = self.lines.len() - 1;
                self.cursor_x = 1;
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            if let Some(prev_line) = self.lines.get(self.cursor_y) {
                self.cursor_x = prev_line.len();
            } else {
                self.cursor_x = 0;
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        if let Some(current_line) = self.lines.get(self.cursor_y) {
            if self.cursor_x < current_line.len() {
                self.cursor_x += 1;
            } else if self.cursor_y < self.lines.len() - 1 {
                self.cursor_y += 1;
                self.cursor_x = 0;
            }
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            if let Some(line) = self.lines.get(self.cursor_y) {
                self.cursor_x = std::cmp::min(self.cursor_x, line.len());
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;
            if let Some(line) = self.lines.get(self.cursor_y) {
                self.cursor_x = std::cmp::min(self.cursor_x, line.len());
            }
        }
    }

    pub fn select_prev_menu_item(&mut self) {
        self.selected_menu_item = match self.selected_menu_item {
            MenuItem::File => MenuItem::Help,
            MenuItem::Edit => MenuItem::File,
            MenuItem::View => MenuItem::Edit,
            MenuItem::Run => MenuItem::View,
            MenuItem::Help => MenuItem::Run,
        };
    }

    pub fn select_next_menu_item(&mut self) {
        self.selected_menu_item = match self.selected_menu_item {
            MenuItem::File => MenuItem::Edit,
            MenuItem::Edit => MenuItem::View,
            MenuItem::View => MenuItem::Run,
            MenuItem::Run => MenuItem::Help,
            MenuItem::Help => MenuItem::File,
        };
    }

    pub fn select_prev_file(&mut self) {
        if self.file_browser_state.selected_index > 0 {
            self.file_browser_state.selected_index -= 1;
        }
    }

    pub fn select_next_file(&mut self) {
        let max_idx = match self.file_browser_state.browsing_mode {
            BrowsingMode::Files => self.file_browser_state.files.len(),
            BrowsingMode::Directories => self.file_browser_state.directories.len(),
        };
        
        if self.file_browser_state.selected_index < max_idx.saturating_sub(1) {
            self.file_browser_state.selected_index += 1;
        }
    }

    pub fn open_selected_file(&mut self) {
        // This would open the selected file in the editor
        // For now, just update the status message
        match self.file_browser_state.browsing_mode {
            BrowsingMode::Files => {
                if let Some(file) = self.file_browser_state.files.get(self.file_browser_state.selected_index) {
                    let path = format!("{}/{}", self.file_browser_state.current_dir, file);
                    self.status_message = format!("Opening file: {}", path);
                    
                    // In a real implementation, we'd load the file content into the editor
                    // For now, just simulate by updating the lines
                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            self.lines = content.lines().map(|s| s.to_string()).collect();
                            self.status_message = format!("Opened file: {}", file);
                        }
                        Err(e) => {
                            self.status_message = format!("Error opening file {}: {}", file, e);
                        }
                    }
                }
            },
            BrowsingMode::Directories => {
                if let Some(dir) = self.file_browser_state.directories.get(self.file_browser_state.selected_index) {
                    let path = format!("{}/{}", self.file_browser_state.current_dir, dir);
                    self.change_directory(&path);
                }
            }
        }
    }
}