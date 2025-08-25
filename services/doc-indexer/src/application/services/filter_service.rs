/// File filtering service for document indexing operations
/// 
/// This service handles safe lists (allow lists) and ignore lists (deny lists)
/// for controlling which files and directories should be indexed.

use std::path::Path;

/// File filtering patterns and configuration
#[derive(Debug, Clone)]
pub struct IndexingFilters {
    /// Safe list patterns - only files/dirs matching these patterns will be indexed
    /// If empty, all files are allowed (subject to ignore list)
    pub safe_list: Vec<String>,
    
    /// Ignore list patterns - files/dirs matching these patterns will be skipped
    pub ignore_list: Vec<String>,
    
    /// Whether to use case-sensitive matching
    pub case_sensitive: bool,
    
    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
}

impl IndexingFilters {
    /// Create new filtering configuration with default settings
    pub fn new() -> Self {
        Self {
            safe_list: Vec::new(),
            ignore_list: Self::default_ignore_patterns(),
            case_sensitive: false,
            follow_symlinks: false,
        }
    }
    
    /// Create filters with custom patterns
    pub fn with_patterns(safe_list: Vec<String>, ignore_list: Vec<String>) -> Self {
        Self {
            safe_list,
            ignore_list,
            case_sensitive: false,
            follow_symlinks: false,
        }
    }
    
    /// Default ignore patterns for common files that shouldn't be indexed
    fn default_ignore_patterns() -> Vec<String> {
        vec![
            // Version control
            ".git".to_string(),
            ".gitignore".to_string(),
            ".gitmodules".to_string(),
            ".svn".to_string(),
            
            // Build artifacts and dependencies
            "target".to_string(),
            "build".to_string(),
            "dist".to_string(),
            "node_modules".to_string(),
            "__pycache__".to_string(),
            "*.pyc".to_string(),
            "*.pyo".to_string(),
            "*.class".to_string(),
            "*.o".to_string(),
            "*.obj".to_string(),
            "*.so".to_string(),
            "*.dll".to_string(),
            "*.dylib".to_string(),
            
            // Package managers
            "Cargo.lock".to_string(),
            "package-lock.json".to_string(),
            "yarn.lock".to_string(),
            "poetry.lock".to_string(),
            "Pipfile.lock".to_string(),
            
            // IDE and editor files
            ".vscode".to_string(),
            ".idea".to_string(),
            "*.swp".to_string(),
            "*.swo".to_string(),
            "*~".to_string(),
            ".DS_Store".to_string(),
            "Thumbs.db".to_string(),
            
            // Temporary and cache files
            "*.tmp".to_string(),
            "*.temp".to_string(),
            "*.cache".to_string(),
            ".cache".to_string(),
            
            // Log files
            "*.log".to_string(),
            "logs".to_string(),
            
            // Archive files (usually don't contain searchable text)
            "*.zip".to_string(),
            "*.tar".to_string(),
            "*.tar.gz".to_string(),
            "*.tgz".to_string(),
            "*.rar".to_string(),
            "*.7z".to_string(),
            
            // Binary and media files
            "*.exe".to_string(),
            "*.bin".to_string(),
            "*.img".to_string(),
            "*.iso".to_string(),
            "*.jpg".to_string(),
            "*.jpeg".to_string(),
            "*.png".to_string(),
            "*.gif".to_string(),
            "*.bmp".to_string(),
            "*.ico".to_string(),
            "*.svg".to_string(),
            "*.mp3".to_string(),
            "*.mp4".to_string(),
            "*.avi".to_string(),
            "*.mov".to_string(),
            "*.wmv".to_string(),
            "*.flv".to_string(),
            "*.webm".to_string(),
        ]
    }
    
    /// Add patterns to the safe list
    pub fn add_safe_patterns(&mut self, patterns: Vec<String>) {
        self.safe_list.extend(patterns);
    }
    
    /// Add patterns to the ignore list
    pub fn add_ignore_patterns(&mut self, patterns: Vec<String>) {
        self.ignore_list.extend(patterns);
    }
    
    /// Remove all patterns from safe list
    pub fn clear_safe_list(&mut self) {
        self.safe_list.clear();
    }
    
    /// Remove all patterns from ignore list
    pub fn clear_ignore_list(&mut self) {
        self.ignore_list.clear();
    }
    
    /// Set case sensitivity for pattern matching
    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.case_sensitive = case_sensitive;
    }
    
    /// Set whether to follow symbolic links
    pub fn set_follow_symlinks(&mut self, follow_symlinks: bool) {
        self.follow_symlinks = follow_symlinks;
    }
}

impl Default for IndexingFilters {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for filtering files and directories during indexing
pub struct FilterService {
    filters: IndexingFilters,
}

impl FilterService {
    /// Create a new filter service with the given configuration
    pub fn new(filters: IndexingFilters) -> Self {
        Self { filters }
    }
    
    /// Check if a path should be indexed based on the filtering rules
    /// 
    /// Returns true if the path should be indexed, false if it should be skipped.
    /// 
    /// Logic:
    /// 1. If ignore list matches, skip regardless of safe list
    /// 2. If safe list is empty, allow (unless ignored)
    /// 3. If safe list is not empty, only allow if it matches safe list
    pub fn should_index(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        
        tracing::debug!("Checking if should index: {} (file_name: {})", path.display(), file_name);
        tracing::debug!("Safe list size: {}, Ignore list size: {}", self.filters.safe_list.len(), self.filters.ignore_list.len());
        
        // Check ignore list first - if matched, always skip
        if self.matches_patterns(&path_str, &file_name, &self.filters.ignore_list) {
            tracing::debug!("Skipping (ignored): {}", path.display());
            return false;
        }
        
        // If safe list is empty, allow everything (that wasn't ignored)
        if self.filters.safe_list.is_empty() {
            tracing::debug!("Allowing (safe list empty): {}", path.display());
            return true;
        }
        
        // If safe list is not empty, only allow if it matches
        if self.matches_patterns(&path_str, &file_name, &self.filters.safe_list) {
            tracing::debug!("Allowing (safe listed): {}", path.display());
            true
        } else {
            tracing::debug!("Skipping (not in safe list): {}", path.display());
            false
        }
    }
    
    /// Check if a path should be followed for directory traversal
    /// 
    /// This is separate from should_index to allow traversing directories
    /// that might contain indexable files, even if the directory itself
    /// wouldn't be indexed.
    pub fn should_traverse(&self, path: &Path) -> bool {
        // Don't traverse if it's explicitly ignored
        if !self.should_index(path) {
            return false;
        }
        
        // Check symlink policy
        if path.is_symlink() && !self.filters.follow_symlinks {
            tracing::debug!("Skipping symlink (policy): {}", path.display());
            return false;
        }
        
        true
    }
    
    /// Check if a path matches any pattern in the given list
    fn matches_patterns(&self, path_str: &str, file_name: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if self.matches_pattern(path_str, file_name, pattern) {
                tracing::debug!("Pattern '{}' matched file: {}", pattern, path_str);
                return true;
            }
        }
        false
    }
    
    /// Check if a path matches a specific pattern
    /// 
    /// Supports:
    /// - Exact matches
    /// - Glob patterns with * and ?
    /// - Directory names (matches if any path component matches)
    fn matches_pattern(&self, path_str: &str, file_name: &str, pattern: &str) -> bool {
        let (path_to_check, pattern_to_check) = if self.filters.case_sensitive {
            (path_str, pattern)
        } else {
            // Create temporary lowercase strings for comparison
            let path_lower = path_str.to_lowercase();
            let pattern_lower = pattern.to_lowercase();
            // We need to return references, so we'll do case-insensitive comparison inline
            return self.matches_pattern_case_insensitive(&path_lower, file_name, &pattern_lower);
        };
        
        // Check if pattern matches the file name exactly
        if glob_match(file_name, pattern_to_check) {
            return true;
        }
        
        // Check if pattern matches any path component
        for component in path_to_check.split('/').chain(path_to_check.split('\\')) {
            if glob_match(component, pattern_to_check) {
                return true;
            }
        }
        
        // Check if pattern matches the full path
        glob_match(path_to_check, pattern_to_check)
    }
    
    /// Case-insensitive version of matches_pattern
    fn matches_pattern_case_insensitive(&self, path_lower: &str, file_name: &str, pattern_lower: &str) -> bool {
        let file_name_lower = file_name.to_lowercase();
        
        // Check if pattern matches the file name exactly
        if glob_match(&file_name_lower, pattern_lower) {
            return true;
        }
        
        // Check if pattern matches any path component
        for component in path_lower.split('/').chain(path_lower.split('\\')) {
            if glob_match(component, pattern_lower) {
                return true;
            }
        }
        
        // Check if pattern matches the full path
        glob_match(path_lower, pattern_lower)
    }
    
    /// Get the current filter configuration
    pub fn filters(&self) -> &IndexingFilters {
        &self.filters
    }
    
    /// Update the filter configuration
    pub fn set_filters(&mut self, filters: IndexingFilters) {
        self.filters = filters;
    }
}

/// Simple glob pattern matching
/// 
/// Supports:
/// - * matches any sequence of characters (except path separators)
/// - ? matches any single character (except path separators)
/// - Exact string matching
fn glob_match(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    
    // Use a simple recursive approach instead of iterators
    match_recursive(text, pattern)
}

/// Recursive helper for glob matching
fn match_recursive(text: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }
    
    if text.is_empty() {
        return pattern.chars().all(|c| c == '*');
    }
    
    let mut pattern_chars = pattern.chars();
    if let Some(pattern_char) = pattern_chars.next() {
        let remaining_pattern = pattern_chars.as_str();
        
        match pattern_char {
            '*' => {
                // Try matching with different amounts of text consumed
                for i in 0..=text.len() {
                    if match_recursive(&text[i..], remaining_pattern) {
                        return true;
                    }
                }
                false
            }
            '?' => {
                if text.len() == 0 {
                    false
                } else {
                    let mut text_chars = text.chars();
                    text_chars.next(); // consume one character
                    match_recursive(text_chars.as_str(), remaining_pattern)
                }
            }
            _ => {
                let mut text_chars = text.chars();
                if text_chars.next() == Some(pattern_char) {
                    match_recursive(text_chars.as_str(), remaining_pattern)
                } else {
                    false
                }
            }
        }
    } else {
        text.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_glob_match() {
        assert!(glob_match("hello.txt", "*.txt"));
        assert!(glob_match("hello.txt", "hello.*"));
        assert!(glob_match("hello.txt", "h?llo.txt"));
        assert!(glob_match("hello.txt", "hello.txt"));
        assert!(!glob_match("hello.txt", "*.rs"));
        assert!(!glob_match("hello.txt", "goodbye.*"));
    }
    
    #[test]
    fn test_default_filters() {
        let filters = IndexingFilters::new();
        let service = FilterService::new(filters);
        
        // Should ignore git directories
        assert!(!service.should_index(Path::new(".git")));
        assert!(!service.should_index(Path::new("project/.git")));
        
        // Should ignore build artifacts
        assert!(!service.should_index(Path::new("target")));
        assert!(!service.should_index(Path::new("node_modules")));
        
        // Should allow regular source files
        assert!(service.should_index(Path::new("src/main.rs")));
        assert!(service.should_index(Path::new("README.md")));
    }
    
    #[test]
    fn test_safe_list() {
        let mut filters = IndexingFilters::new();
        filters.clear_ignore_list(); // Remove default ignores for this test
        filters.add_safe_patterns(vec!["*.rs".to_string(), "*.md".to_string()]);
        
        let service = FilterService::new(filters);
        
        // Should allow files matching safe list
        assert!(service.should_index(Path::new("main.rs")));
        assert!(service.should_index(Path::new("README.md")));
        
        // Should reject files not matching safe list
        assert!(!service.should_index(Path::new("main.py")));
        assert!(!service.should_index(Path::new("config.json")));
    }
    
    #[test]
    fn test_ignore_overrides_safe_list() {
        let mut filters = IndexingFilters::new();
        filters.clear_ignore_list(); // Start clean
        filters.add_safe_patterns(vec!["*.rs".to_string()]);
        filters.add_ignore_patterns(vec!["*test*.rs".to_string()]);
        
        let service = FilterService::new(filters);
        
        // Should allow regular .rs files
        assert!(service.should_index(Path::new("main.rs")));
        
        // Should ignore test files even though they match safe list
        assert!(!service.should_index(Path::new("test_main.rs")));
        assert!(!service.should_index(Path::new("main_test.rs")));
    }
}
