use std::process::Command;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub name: String,
    pub version: Option<String>,
    pub installed: bool,
    pub package_manager: Option<String>,
}

pub struct LanguageDetector;

impl LanguageDetector {
    pub fn new() -> Self {
        LanguageDetector
    }

    /// Detect all available programming languages on the system
    pub fn detect_languages(&self) -> Vec<LanguageInfo> {
        let mut detected_langs = Vec::new();
        
        // Common languages to detect
        let languages = [
            ("python", "Python", "--version"),
            ("python3", "Python", "--version"),
            ("node", "JavaScript", "--version"),
            ("npm", "JavaScript", "--version"),
            ("javac", "Java", "-version"),
            ("java", "Java", "-version"),
            ("go", "Go", "version"),
            ("rustc", "Rust", "--version"),
            ("cargo", "Rust", "--version"),
            ("gcc", "C", "--version"),
            ("g++", "C++", "--version"),
            ("ruby", "Ruby", "--version"),
            ("perl", "Perl", "--version"),
            ("php", "PHP", "--version"),
            ("lua", "Lua", "--version"),
            ("swift", "Swift", "--version"),
            ("kotlin", "Kotlin", "-version"),
            ("scala", "Scala", "-version"),
            ("typescript", "TypeScript", "--version"),
            ("deno", "Deno", "--version"),
        ];

        for (cmd, lang_name, version_flag) in languages.iter() {
            if let Ok(version) = self.check_command_version(cmd, version_flag) {
                detected_langs.push(LanguageInfo {
                    name: lang_name.to_string(),
                    version: Some(version),
                    installed: true,
                    package_manager: self.get_package_manager(lang_name),
                });
            }
        }

        detected_langs
    }

    /// Check if a specific command exists and get its version
    fn check_command_version(&self, cmd: &str, version_flag: &str) -> Result<String, String> {
        let output = Command::new(cmd)
            .arg(version_flag)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let version_output = String::from_utf8_lossy(&output.stderr);
                    if version_output.is_empty() {
                        let version_output = String::from_utf8_lossy(&output.stdout);
                        Ok(version_output.trim().to_string())
                    } else {
                        Ok(version_output.trim().to_string())
                    }
                } else {
                    Err("Command failed".to_string())
                }
            }
            Err(_) => Err("Command not found".to_string()),
        }
    }

    /// Get the associated package manager for a language
    fn get_package_manager(&self, language: &str) -> Option<String> {
        match language {
            "Python" => Some("pip/pip3".to_string()),
            "JavaScript" => Some("npm/yarn".to_string()),
            "Java" => Some("Maven/Gradle".to_string()),
            "Rust" => Some("Cargo".to_string()),
            "Go" => Some("Go Modules".to_string()),
            "Ruby" => Some("Gem".to_string()),
            "PHP" => Some("Composer".to_string()),
            _ => None,
        }
    }

    /// Check if a specific language is available
    pub fn is_language_available(&self, language_cmd: &str) -> bool {
        Command::new(language_cmd)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
    }

    /// Get a list of supported languages
    pub fn get_supported_languages() -> Vec<&'static str> {
        vec![
            "Python", "JavaScript", "Java", "Rust", "Go", "C", "C++", 
            "Ruby", "PHP", "Lua", "Swift", "Kotlin", "Scala", "TypeScript", "Deno"
        ]
    }
}

/// Global instance for language detection
pub static LANGUAGE_DETECTOR: LanguageDetector = LanguageDetector;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detector_creation() {
        let detector = LanguageDetector::new();
        assert!(true); // Just test that we can create it
    }

    #[test]
    fn test_get_supported_languages() {
        let langs = LanguageDetector::get_supported_languages();
        assert!(langs.contains(&"Python"));
        assert!(langs.contains(&"JavaScript"));
        assert!(langs.contains(&"Rust"));
    }
}