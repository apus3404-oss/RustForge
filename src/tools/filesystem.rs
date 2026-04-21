use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::tools::traits::Tool;
use crate::tools::types::{ParameterType, ToolParameter, ToolResult};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileSystemTool {
    allowed_paths: Vec<PathBuf>,
}

impl FileSystemTool {
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self { allowed_paths }
    }

    pub fn unrestricted() -> Self {
        Self {
            allowed_paths: vec![],
        }
    }

    fn validate_path(&self, path: &Path) -> Result<()> {
        if self.allowed_paths.is_empty() {
            return Ok(());
        }

        let canonical = path.canonicalize().map_err(|e| {
            Error::Internal(format!("Failed to canonicalize path: {}", e))
        })?;

        for allowed in &self.allowed_paths {
            // Canonicalize allowed path for proper comparison
            if let Ok(canonical_allowed) = allowed.canonicalize() {
                if canonical.starts_with(&canonical_allowed) {
                    return Ok(());
                }
            }
        }

        Err(Error::Internal(format!(
            "Access denied: path {:?} is outside allowed paths",
            path
        )))
    }

    async fn read_file(&self, path: &Path) -> Result<ToolResult> {
        self.validate_path(path)?;

        let content = fs::read_to_string(path).map_err(|e| {
            Error::Internal(format!("Failed to read file: {}", e))
        })?;

        Ok(ToolResult::success(json!({
            "content": content,
            "path": path.display().to_string()
        })))
    }

    async fn write_file(&self, path: &Path, content: &str) -> Result<ToolResult> {
        self.validate_path(path)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                Error::Internal(format!("Failed to create parent directories: {}", e))
            })?;
        }

        fs::write(path, content).map_err(|e| {
            Error::Internal(format!("Failed to write file: {}", e))
        })?;

        Ok(ToolResult::success(json!({
            "path": path.display().to_string(),
            "bytes_written": content.len()
        })))
    }

    async fn list_directory(&self, path: &Path) -> Result<ToolResult> {
        self.validate_path(path)?;

        let entries = fs::read_dir(path)
            .map_err(|e| Error::Internal(format!("Failed to read directory: {}", e)))?;

        let mut files = Vec::new();
        let mut directories = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| {
                Error::Internal(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                directories.push(name);
            } else {
                files.push(name);
            }
        }

        Ok(ToolResult::success(json!({
            "path": path.display().to_string(),
            "files": files,
            "directories": directories
        })))
    }

    async fn delete(&self, path: &Path) -> Result<ToolResult> {
        self.validate_path(path)?;

        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| {
                Error::Internal(format!("Failed to delete directory: {}", e))
            })?;
        } else {
            fs::remove_file(path).map_err(|e| {
                Error::Internal(format!("Failed to delete file: {}", e))
            })?;
        }

        Ok(ToolResult::success(json!({
            "path": path.display().to_string(),
            "deleted": true
        })))
    }

    async fn mkdir(&self, path: &Path) -> Result<ToolResult> {
        self.validate_path(path)?;

        fs::create_dir_all(path).map_err(|e| {
            Error::Internal(format!("Failed to create directory: {}", e))
        })?;

        Ok(ToolResult::success(json!({
            "path": path.display().to_string(),
            "created": true
        })))
    }

    async fn search(&self, dir: &Path, pattern: &str) -> Result<ToolResult> {
        self.validate_path(dir)?;

        let mut matches = Vec::new();
        self.search_recursive(dir, pattern, &mut matches)?;

        Ok(ToolResult::success(json!({
            "pattern": pattern,
            "matches": matches
        })))
    }

    fn search_recursive(&self, dir: &Path, pattern: &str, matches: &mut Vec<String>) -> Result<()> {
        let entries = fs::read_dir(dir)
            .map_err(|e| Error::Internal(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                Error::Internal(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.contains(pattern) {
                matches.push(path.display().to_string());
            }

            if path.is_dir() {
                let _ = self.search_recursive(&path, pattern, matches);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Tool for FileSystemTool {
    fn name(&self) -> &str {
        "file_system"
    }

    fn description(&self) -> &str {
        "Read, write, list, search, delete files and directories"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter::new(
                "operation",
                "Operation: read, write, list, search, delete, mkdir",
                ParameterType::String,
                true,
            ),
            ToolParameter::new(
                "path",
                "File or directory path",
                ParameterType::String,
                true,
            ),
            ToolParameter::new(
                "content",
                "Content to write (for write operation)",
                ParameterType::String,
                false,
            ),
            ToolParameter::new(
                "pattern",
                "Search pattern (for search operation)",
                ParameterType::String,
                false,
            ),
        ]
    }

    async fn execute(&self, params: HashMap<String, Value>) -> Result<ToolResult> {
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing operation parameter".to_string()))?;

        let path_str = params
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Internal("Missing path parameter".to_string()))?;

        let path = PathBuf::from(path_str);

        match operation {
            "read" => self.read_file(&path).await,
            "write" => {
                let content = params
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Internal("Missing content parameter for write".to_string()))?;
                self.write_file(&path, content).await
            }
            "list" => self.list_directory(&path).await,
            "delete" => self.delete(&path).await,
            "mkdir" => self.mkdir(&path).await,
            "search" => {
                let pattern = params
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Internal("Missing pattern parameter for search".to_string()))?;
                self.search(&path, pattern).await
            }
            _ => Err(Error::Internal(format!("Unknown operation: {}", operation))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FileSystemTool::unrestricted();

        let file_path = temp_dir.path().join("test.txt");
        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("write"));
        params.insert("path".to_string(), json!(file_path.display().to_string()));
        params.insert("content".to_string(), json!("Hello, World!"));

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);

        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("read"));
        params.insert("path".to_string(), json!(file_path.display().to_string()));

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        assert_eq!(result.output["content"], "Hello, World!");
    }

    #[tokio::test]
    async fn test_list_directory() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FileSystemTool::unrestricted();

        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("list"));
        params.insert("path".to_string(), json!(temp_dir.path().display().to_string()));

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);

        let files = result.output["files"].as_array().unwrap();
        let dirs = result.output["directories"].as_array().unwrap();

        assert_eq!(files.len(), 2);
        assert_eq!(dirs.len(), 1);
    }

    #[tokio::test]
    async fn test_mkdir() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FileSystemTool::unrestricted();

        let new_dir = temp_dir.path().join("new_dir");
        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("mkdir"));
        params.insert("path".to_string(), json!(new_dir.display().to_string()));

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        assert!(new_dir.exists());
        assert!(new_dir.is_dir());
    }

    #[tokio::test]
    async fn test_delete_file() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FileSystemTool::unrestricted();

        let file_path = temp_dir.path().join("to_delete.txt");
        fs::write(&file_path, "content").unwrap();

        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("delete"));
        params.insert("path".to_string(), json!(file_path.display().to_string()));

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_search() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FileSystemTool::unrestricted();

        fs::write(temp_dir.path().join("test1.txt"), "content").unwrap();
        fs::write(temp_dir.path().join("test2.txt"), "content").unwrap();
        fs::write(temp_dir.path().join("other.txt"), "content").unwrap();

        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("search"));
        params.insert("path".to_string(), json!(temp_dir.path().display().to_string()));
        params.insert("pattern".to_string(), json!("test"));

        let result = tool.execute(params).await.unwrap();
        assert!(result.success);

        let matches = result.output["matches"].as_array().unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[tokio::test]
    async fn test_path_validation() {
        let temp_dir = TempDir::new().unwrap();
        let allowed = temp_dir.path().to_path_buf();
        let tool = FileSystemTool::new(vec![allowed.clone()]);

        let file_path = allowed.join("allowed.txt");
        fs::write(&file_path, "content").unwrap();

        let mut params = HashMap::new();
        params.insert("operation".to_string(), json!("read"));
        params.insert("path".to_string(), json!(file_path.display().to_string()));

        let result = tool.execute(params).await;
        assert!(result.is_ok());
    }
}
