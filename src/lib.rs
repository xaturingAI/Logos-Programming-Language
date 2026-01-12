// Logos Programming Language Library
// This library provides the core functionality for the Logos programming language
// including parsing, compilation, execution, and multi-language integration.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod codegen;
pub mod optimizer;
pub mod type_checker;
pub mod type_system;
pub mod modules;
pub mod effects;
pub mod trait_system;
pub mod actor_model;
pub mod garbage_collector;
pub mod gc;
pub mod memory;
pub mod memory_safety;
pub mod concurrency;
pub mod intelligence;
pub mod lang_detection;
pub mod debug_tokens;
pub mod decoder;
pub mod env_manager;
pub mod debugger;
pub mod ffi;
pub mod std_lib;
pub mod macros;
pub mod multilang_integration;
pub mod package_manager;
pub mod multilang_smart_ptrs;
pub mod performance_optimizer;
pub mod enhanced_ffi;
pub mod error_handler;
pub mod shell;
/// GUI module providing cross-platform graphical user interface capabilities
/// with support for both Wayland and X11 (Xorg) display servers
pub mod gui;
/// Networking module providing HTTP, WebSocket, and other networking capabilities
pub mod networking;
pub mod memory_management {
    pub mod ownership;
    pub mod safety_without_gc;
    pub mod option;
    pub mod immutability;
    pub mod advanced_ownership;
}


use std::process::Command;

/// Checks if the source code contains multi-language annotations like @python{...}, @go{...}, etc.
fn contains_multilang_annotations(source: &str) -> bool {
    // Look for the pattern @language{...} where language is a known language
    let multilang_patterns = [
        "@python{",
        "@go{",
        "@rust{",
        "@js{",
        "@javascript{",
        "@java{",
        "@cpp{",
        "@c{",
        "@csharp{",
        "@php{",
        "@ruby{",
        "@swift{",
        "@kotlin{",
        "@scala{",
        "@typescript{",
        "@deno{",
    ];

    for pattern in &multilang_patterns {
        if source.contains(pattern) {
            return true;
        }
    }

    false
}

// Re-export commonly used modules
pub use ast::*;
pub use lexer::*;
pub use parser::*;
pub use type_system::*;
pub use modules::*;
pub use decoder::*;

/// Defines the level of multi-language support to use during operations
#[derive(Debug, Clone, PartialEq)]
pub enum MultiLangSupport {
    Go,        // Use Go for analysis and processing
    Python,    // Use Python for analysis and processing
    Both,      // Use both Go and Python for comprehensive analysis
    RustOnly,  // Use only Rust-based processing
}

/// Contains the results of multi-language analysis
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub go_result: Option<String>,                    // Result from Go analysis
    pub python_result: Option<String>,               // Result from Python analysis
    pub rust_result: Option<String>,                 // Result from Rust analysis
    pub combined_result: String,                     // Combined analysis result
    pub performance_metrics: HashMap<String, String>, // Performance metrics collected during analysis
}

/// Executes Logos source code by parsing, analyzing, and running it
///
/// # Arguments
/// * `source` - The Logos source code to execute
///
/// # Returns
/// * `Ok(())` if execution was successful
/// * `Err` with error details if execution failed
pub fn execute(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the source contains multi-language annotations (@python{}, @go{}, etc.)
    let has_multilang_annotations = contains_multilang_annotations(source);

    if has_multilang_annotations {
        // If multi-language annotations are present, use multi-language processing
        println!("Multi-language annotations detected, using multi-language processing...");
        let analysis = analyze_with_multilang(source, MultiLangSupport::Both)?;
        println!("Execution analysis: {}", analysis.combined_result);
    } else {
        // For efficiency, skip multi-language analysis if no annotations are present
        println!("No multi-language annotations detected, using Rust-only processing...");
    }

    // Parse the source code into an AST
    let mut parser = parser::Parser::new(source);
    let ast = parser.parse_program()?;

    // Execute the AST using the runtime
    let mut runtime = crate::runtime::Runtime::new();
    runtime.eval_program(&ast)?;

    Ok(())
}

/// Compiles Logos source code to executable format with multi-language processing
///
/// # Arguments
/// * `source` - The Logos source code to compile
///
/// # Returns
/// * `Ok(String)` containing the compiled code if successful
/// * `Err` with error details if compilation failed
pub fn compile(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Check if the source contains multi-language annotations (@python{}, @go{}, etc.)
    let has_multilang_annotations = contains_multilang_annotations(source);

    decoder::init_decoder()?;

    let support_level = if has_multilang_annotations {
        // Only use multi-language support if the code contains annotations
        println!("Multi-language annotations detected, using multi-language processing...");
        MultiLangSupport::Both
    } else {
        // Use Rust-only processing for efficiency if no annotations are present
        println!("No multi-language annotations detected, using Rust-only processing...");
        MultiLangSupport::RustOnly
    };

    let analysis = analyze_with_multilang(source, support_level.clone())?;
    if has_multilang_annotations {
        println!("Compilation analysis: {}", analysis.combined_result);
    }

    let mut parser = parser::Parser::new(source);
    let ast = parser.parse_program()?;
    let code = generate_code_with_multilang(&ast, support_level.clone())?;
    let optimized_code = optimize_with_multilang(&code, support_level)?;

    Ok(optimized_code)
}

/// Performs multi-language analysis of the provided source code
///
/// # Arguments
/// * `source` - The source code to analyze
/// * `support` - The level of multi-language support to use
///
/// # Returns
/// * `Ok(AnalysisResult)` containing the analysis results if successful
/// * `Err` with error details if analysis failed
pub fn analyze_with_multilang(source: &str, support: MultiLangSupport) -> Result<AnalysisResult, Box<dyn std::error::Error>> {
    let mut go_result = None;
    let mut python_result = None;
    let mut combined_result = String::new();
    let mut performance_metrics = HashMap::new();

    match support {
        MultiLangSupport::Go => {
            go_result = Some(decoder::LogosDecoder::parse_with_go(source)?);
            combined_result = format!("Go Analysis: {}", go_result.as_ref().unwrap());
        },
        MultiLangSupport::Python => {
            python_result = Some(decoder::LogosDecoder::parse_with_python(source).map_err(|e| format!("Python error: {:?}", e))?);
            combined_result = format!("Python Analysis: {}", python_result.as_ref().unwrap());
        },
        MultiLangSupport::Both => {
            go_result = Some(decoder::LogosDecoder::parse_with_go(source)?);
            python_result = Some(decoder::LogosDecoder::parse_with_python(source).map_err(|e| format!("Python error: {:?}", e))?);

            combined_result = format!(
                "Go Result: {}\nPython Result: {}",
                go_result.as_ref().unwrap(),
                python_result.as_ref().unwrap()
            );

            let go_analysis = decoder::LogosDecoder::analyze_with_go(source)?;
            let python_analysis = decoder::LogosDecoder::analyze_with_python(source).map_err(|e| format!("Python error: {:?}", e))?;

            combined_result.push_str(&format!("\nGo Performance: {}\nPython Performance: {}", go_analysis, python_analysis));
        },
        MultiLangSupport::RustOnly => {
            // Use pure Rust analysis when no multi-language support is needed
            let rust_analysis = decoder::LogosDecoder::parse_with_rust(source)?;
            combined_result = format!("Pure Rust analysis: {}", rust_analysis);
        }
    }

    performance_metrics.insert("source_length".to_string(), source.len().to_string());
    performance_metrics.insert("line_count".to_string(), source.lines().count().to_string());

    let rust_result = if support == MultiLangSupport::RustOnly {
        Some(combined_result.clone()) // Use the Rust analysis result
    } else {
        None
    };

    Ok(AnalysisResult {
        go_result,
        python_result,
        rust_result,
        combined_result,
        performance_metrics,
    })
}

/// Optimizes code using multi-language techniques
/// 
/// # Arguments
/// * `source` - The source code to optimize
/// * `support` - The level of multi-language support to use
/// 
/// # Returns
/// * `Ok(String)` containing the optimized code if successful
/// * `Err` with error details if optimization failed
pub fn optimize_with_multilang(source: &str, support: MultiLangSupport) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = source.to_string();

    match support {
        MultiLangSupport::Go => {
            result = decoder::LogosDecoder::optimize_with_go(source)?;
        },
        MultiLangSupport::Python => {
            result = decoder::LogosDecoder::optimize_with_python(source).map_err(|e| format!("Python error: {:?}", e))?;
        },
        MultiLangSupport::Both => {
            result = decoder::LogosDecoder::optimize_with_go(source)?;
            result = decoder::LogosDecoder::optimize_with_python(&result).map_err(|e| format!("Python error: {:?}", e))?;
        },
        MultiLangSupport::RustOnly => {
            result = optimize(result);
        }
    }

    Ok(result)
}

/// Generates code using multi-language techniques
/// 
/// # Arguments
/// * `ast` - The abstract syntax tree to generate code from
/// * `support` - The level of multi-language support to use
/// 
/// # Returns
/// * `Ok(String)` containing the generated code if successful
/// * `Err` with error details if code generation failed
pub fn generate_code_with_multilang(ast: &Program, support: MultiLangSupport) -> Result<String, Box<dyn std::error::Error>> {
    let basic_code = generate_code(&format!("{:?}", ast))?;

    match support {
        MultiLangSupport::Go => {
            let processed = decoder::LogosDecoder::parse_with_go(&basic_code)?;
            Ok(processed)
        },
        MultiLangSupport::Python => {
            let processed = decoder::LogosDecoder::parse_with_python(&basic_code).map_err(|e| format!("Python error: {:?}", e))?;
            Ok(processed)
        },
        MultiLangSupport::Both => {
            let go_processed = decoder::LogosDecoder::parse_with_go(&basic_code)?;
            let python_processed = decoder::LogosDecoder::parse_with_python(&go_processed).map_err(|e| format!("Python error: {:?}", e))?;
            Ok(python_processed)
        },
        MultiLangSupport::RustOnly => {
            Ok(basic_code)
        }
    }
}

/// Checks syntax and types with multi-language validation
/// 
/// # Arguments
/// * `source` - The source code to check
/// * `support` - The level of multi-language support to use
/// 
/// # Returns
/// * `Ok(())` if syntax and type checking was successful
/// * `Err` with error details if checking failed
pub fn check_syntax_and_types_with_multilang(source: &str, support: MultiLangSupport) -> Result<(), Box<dyn std::error::Error>> {
    check_syntax_and_types(source)?;

    match support {
        MultiLangSupport::Go => {
            let validation = decoder::LogosDecoder::validate_with_go(source)?;
            println!("Go validation: {}", validation);
        },
        MultiLangSupport::Python => {
            let validation = decoder::LogosDecoder::validate_with_python(source).map_err(|e| format!("Python error: {:?}", e))?;
            println!("Python validation: {}", validation);
        },
        MultiLangSupport::Both => {
            let go_validation = decoder::LogosDecoder::validate_with_go(source)?;
            let python_validation = decoder::LogosDecoder::validate_with_python(source).map_err(|e| format!("Python error: {:?}", e))?;
            println!("Go validation: {}", go_validation);
            println!("Python validation: {}", python_validation);
        },
        MultiLangSupport::RustOnly => {
        }
    }

    Ok(())
}

/// Performs basic optimization on the provided program
/// 
/// # Arguments
/// * `program` - The program to optimize
/// 
/// # Returns
/// The optimized program
pub fn optimize(program: String) -> String {
    let _analysis = decoder::LogosDecoder::full_analysis(&program).unwrap_or_else(|_| "Analysis failed".to_string());
    program
}

/// Generates code from the provided program representation
/// 
/// # Arguments
/// * `program` - The program representation to generate code from
/// 
/// # Returns
/// * `Ok(String)` containing the generated code if successful
/// * `Err` with error details if code generation failed
pub fn generate_code(program: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(format!("// Generated code from: {}\n{}", program, program))
}

/// Performs basic syntax and type checking on the provided source
/// 
/// # Arguments
/// * `source` - The source code to check
/// 
/// # Returns
/// * `Ok(())` if syntax and type checking was successful
/// * `Err` with error details if checking failed
pub fn check_syntax_and_types(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    // First perform lexical analysis
    let tokens = lexer::tokenize(source)?;

    // Then parse the tokens
    let mut parser = parser::Parser::new(source);
    let program = parser.parse_program()?;

    // Finally, perform type checking
    type_checker::check_types(&program)?;

    Ok(())
}

/// Transpiles Logos code to Python
///
/// # Arguments
/// * `source` - The Logos source code to transpile
///
/// # Returns
/// * `Ok(String)` containing the Python equivalent if successful
/// * `Err` with error details if transpilation failed
#[cfg(feature = "python")]
pub fn transpile_to_python(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    decoder::init_decoder()?;

    let result = Python::with_gil(|py| -> PyResult<String> {
        let processor_module = PyModule::from_code(
            py,
            r#"from logos_processor import transpile_to_python
def transpile(code):
    return transpile_to_python(code)"#,
            "transpiler.py",
            "transpiler"
        )?;

        let result = processor_module
            .getattr("transpile")?
            .call1((source,))?
            .extract::<String>()?;

        Ok(result)
    }).map_err(|e| format!("Python transpilation error: {:?}", e))?;

    Ok(result)
}

/// Formats Logos code using multi-language techniques
///
/// # Arguments
/// * `source` - The source code to format
/// * `support` - The level of multi-language support to use
///
/// # Returns
/// * `Ok(String)` containing the formatted code if successful
/// * `Err` with error details if formatting failed
#[cfg(feature = "python")]
pub fn format_code(source: &str, support: MultiLangSupport) -> Result<String, Box<dyn std::error::Error>> {
    decoder::init_decoder()?;

    match support {
        MultiLangSupport::Python => {
            let result = Python::with_gil(|py| -> PyResult<String> {
                let processor_module = PyModule::from_code(
                    py,
                    r#"from logos_processor import format_logos_code
def format_code(code):
    return format_logos_code(code)"#,
                    "formatter.py",
                    "formatter"
                )?;

                let result = processor_module
                    .getattr("format_code")?
                    .call1((source,))?
                    .extract::<String>()?;

                Ok(result)
            }).map_err(|e| format!("Python formatting error: {:?}", e))?;

            Ok(result)
        },
        _ => {
            transpile_to_python(source)
        }
    }
}

/// Initializes multi-language support for the Logos compiler
/// 
/// # Returns
/// * `Ok(())` if initialization was successful
/// * `Err` with error details if initialization failed
pub fn init_multilang_support() -> Result<(), Box<dyn std::error::Error>> {
    decoder::init_decoder()?;
    println!("Multi-language support initialized");
    Ok(())
}

/// Checks if Go is available in the system
/// 
/// # Returns
/// `true` if Go is available, `false` otherwise
pub fn is_go_available() -> bool {
    match Command::new("go").arg("version").output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Checks if Python is available in the system
/// 
/// # Returns
/// `true` if Python is available, `false` otherwise
#[cfg(feature = "python")]
pub fn is_python_available() -> bool {
    match Command::new("python3").arg("--version").output() {
        Ok(output) => output.status.success(),
        Err(_) => {
            match Command::new("python").arg("--version").output() {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }
    }
}

#[cfg(feature = "python")]
pub use pyo3;

// Re-export for main binary as logos_lang
pub mod logos_lang {
    pub use crate::ast::*;
    pub use crate::lexer::*;
    pub use crate::parser::*;
    pub use crate::decoder;
    pub use crate::decoder::LogosDecoder;
    pub use crate::execute;
    pub use crate::compile;
    pub use crate::analyze_with_multilang;
    pub use crate::optimize_with_multilang;
    pub use crate::generate_code_with_multilang;
    pub use crate::check_syntax_and_types_with_multilang;
    pub use crate::optimize;
    pub use crate::generate_code;
    pub use crate::check_syntax_and_types;
    #[cfg(feature = "python")]
    pub use crate::transpile_to_python;
    #[cfg(feature = "python")]
    pub use crate::format_code;
    pub use crate::init_multilang_support;
    pub use crate::is_go_available;
    #[cfg(feature = "python")]
    pub use crate::is_python_available;
    pub use crate::env_manager::is_logos_available;
    pub use crate::env_manager::create_logos_environment;
    pub use crate::gui::{Window, Application, init_gui};
}

// Enhanced multi-library support module
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::Path;
// use lazy_static::lazy_static;  // Commented out to avoid linking issues

/// Library management module for handling external libraries
pub mod library_manager {
    use super::*;

    /// Metadata for a library including name, version, and dependencies
    #[derive(Debug, Clone)]
    pub struct LibraryMetadata {
        pub name: String,
        pub version: String,
        pub description: String,
        pub authors: Vec<String>,
        pub dependencies: Vec<String>,
        pub license: String,
        pub repository: Option<String>,
        pub homepage: Option<String>,
    }

    /// Represents a loaded library with its metadata and path
    #[derive(Debug, Clone)]
    pub struct Library {
        pub metadata: LibraryMetadata,
        pub path: String,
        pub loaded: bool,
    }

    /// Thread-safe library manager for registering and retrieving libraries
    pub struct LibraryManager {
        libraries: Arc<Mutex<HashMap<String, Library>>>,
    }

    impl LibraryManager {
        /// Creates a new library manager instance
        pub fn new() -> Self {
            Self {
                libraries: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        /// Registers a library in the manager
        /// 
        /// # Arguments
        /// * `library` - The library to register
        /// 
        /// # Returns
        /// * `Ok(())` if registration was successful
        /// * `Err` with error details if registration failed
        pub fn register_library(&self, library: Library) -> Result<(), String> {
            let mut libs = self.libraries.lock()
                .map_err(|e| format!("Poisoned lock: {}", e))?;

            libs.insert(library.metadata.name.clone(), library);
            Ok(())
        }

        /// Retrieves a library by name
        /// 
        /// # Arguments
        /// * `name` - The name of the library to retrieve
        /// 
        /// # Returns
        /// The library if found, None otherwise
        pub fn get_library(&self, name: &str) -> Option<Library> {
            let libs = self.libraries.lock()
                .expect("Poisoned lock");
            libs.get(name).cloned()
        }

        /// Lists all registered library names
        /// 
        /// # Returns
        /// A vector containing the names of all registered libraries
        pub fn list_libraries(&self) -> Vec<String> {
            let libs = self.libraries.lock()
                .expect("Poisoned lock");
            libs.keys().cloned().collect()
        }

        /// Loads a library from the specified path
        /// 
        /// # Arguments
        /// * `path` - The path to load the library from
        /// 
        /// # Returns
        /// * `Ok(Library)` containing the loaded library if successful
        /// * `Err` with error details if loading failed
        pub fn load_library_from_path<P: AsRef<Path>>(&self, path: P) -> Result<Library, String> {
            // In a real implementation, this would load a library from the given path
            // For now, we'll create a mock library
            let path_str = path.as_ref().to_str()
                .ok_or("Invalid path")?
                .to_string();

            let metadata = LibraryMetadata {
                name: format!("lib_{}", path_str.replace("/", "_").replace(".", "_")),
                version: "0.1.0".to_string(),
                description: "Dynamically loaded library".to_string(),
                authors: vec!["Logos Team".to_string()],
                dependencies: vec![],
                license: "MIT".to_string(),
                repository: None,
                homepage: None,
            };

            let library = Library {
                metadata,
                path: path_str,
                loaded: true,
            };

            self.register_library(library.clone())
                .map_err(|e| format!("Failed to register library: {}", e))?;

            Ok(library)
        }
    }

    use std::sync::{Mutex, OnceLock};

    static GLOBAL_MANAGER_INSTANCE: OnceLock<Mutex<LibraryManager>> = OnceLock::new();

    fn get_global_manager() -> &'static Mutex<LibraryManager> {
        GLOBAL_MANAGER_INSTANCE.get_or_init(|| Mutex::new(LibraryManager::new()))
    }

    /// Registers a library in the global manager
    ///
    /// # Arguments
    /// * `library` - The library to register
    ///
    /// # Returns
    /// * `Ok(())` if registration was successful
    /// * `Err` with error details if registration failed
    pub fn register_library(library: Library) -> Result<(), String> {
        let manager = get_global_manager();
        let mut guard = manager.lock().map_err(|e| e.to_string())?;
        guard.register_library(library)
    }

    /// Retrieves a library by name from the global manager
    ///
    /// # Arguments
    /// * `name` - The name of the library to retrieve
    ///
    /// # Returns
    /// The library if found, None otherwise
    pub fn get_library(name: &str) -> Option<Library> {
        let manager = get_global_manager();
        let guard = manager.lock().ok()?;
        match guard.get_library(name) {
            Some(lib) => Some(lib.clone()),
            None => None,
        }
    }

    /// Lists all registered library names in the global manager
    ///
    /// # Returns
    /// A vector containing the names of all registered libraries
    pub fn list_libraries() -> Vec<String> {
        let manager = get_global_manager();
        let guard = manager.lock().unwrap();
        guard.list_libraries()
    }

    /// Loads a library from the specified path in the global manager
    ///
    /// # Arguments
    /// * `path` - The path to load the library from
    ///
    /// # Returns
    /// * `Ok(Library)` containing the loaded library if successful
    /// * `Err` with error details if loading failed
    pub fn load_library_from_path<P: AsRef<Path>>(path: P) -> Result<Library, String> {
        let manager = get_global_manager();
        let mut guard = manager.lock().map_err(|e| e.to_string())?;
        guard.load_library_from_path(path)
    }
}

/// Multi-language library loader for managing libraries in different languages
pub mod multi_lang_loader {
    use super::*;
    use std::process::Command;

    /// Supported programming languages for library loading
    #[derive(Debug)]
    pub enum Language {
        Rust,
        Go,
        Python,
        JavaScript,
        C,
        CPP,
    }

    impl std::fmt::Display for Language {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                Language::Rust => write!(f, "Rust"),
                Language::Go => write!(f, "Go"),
                Language::Python => write!(f, "Python"),
                Language::JavaScript => write!(f, "JavaScript"),
                Language::C => write!(f, "C"),
                Language::CPP => write!(f, "C++"),
            }
        }
    }

    /// Represents a multi-language library with its properties
    #[derive(Debug)]
    pub struct MultiLangLibrary {
        pub name: String,           // Name of the library
        pub language: Language,     // Language the library is written in
        pub path: String,           // Path to the library
        pub exports: Vec<String>,   // List of exported functions/objects
        pub loaded: bool,           // Whether the library is currently loaded
    }

    /// Loader for multi-language libraries
    pub struct MultiLangLoader {
        libraries: Vec<MultiLangLibrary>,  // List of loaded libraries
    }

    impl MultiLangLoader {
        /// Creates a new multi-language loader instance
        pub fn new() -> Self {
            Self {
                libraries: Vec::new(),
            }
        }

        /// Loads a Rust library
        /// 
        /// # Arguments
        /// * `name` - Name of the library
        /// * `path` - Path to the library
        /// 
        /// # Returns
        /// * `Ok(())` if loading was successful
        /// * `Err` with error details if loading failed
        pub fn load_rust_library(&mut self, name: &str, path: &str) -> Result<(), String> {
            // Check if rust library exists and is valid
            if !std::path::Path::new(path).exists() {
                return Err("Rust library path does not exist".to_string());
            }

            let library = MultiLangLibrary {
                name: name.to_string(),
                language: Language::Rust,
                path: path.to_string(),
                exports: vec![], // Would be populated by inspecting the library
                loaded: true,
            };

            self.libraries.push(library);
            Ok(())
        }

        /// Loads a Go library
        /// 
        /// # Arguments
        /// * `name` - Name of the library
        /// * `path` - Path to the library
        /// 
        /// # Returns
        /// * `Ok(())` if loading was successful
        /// * `Err` with error details if loading failed
        pub fn load_go_library(&mut self, name: &str, path: &str) -> Result<(), String> {
            // Check if Go is available
            if !crate::is_go_available() {
                return Err("Go is not available in the system".to_string());
            }

            // Check if the Go library exists
            if !std::path::Path::new(path).exists() {
                return Err("Go library path does not exist".to_string());
            }

            // Attempt to build the Go library
            let output = Command::new("go")
                .args(&["build", path])
                .output()
                .map_err(|e| format!("Failed to execute Go build: {}", e))?;

            if !output.status.success() {
                return Err(format!("Go build failed: {}", String::from_utf8_lossy(&output.stderr)));
            }

            let library = MultiLangLibrary {
                name: name.to_string(),
                language: Language::Go,
                path: path.to_string(),
                exports: vec![], // Would be populated by inspecting the Go code
                loaded: true,
            };

            self.libraries.push(library);
            Ok(())
        }

        /// Loads a Python library
        /// 
        /// # Arguments
        /// * `name` - Name of the library
        /// * `path` - Path to the library
        /// 
        /// # Returns
        /// * `Ok(())` if loading was successful
        /// * `Err` with error details if loading failed
        pub fn load_python_library(&mut self, name: &str, path: &str) -> Result<(), String> {
            #[cfg(feature = "python")]
            {
                // Check if Python is available
                if !crate::is_python_available() {
                    return Err("Python is not available in the system".to_string());
                }

                // Check if the Python library exists
                if !std::path::Path::new(path).exists() {
                    return Err("Python library path does not exist".to_string());
                }
            }

            let library = MultiLangLibrary {
                name: name.to_string(),
                language: Language::Python,
                path: path.to_string(),
                exports: vec![], // Would be populated by inspecting the Python module
                loaded: true,
            };

            self.libraries.push(library);
            Ok(())
        }

        /// Gets a reference to all loaded libraries
        /// 
        /// # Returns
        /// A slice containing all loaded libraries
        pub fn get_loaded_libraries(&self) -> &[MultiLangLibrary] {
            &self.libraries
        }

        /// Unloads a library by name
        /// 
        /// # Arguments
        /// * `name` - Name of the library to unload
        /// 
        /// # Returns
        /// * `Ok(())` if unloading was successful
        /// * `Err` with error details if unloading failed
        pub fn unload_library(&mut self, name: &str) -> Result<(), String> {
            let index = self.libraries.iter()
                .position(|lib| lib.name == name)
                .ok_or("Library not found".to_string())?;

            self.libraries.remove(index);
            Ok(())
        }
    }
}

// Export the new modules
pub use library_manager::*;
pub use multi_lang_loader::*;