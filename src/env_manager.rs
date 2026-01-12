// Logos Programming Language Environment Manager
// This module provides environment management for multi-language integration
// including Python virtual environments, Go modules, and other language-specific environments

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Represents different types of environments for multi-language support
#[derive(Debug, Clone)]
pub enum EnvironmentType {
    Python,
    Go,
    JavaScript,
    Rust,
    C,
    CPP,
    Java,
    Logos,  // Add Logos environment type
    Other(String),
}

/// Represents a language-specific environment with its configuration
#[derive(Debug, Clone)]
pub struct LanguageEnvironment {
    pub name: String,                    // Name of the environment
    pub language: EnvironmentType,
    pub path: PathBuf,
    pub dependencies: Vec<String>,
    pub active: bool,
    pub config_file: Option<PathBuf>,
}

/// Environment manager for handling multiple language environments
#[derive(Debug)]
pub struct EnvironmentManager {
    environments: HashMap<String, LanguageEnvironment>,
    current_env: Option<String>,
}

impl EnvironmentManager {
    /// Creates a new environment manager instance
    pub fn new() -> Self {
        Self {
            environments: HashMap::new(),
            current_env: None,
        }
    }

    /// Creates a new Python virtual environment
    pub fn create_python_env(&mut self, name: &str, path: PathBuf) -> Result<(), String> {
        if !self.is_python_available() {
            return Err("Python is not available in the system".to_string());
        }

        // Create the virtual environment
        let status = Command::new("python3")
            .args(&["-m", "venv", &path.to_string_lossy()])
            .status()
            .map_err(|e| format!("Failed to create Python environment: {}", e))?;

        if !status.success() {
            return Err("Failed to create Python environment".to_string());
        }

        let env = LanguageEnvironment {
            name: name.to_string(),
            language: EnvironmentType::Python,
            path,
            dependencies: Vec::new(),
            active: false,
            config_file: Some(PathBuf::from("requirements.txt")),
        };

        self.environments.insert(name.to_string(), env);
        Ok(())
    }

    /// Creates a new Go module environment
    pub fn create_go_env(&mut self, name: &str, path: PathBuf) -> Result<(), String> {
        if !self.is_go_available() {
            return Err("Go is not available in the system".to_string());
        }

        // Initialize a new Go module
        let status = Command::new("go")
            .args(&["mod", "init", name])
            .current_dir(&path)
            .status()
            .map_err(|e| format!("Failed to initialize Go module: {}", e))?;

        if !status.success() {
            return Err("Failed to initialize Go module".to_string());
        }

        let env = LanguageEnvironment {
            name: name.to_string(),
            language: EnvironmentType::Go,
            path,
            dependencies: Vec::new(),
            active: false,
            config_file: Some(PathBuf::from("go.mod")),
        };

        self.environments.insert(name.to_string(), env);
        Ok(())
    }

    /// Creates a new JavaScript/npm environment
    pub fn create_js_env(&mut self, name: &str, path: PathBuf) -> Result<(), String> {
        if !self.is_node_available() {
            return Err("Node.js is not available in the system".to_string());
        }

        // Initialize a new npm package
        let status = Command::new("npm")
            .args(&["init", "-y"])
            .current_dir(&path)
            .status()
            .map_err(|e| format!("Failed to initialize JavaScript environment: {}", e))?;

        if !status.success() {
            return Err("Failed to initialize JavaScript environment".to_string());
        }

        let env = LanguageEnvironment {
            name: name.to_string(),
            language: EnvironmentType::JavaScript,
            path,
            dependencies: Vec::new(),
            active: false,
            config_file: Some(PathBuf::from("package.json")),
        };

        self.environments.insert(name.to_string(), env);
        Ok(())
    }

    /// Creates a new Logos environment
    pub fn create_logos_env(&mut self, name: &str, path: PathBuf) -> Result<(), String> {
        // Check if Logos is available in the system
        if !self.is_logos_available() {
            return Err("Logos is not available in the system".to_string());
        }

        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create Logos environment directory: {}", e))?;

        // Initialize a new Logos project
        let status = Command::new("cargo")
            .args(&["run", "--bin", "logos", "--", "init", name])
            .current_dir(&path)
            .status()
            .map_err(|e| format!("Failed to initialize Logos environment: {}", e))?;

        if !status.success() {
            return Err("Failed to initialize Logos environment".to_string());
        }

        let env = LanguageEnvironment {
            name: name.to_string(),
            language: EnvironmentType::Logos,
            path,
            dependencies: Vec::new(),
            active: false,
            config_file: Some(PathBuf::from("logos.toml")), // Logos configuration file
        };

        self.environments.insert(name.to_string(), env);
        Ok(())
    }

    /// Activates an environment by name
    pub fn activate_env(&mut self, name: &str) -> Result<(), String> {
        if let Some(env) = self.environments.get_mut(name) {
            env.active = true;
            self.current_env = Some(name.to_string());
            Ok(())
        } else {
            Err(format!("Environment '{}' not found", name))
        }
    }

    /// Deactivates the current environment
    pub fn deactivate_env(&mut self) -> Result<(), String> {
        if let Some(name) = &self.current_env {
            if let Some(env) = self.environments.get_mut(name) {
                env.active = false;
                self.current_env = None;
                Ok(())
            } else {
                Err("Current environment not found".to_string())
            }
        } else {
            Err("No environment is currently active".to_string())
        }
    }

    /// Adds a dependency to an environment
    pub fn add_dependency(&mut self, env_name: &str, dependency: String) -> Result<(), String> {
        if let Some(env) = self.environments.get_mut(env_name) {
            env.dependencies.push(dependency);
            Ok(())
        } else {
            Err(format!("Environment '{}' not found", env_name))
        }
    }

    /// Checks if Python is available in the system
    pub fn is_python_available(&self) -> bool {
        Command::new("python3")
            .arg("--version")
            .output()
            .is_ok()
            || Command::new("python")
                .arg("--version")
                .output()
                .is_ok()
    }

    /// Checks if Go is available in the system
    pub fn is_go_available(&self) -> bool {
        Command::new("go")
            .arg("version")
            .output()
            .is_ok()
    }

    /// Checks if Node.js is available in the system
    pub fn is_node_available(&self) -> bool {
        Command::new("node")
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Checks if Logos is available in the system
    pub fn is_logos_available(&self) -> bool {
        // Check if the logos binary is available
        Command::new("cargo")
            .args(&["run", "--bin", "logos", "--", "--version"])
            .output()
            .is_ok()
    }

    /// Checks if Rust is available in the system
    pub fn is_rust_available(&self) -> bool {
        Command::new("rustc")
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Creates a new Rust environment
    pub fn create_rust_env(&mut self, name: &str, path: PathBuf) -> Result<(), String> {
        if !self.is_rust_available() {
            return Err("Rust is not available in the system".to_string());
        }

        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create Rust environment directory: {}", e))?;

        // Initialize a new Rust project
        let status = Command::new("cargo")
            .args(&["new", "--bin", name])
            .current_dir(&path)
            .status()
            .map_err(|e| format!("Failed to initialize Rust environment: {}", e))?;

        if !status.success() {
            return Err("Failed to initialize Rust environment".to_string());
        }

        let env = LanguageEnvironment {
            name: name.to_string(),
            language: EnvironmentType::Rust,
            path,
            dependencies: Vec::new(),
            active: false,
            config_file: Some(PathBuf::from("Cargo.toml")), // Rust configuration file
        };

        self.environments.insert(name.to_string(), env);
        Ok(())
    }

    /// Gets the current active environment
    pub fn get_current_env(&self) -> Option<&LanguageEnvironment> {
        if let Some(name) = &self.current_env {
            self.environments.get(name)
        } else {
            None
        }
    }

    /// Gets an environment by name
    pub fn get_env(&self, name: &str) -> Option<&LanguageEnvironment> {
        self.environments.get(name)
    }

    /// Lists all registered environments
    pub fn list_envs(&self) -> Vec<String> {
        self.environments.keys().cloned().collect()
    }

    /// Synchronizes an environment with its dependencies
    pub fn sync_env(&self, env_name: &str) -> Result<(), String> {
        if let Some(env) = self.environments.get(env_name) {
            match &env.language {
                EnvironmentType::Python => {
                    if let Some(config_file) = &env.config_file {
                        if config_file.file_name().unwrap_or_default() == "requirements.txt" {
                            let status = Command::new("pip")
                                .args(&["install", "-r", &config_file.to_string_lossy()])
                                .current_dir(&env.path)
                                .status()
                                .map_err(|e| format!("Failed to sync Python environment: {}", e))?;

                            if !status.success() {
                                return Err("Failed to sync Python environment".to_string());
                            }
                        } else {
                            // Use pyproject.toml or setup.py
                            let status = Command::new("pip")
                                .args(&["install", "-e", "."])
                                .current_dir(&env.path)
                                .status()
                                .map_err(|e| format!("Failed to sync Python environment: {}", e))?;

                            if !status.success() {
                                return Err("Failed to sync Python environment".to_string());
                            }
                        }
                    }
                },
                EnvironmentType::Go => {
                    let status = Command::new("go")
                        .args(&["mod", "tidy"])
                        .current_dir(&env.path)
                        .status()
                        .map_err(|e| format!("Failed to sync Go environment: {}", e))?;

                    if !status.success() {
                        return Err("Failed to sync Go environment".to_string());
                    }
                },
                EnvironmentType::JavaScript => {
                    let status = Command::new("npm")
                        .args(&["install"])
                        .current_dir(&env.path)
                        .status()
                        .map_err(|e| format!("Failed to sync JavaScript environment: {}", e))?;

                    if !status.success() {
                        return Err("Failed to sync JavaScript environment".to_string());
                    }
                },
                EnvironmentType::Logos => {
                    // Sync Logos environment by installing dependencies
                    let status = Command::new("cargo")
                        .args(&["run", "--bin", "logos", "--", "sync"])
                        .current_dir(&env.path)
                        .status()
                        .map_err(|e| format!("Failed to sync Logos environment: {}", e))?;

                    if !status.success() {
                        return Err("Failed to sync Logos environment".to_string());
                    }
                },
                _ => {
                    return Err(format!("Sync not implemented for {:?}", env.language));
                }
            }

            Ok(())
        } else {
            Err(format!("Environment '{}' not found", env_name))
        }
    }
}

// Global environment manager instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_ENV_MANAGER: Arc<Mutex<EnvironmentManager>> = Arc::new(Mutex::new(EnvironmentManager::new()));
}

/// Creates a new Python environment
pub fn create_python_environment(name: &str, path: &str) -> Result<(), String> {
    let mut manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.create_python_env(name, PathBuf::from(path))
}

/// Creates a new Go environment
pub fn create_go_environment(name: &str, path: &str) -> Result<(), String> {
    let mut manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.create_go_env(name, PathBuf::from(path))
}

/// Creates a new JavaScript environment
pub fn create_js_environment(name: &str, path: &str) -> Result<(), String> {
    let mut manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.create_js_env(name, PathBuf::from(path))
}

/// Activates an environment
pub fn activate_environment(name: &str) -> Result<(), String> {
    let mut manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.activate_env(name)
}

/// Deactivates the current environment
pub fn deactivate_environment() -> Result<(), String> {
    let mut manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.deactivate_env()
}

/// Synchronizes an environment with its dependencies
pub fn sync_environment(name: &str) -> Result<(), String> {
    let manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.sync_env(name)
}

/// Checks if an environment exists
pub fn environment_exists(name: &str) -> bool {
    let manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.get_env(name).is_some()
}

/// Lists all registered environments
pub fn list_environments() -> Vec<String> {
    let manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.list_envs()
}

/// Adds a dependency to an environment
pub fn add_dependency_to_environment(env_name: &str, dependency: &str) -> Result<(), String> {
    let mut manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.add_dependency(env_name, dependency.to_string())
}

/// Checks if Python is available in the system
pub fn is_python_available() -> bool {
    let manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.is_python_available()
}

/// Checks if Go is available in the system
pub fn is_go_available() -> bool {
    let manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.is_go_available()
}

/// Checks if Node.js is available in the system
pub fn is_node_available() -> bool {
    let manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.is_node_available()
}

/// Checks if Logos is available in the system
pub fn is_logos_available() -> bool {
    let manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.is_logos_available()
}

/// Creates a new Logos environment
pub fn create_logos_environment(name: &str, path: &str) -> Result<(), String> {
    let mut manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.create_logos_env(name, PathBuf::from(path))
}

/// Creates a new Rust environment
pub fn create_rust_environment(name: &str, path: &str) -> Result<(), String> {
    let mut manager = GLOBAL_ENV_MANAGER.lock().unwrap();
    manager.create_rust_env(name, PathBuf::from(path))
}

