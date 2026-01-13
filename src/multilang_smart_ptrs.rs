//! Smart pointers and lazy loading for multi-language libraries in Logos
//! Ensures libraries are only loaded when actually needed

use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use std::path::PathBuf;

/// Lazy-loaded multi-language library that only initializes when first accessed
pub struct LazyMultiLangLibrary {
    name: String,
    language: String,
    path: PathBuf,
    loaded_lib: OnceLock<Option<LoadedLibrary>>,
    init_fn: Box<dyn Fn() -> Result<LoadedLibrary, String> + Send + Sync>,
}

/// Represents a loaded library from another language
#[derive(Clone)]
pub enum LoadedLibrary {
    Go(GoLibrary),
    Python(PythonLibrary),
    Rust(RustLibrary),
    JavaScript(JSLibrary),
    C(CLibrary),
    CPP(CPPLibrary),
    Java(JavaLibrary),
}

/// Go library wrapper
#[derive(Clone)]
pub struct GoLibrary {
    pub handle: usize,  // Using usize instead of raw pointer for Clone
    pub functions: HashMap<String, usize>,  // Using usize instead of raw pointer for Clone
    pub loaded: bool,
}

/// Python library wrapper
#[derive(Clone)]
pub struct PythonLibrary {
    pub module: Option<usize>,  // Using usize instead of raw pointer for Clone
    pub loaded: bool,
}

/// Rust library wrapper
#[derive(Clone)]
pub struct RustLibrary {
    pub functions: HashMap<String, String>,  // Using String as placeholder
    pub loaded: bool,
}

/// JavaScript library wrapper
#[derive(Clone)]
pub struct JSLibrary {
    pub engine: Option<usize>,  // Using usize instead of raw pointer for Clone
    pub loaded: bool,
}

/// C library wrapper
#[derive(Clone)]
pub struct CLibrary {
    pub handle: usize,  // Using usize instead of raw pointer for Clone
    pub functions: HashMap<String, usize>,  // Using usize instead of raw pointer for Clone
    pub loaded: bool,
}

/// C++ library wrapper
#[derive(Clone)]
pub struct CPPLibrary {
    pub handle: usize,  // Using usize instead of raw pointer for Clone
    pub functions: HashMap<String, usize>,  // Using usize instead of raw pointer for Clone
    pub loaded: bool,
}

/// Java library wrapper
#[derive(Clone)]
pub struct JavaLibrary {
    pub jvm: Option<usize>,  // Using usize instead of raw pointer for Clone
    pub loaded: bool,
}

impl LazyMultiLangLibrary {
    /// Creates a new lazy-loaded library
    pub fn new<F>(name: String, language: String, path: PathBuf, init_fn: F) -> Self
    where
        F: Fn() -> Result<LoadedLibrary, String> + Send + Sync + 'static,
    {
        LazyMultiLangLibrary {
            name,
            language,
            path,
            loaded_lib: OnceLock::new(),
            init_fn: Box::new(init_fn),
        }
    }
    
    /// Gets the loaded library, initializing it if necessary
    pub fn get_library(&self) -> Result<LoadedLibrary, String> {
        let lib = self.loaded_lib.get_or_init(|| {
            (self.init_fn)()
                .map_err(|e| eprintln!("Failed to load library {}: {}", self.name, e))
                .ok()  // Convert error to None
        });
        
        match lib {
            Some(lib) => Ok(lib.clone()), // Clone the library to return ownership
            None => Err(format!("Failed to load library: {}", self.name))
        }
    }
    
    /// Checks if the library is loaded without loading it
    pub fn is_loaded(&self) -> bool {
        self.loaded_lib.get().is_some()
    }
    
    /// Gets the library name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Gets the library language
    pub fn language(&self) -> &str {
        &self.language
    }
    
    /// Gets the library path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Manager for lazy-loaded multi-language libraries
pub struct MultiLangLibraryManager {
    libraries: HashMap<String, Arc<Mutex<LazyMultiLangLibrary>>>,
    active_calls: Arc<Mutex<HashMap<String, usize>>>,  // Track how often each library is called
}

impl MultiLangLibraryManager {
    /// Creates a new library manager
    pub fn new() -> Self {
        MultiLangLibraryManager {
            libraries: HashMap::new(),
            active_calls: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Registers a new lazy-loaded library
    pub fn register_library<F>(&mut self, name: String, language: String, path: PathBuf, init_fn: F) 
    where
        F: Fn() -> Result<LoadedLibrary, String> + Send + Sync + 'static,
    {
        let library = Arc::new(Mutex::new(LazyMultiLangLibrary::new(name.clone(), language, path, init_fn)));
        self.libraries.insert(name, library);
    }
    
    /// Gets a library reference (doesn't load it yet)
    pub fn get_library_ref(&self, name: &str) -> Option<Arc<Mutex<LazyMultiLangLibrary>>> {
        self.libraries.get(name).cloned()
    }
    
    /// Gets a loaded library (loads it if necessary)
    pub fn get_library(&self, name: &str) -> Result<LoadedLibrary, String> {
        match self.libraries.get(name) {
            Some(lib_mutex) => {
                // Increment call counter
                {
                    let mut calls = self.active_calls.lock().unwrap();
                    *calls.entry(name.to_string()).or_insert(0) += 1;
                }
                
                // Get the library
                let lib = lib_mutex.lock().unwrap();
                lib.get_library()
            },
            None => Err(format!("Library '{}' not found", name))
        }
    }
    
    /// Checks if a library is loaded without loading it
    pub fn is_library_loaded(&self, name: &str) -> bool {
        match self.libraries.get(name) {
            Some(lib_mutex) => {
                let lib = lib_mutex.lock().unwrap();
                lib.is_loaded()
            },
            None => false,
        }
    }
    
    /// Gets statistics about library usage
    pub fn get_library_stats(&self) -> HashMap<String, (bool, usize)> {
        let mut stats = HashMap::new();
        let calls = self.active_calls.lock().unwrap();
        
        for (name, lib_mutex) in &self.libraries {
            let lib = lib_mutex.lock().unwrap();
            let is_loaded = lib.is_loaded();
            let call_count = *calls.get(name).unwrap_or(&0);
            stats.insert(name.clone(), (is_loaded, call_count));
        }
        stats
    }
    
    /// Gets all registered library names
    pub fn get_library_names(&self) -> Vec<String> {
        self.libraries.keys().cloned().collect()
    }
    
    /// Unloads a library if it's loaded
    pub fn unload_library(&mut self, name: &str) -> Result<(), String> {
        // In a real implementation, this would unload the dynamic library
        // For now, we'll just remove it from the active calls
        self.active_calls.lock().unwrap().remove(name);
        Ok(())
    }
}

/// Smart pointer for multi-language library access
pub struct MultiLangPtr<T> {
    library_name: String,
    resource: Arc<Mutex<Option<T>>>,
    loader: Box<dyn Fn() -> Result<T, String> + Send + Sync>,
}

impl<T> MultiLangPtr<T> {
    /// Creates a new multi-language smart pointer
    pub fn new<F>(library_name: String, loader: F) -> Self
    where
        F: Fn() -> Result<T, String> + Send + Sync + 'static,
    {
        MultiLangPtr {
            library_name,
            resource: Arc::new(Mutex::new(None)),
            loader: Box::new(loader),
        }
    }
    
    /// Gets the resource, loading it if necessary
    pub fn get(&self) -> Result<T, String> 
    where
        T: Clone,
    {
        let mut resource_guard = self.resource.lock().unwrap();
        if resource_guard.is_none() {
            *resource_guard = Some((self.loader)()?);
        }
        
        match resource_guard.as_ref() {
            Some(resource) => Ok(resource.clone()),
            None => Err(format!("Resource for library '{}' not available", self.library_name))
        }
    }
    
    /// Checks if the resource is loaded
    pub fn is_loaded(&self) -> bool {
        self.resource.lock().unwrap().is_some()
    }
    
    /// Gets the library name
    pub fn library_name(&self) -> &str {
        &self.library_name
    }
}

/// Global multi-language library manager
use std::sync::RwLock;

static GLOBAL_LIB_MANAGER: std::sync::OnceLock<RwLock<MultiLangLibraryManager>> = std::sync::OnceLock::new();

/// Initializes the global library manager
pub fn init_global_lib_manager() -> Result<(), String> {
    let manager = MultiLangLibraryManager::new();
    GLOBAL_LIB_MANAGER.set(RwLock::new(manager))
        .map_err(|_| "Failed to initialize global library manager".to_string())
}

/// Gets a reference to the global library manager
pub fn get_global_lib_manager() -> Result<std::sync::RwLockReadGuard<'static, MultiLangLibraryManager>, String> {
    match GLOBAL_LIB_MANAGER.get() {
        Some(manager) => Ok(manager.read().map_err(|_| "Could not acquire read lock".to_string())?),
        None => Err("Global library manager not initialized".to_string())
    }
}

/// Gets a mutable reference to the global library manager
pub fn get_global_lib_manager_mut() -> Result<std::sync::RwLockWriteGuard<'static, MultiLangLibraryManager>, String> {
    match GLOBAL_LIB_MANAGER.get() {
        Some(manager) => Ok(manager.write().map_err(|_| "Could not acquire write lock".to_string())?),
        None => Err("Global library manager not initialized".to_string())
    }
}

/// Registers a library with the global manager
pub fn register_library<F>(name: String, language: String, path: PathBuf, init_fn: F) -> Result<(), String>
where
    F: Fn() -> Result<LoadedLibrary, String> + Send + Sync + 'static,
{
    let mut manager = get_global_lib_manager_mut()
        .map_err(|e| format!("Could not get global library manager: {}", e))?;
    
    manager.register_library(name, language, path, init_fn);
    Ok(())
}

/// Gets a library from the global manager
pub fn get_library(name: &str) -> Result<LoadedLibrary, String> {
    let manager = get_global_lib_manager()
        .map_err(|e| format!("Could not get global library manager: {}", e))?;
    
    manager.get_library(name)
}

/// Checks if a library is loaded in the global manager
pub fn is_library_loaded(name: &str) -> bool {
    match get_global_lib_manager() {
        Ok(manager) => manager.is_library_loaded(name),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_multi_lang_library() {
        // Create a mock library that simulates loading
        let mock_lib = LazyMultiLangLibrary::new(
            "test_lib".to_string(),
            "go".to_string(),
            PathBuf::from("/mock/path"),
            || {
                // Simulate loading a library
                Ok(LoadedLibrary::Go(GoLibrary {
                    handle: 12345,
                    functions: HashMap::new(),
                    loaded: true,
                }))
            }
        );

        // Initially not loaded
        assert_eq!(mock_lib.is_loaded(), false);

        // After getting the library, it should be loaded
        assert!(mock_lib.get_library().is_ok());
        assert_eq!(mock_lib.is_loaded(), true);
    }

    #[test]
    fn test_multi_lang_library_manager() {
        let mut manager = MultiLangLibraryManager::new();

        // Register a mock library
        manager.register_library(
            "test_lib".to_string(),
            "go".to_string(),
            PathBuf::from("/mock/path"),
            || {
                Ok(LoadedLibrary::Go(GoLibrary {
                    handle: 12345,
                    functions: HashMap::new(),
                    loaded: true,
                }))
            }
        );

        // Library should not be loaded initially
        assert_eq!(manager.is_library_loaded("test_lib"), false);

        // After getting the library, it should be loaded
        assert!(manager.get_library("test_lib").is_ok());
        assert_eq!(manager.is_library_loaded("test_lib"), true);
    }

    #[test]
    fn test_multi_lang_ptr() {
        let ptr = MultiLangPtr::new("test_lib".to_string(), || {
            Ok("test_resource".to_string())
        });

        // Initially not loaded
        assert_eq!(ptr.is_loaded(), false);

        // After getting the resource, it should be loaded
        assert!(ptr.get().is_ok());
        assert_eq!(ptr.is_loaded(), true);
    }
}