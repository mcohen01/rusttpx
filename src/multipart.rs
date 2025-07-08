use std::path::PathBuf;
use std::collections::HashMap;
use reqwest::multipart::Form;
use serde::{Serialize, Deserialize};

use crate::error::{Error, Result};

/// Multipart form data builder
///
/// This provides a fluent interface for building multipart form data
/// requests with files and fields.
#[derive(Debug)]
pub struct MultipartBuilder {
    form: Form,
    fields: HashMap<String, String>,
    files: HashMap<String, FileData>,
}

/// File data for multipart uploads
#[derive(Debug, Clone)]
pub struct FileData {
    /// File path
    pub path: PathBuf,
    /// File name (optional, defaults to path filename)
    pub filename: Option<String>,
    /// Content type (optional)
    pub content_type: Option<String>,
}

impl FileData {
    /// Create a new file data instance
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            filename: None,
            content_type: None,
        }
    }

    /// Set the filename
    pub fn filename(mut self, filename: &str) -> Self {
        self.filename = Some(filename.to_string());
        self
    }

    /// Set the content type
    pub fn content_type(mut self, content_type: &str) -> Self {
        self.content_type = Some(content_type.to_string());
        self
    }

    /// Get the filename (either set or from path)
    pub fn get_filename(&self) -> String {
        self.filename.clone().unwrap_or_else(|| {
            self.path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string()
        })
    }
}

impl MultipartBuilder {
    /// Create a new multipart builder
    pub fn new() -> Self {
        Self {
            form: Form::new(),
            fields: HashMap::new(),
            files: HashMap::new(),
        }
    }

    /// Add a text field
    pub fn text(mut self, name: &str, value: &str) -> Self {
        let name_owned = name.to_string();
        let value_owned = value.to_string();
        self.form = self.form.text(name_owned.clone(), value_owned.clone());
        self.fields.insert(name_owned, value_owned);
        self
    }

    /// Add a file field
    pub fn file(mut self, name: &str, path: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(Error::multipart(format!("File not found: {}", path.display())));
        }
        
        let file_data = FileData::new(path.clone());
        self.files.insert(name.to_string(), file_data.clone());
        
        // Add to reqwest form
        let filename = file_data.get_filename();
        let name_owned = name.to_string();
        match std::fs::read(&file_data.path) {
            Ok(data) => {
                let part = reqwest::multipart::Part::bytes(data)
                    .file_name(filename);
                self.form = self.form.part(name_owned, part);
            }
            Err(_) => {
                return Err(Error::multipart(format!("Failed to read file: {}", path.display())));
            }
        }
        
        Ok(self)
    }

    /// Add a file field with custom filename
    pub fn file_with_name(mut self, name: &str, path: &str, filename: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(Error::multipart(format!("File not found: {}", path.display())));
        }
        
        let mut file_data = FileData::new(path.clone());
        file_data = file_data.filename(filename);
        self.files.insert(name.to_string(), file_data.clone());
        
        // Add to reqwest form
        let name_owned = name.to_string();
        let filename_owned = filename.to_string();
        match std::fs::read(&file_data.path) {
            Ok(data) => {
                let part = reqwest::multipart::Part::bytes(data)
                    .file_name(filename_owned);
                self.form = self.form.part(name_owned, part);
            }
            Err(_) => {
                return Err(Error::multipart(format!("Failed to read file: {}", path.display())));
            }
        }
        
        Ok(self)
    }

    /// Add a file field with content type
    pub fn file_with_content_type(mut self, name: &str, path: &str, content_type: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(Error::multipart(format!("File not found: {}", path.display())));
        }
        
        let mut file_data = FileData::new(path.clone());
        file_data = file_data.content_type(content_type);
        self.files.insert(name.to_string(), file_data.clone());
        
        // Add to reqwest form
        let name_owned = name.to_string();
        match std::fs::read(&file_data.path) {
            Ok(data) => {
                let part = reqwest::multipart::Part::bytes(data)
                    .mime_str(content_type)
                    .map_err(|e| Error::multipart(format!("Invalid content type: {}", e)))?;
                self.form = self.form.part(name_owned, part);
            }
            Err(_) => {
                return Err(Error::multipart(format!("Failed to read file: {}", path.display())));
            }
        }
        
        Ok(self)
    }

    /// Add bytes as a file
    pub fn bytes(mut self, name: &str, data: Vec<u8>, filename: &str) -> Self {
        let name_owned = name.to_string();
        let filename_owned = filename.to_string();
        let part = reqwest::multipart::Part::bytes(data)
            .file_name(filename_owned);
        self.form = self.form.part(name_owned, part);
        self
    }

    /// Add bytes as a file with content type
    pub fn bytes_with_content_type(mut self, name: &str, data: Vec<u8>, filename: &str, content_type: &str) -> Result<Self> {
        let name_owned = name.to_string();
        let filename_owned = filename.to_string();
        let part = reqwest::multipart::Part::bytes(data)
            .file_name(filename_owned)
            .mime_str(content_type)
            .map_err(|e| Error::multipart(format!("Invalid content type: {}", e)))?;
        self.form = self.form.part(name_owned, part);
        Ok(self)
    }

    // Note: reqwest::multipart::Part::stream has compatibility issues in this version
    // pub fn stream(mut self, name: &str, stream: impl futures::Stream<Item = Result<Vec<u8>, std::io::Error>> + Send + 'static, filename: &str) -> Self {
    //     let part = reqwest::multipart::Part::stream(stream)
    //         .file_name(filename);
    //     self.form = self.form.part(name, part);
    //     self
    // }

    /// Add multiple text fields from a map
    pub fn fields(mut self, fields: HashMap<String, String>) -> Self {
        for (name, value) in fields {
            let name_owned = name.clone();
            let value_owned = value.clone();
            self.form = self.form.text(name_owned, value_owned);
            self.fields.insert(name, value);
        }
        self
    }

    /// Add multiple files from a map
    pub fn files(mut self, files: HashMap<String, String>) -> Result<Self> {
        for (name, path) in files {
            self = self.file(&name, &path)?;
        }
        Ok(self)
    }

    /// Get the fields
    pub fn get_fields(&self) -> &HashMap<String, String> {
        &self.fields
    }

    /// Get the files
    pub fn get_files(&self) -> &HashMap<String, FileData> {
        &self.files
    }

    /// Check if the form has any fields
    pub fn has_fields(&self) -> bool {
        !self.fields.is_empty()
    }

    /// Check if the form has any files
    pub fn has_files(&self) -> bool {
        !self.files.is_empty()
    }

    /// Get the number of fields
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Get the number of files
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Build the multipart form
    pub fn build(self) -> Form {
        self.form
    }

    /// Build and get the boundary
    pub fn build_with_boundary(self) -> (Form, String) {
        let boundary = self.form.boundary().to_string();
        (self.form, boundary)
    }
}

impl Default for MultipartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Multipart form data with metadata
#[derive(Debug, Clone)]
pub struct MultipartForm {
    /// Form fields
    pub fields: HashMap<String, String>,
    /// Form files
    pub files: HashMap<String, FileData>,
    /// Form boundary
    pub boundary: String,
}

impl MultipartForm {
    /// Create a new multipart form
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            files: HashMap::new(),
            boundary: generate_boundary(),
        }
    }

    /// Add a text field
    pub fn add_field(mut self, name: &str, value: &str) -> Self {
        self.fields.insert(name.to_string(), value.to_string());
        self
    }

    /// Add a file
    pub fn add_file(mut self, name: &str, file_data: FileData) -> Self {
        self.files.insert(name.to_string(), file_data);
        self
    }

    /// Add multiple fields
    pub fn add_fields(mut self, fields: HashMap<String, String>) -> Self {
        self.fields.extend(fields);
        self
    }

    /// Add multiple files
    pub fn add_files(mut self, files: HashMap<String, FileData>) -> Self {
        self.files.extend(files);
        self
    }

    /// Get a field value
    pub fn get_field(&self, name: &str) -> Option<&String> {
        self.fields.get(name)
    }

    /// Get a file
    pub fn get_file(&self, name: &str) -> Option<&FileData> {
        self.files.get(name)
    }

    /// Check if a field exists
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    /// Check if a file exists
    pub fn has_file(&self, name: &str) -> bool {
        self.files.contains_key(name)
    }

    /// Remove a field
    pub fn remove_field(&mut self, name: &str) -> Option<String> {
        self.fields.remove(name)
    }

    /// Remove a file
    pub fn remove_file(&mut self, name: &str) -> Option<FileData> {
        self.files.remove(name)
    }

    /// Clear all fields
    pub fn clear_fields(&mut self) {
        self.fields.clear();
    }

    /// Clear all files
    pub fn clear_files(&mut self) {
        self.files.clear();
    }

    /// Get the total size (approximate)
    pub fn size(&self) -> usize {
        let fields_size: usize = self.fields
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum();
        
        let files_size: usize = self.files
            .iter()
            .map(|(_, file)| {
                if let Ok(metadata) = std::fs::metadata(&file.path) {
                    metadata.len() as usize
                } else {
                    0
                }
            })
            .sum();
        
        fields_size + files_size
    }

    /// Convert to reqwest Form
    pub fn to_reqwest_form(self) -> Result<Form> {
        let mut form = Form::new();
        
        // Add fields
        for (name, value) in self.fields {
            form = form.text(name, value);
        }
        
        // Add files
        for (name, file_data) in self.files {
            if !file_data.path.exists() {
                return Err(Error::multipart(format!("File not found: {}", file_data.path.display())));
            }
            
            match std::fs::read(&file_data.path) {
                Ok(data) => {
                    let mut part = reqwest::multipart::Part::bytes(data.clone());
                    if let Some(filename) = &file_data.filename {
                        let filename_owned = filename.clone();
                        part = part.file_name(filename_owned);
                    }
                    if let Some(content_type) = &file_data.content_type {
                        let name_owned = name.clone();
                        match part.mime_str(content_type) {
                            Ok(part) => {
                                form = form.part(name_owned, part);
                            }
                            Err(_) => {
                                // If content type is invalid, create a new part without content type
                                let name_owned = name.clone();
                                let new_part = reqwest::multipart::Part::bytes(data);
                                form = form.part(name_owned, new_part);
                            }
                        }
                    } else {
                        let name_owned = name.clone();
                        form = form.part(name_owned, part);
                    }
                }
                Err(_) => {
                    return Err(Error::multipart(format!("Failed to read file: {}", file_data.path.display())));
                }
            }
        }
        
        Ok(form)
    }
}

impl Default for MultipartForm {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random boundary for multipart forms
fn generate_boundary() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    format!("----WebKitFormBoundary{}", hex::encode(bytes))
}

/// Multipart utilities
pub mod utils {
    use super::*;

    /// Check if a file is valid for multipart upload
    pub fn is_valid_file(path: &PathBuf) -> bool {
        if !path.exists() {
            return false;
        }
        
        if let Ok(metadata) = std::fs::metadata(path) {
            metadata.is_file() && metadata.len() > 0
        } else {
            false
        }
    }

    /// Get the content type for a file based on its extension
    pub fn get_content_type_for_file(path: &PathBuf) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" => Some("image/jpeg"),
                "png" => Some("image/png"),
                "gif" => Some("image/gif"),
                "pdf" => Some("application/pdf"),
                "txt" => Some("text/plain"),
                "html" | "htm" => Some("text/html"),
                "css" => Some("text/css"),
                "js" => Some("application/javascript"),
                "json" => Some("application/json"),
                "xml" => Some("application/xml"),
                "zip" => Some("application/zip"),
                "tar" => Some("application/x-tar"),
                "gz" => Some("application/gzip"),
                _ => None,
            })
            .map(|s| s.to_string())
    }

    /// Create a multipart form from a directory
    pub fn from_directory(dir_path: &str, field_name: &str) -> Result<MultipartForm> {
        let dir_path = PathBuf::from(dir_path);
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(Error::multipart(format!("Directory not found: {}", dir_path.display())));
        }
        
        let mut form = MultipartForm::new();
        
        for entry in std::fs::read_dir(&dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");
                
                let file_data = FileData::new(path.clone());
                form = form.add_file(&format!("{}_{}", field_name, filename), file_data);
            }
        }
        
        Ok(form)
    }

    /// Create a multipart form from a struct
    pub fn from_struct<T: Serialize>(data: &T) -> Result<MultipartForm> {
        let mut form = MultipartForm::new();
        
        // Convert struct to HashMap
        let json = serde_json::to_value(data)?;
        if let serde_json::Value::Object(map) = json {
            for (key, value) in map {
                if let Some(str_value) = value.as_str() {
                    form = form.add_field(&key, str_value);
                } else {
                    form = form.add_field(&key, &value.to_string());
                }
            }
        }
        
        Ok(form)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_multipart_builder_creation() {
        let builder = MultipartBuilder::new();
        assert!(!builder.has_fields());
        assert!(!builder.has_files());
    }

    #[test]
    fn test_multipart_builder_text() {
        let builder = MultipartBuilder::new()
            .text("name", "value")
            .text("another", "field");
        
        assert!(builder.has_fields());
        assert_eq!(builder.field_count(), 2);
        assert_eq!(builder.get_fields().get("name"), Some(&"value".to_string()));
    }

    #[test]
    fn test_file_data() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_file.txt");
        
        // Create a test file
        std::fs::write(&test_file, "test content").unwrap();
        
        let file_data = FileData::new(test_file.clone())
            .filename("custom_name.txt")
            .content_type("text/plain");
        
        assert_eq!(file_data.get_filename(), "custom_name.txt");
        assert_eq!(file_data.content_type, Some("text/plain".to_string()));
    }

    #[test]
    fn test_multipart_form() {
        let mut form = MultipartForm::new();
        form = form.add_field("name", "value");
        
        assert!(form.has_field("name"));
        assert_eq!(form.get_field("name"), Some(&"value".to_string()));
    }

    #[test]
    fn test_utils() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_utils.txt");
        
        // Test with non-existent file
        assert!(!utils::is_valid_file(&test_file));
        
        // Create a test file
        std::fs::write(&test_file, "test content").unwrap();
        
        // Test with existing file
        assert!(utils::is_valid_file(&test_file));
        
        // Test content type detection
        let content_type = utils::get_content_type_for_file(&test_file);
        assert_eq!(content_type, Some("text/plain".to_string()));
        
        // Clean up
        std::fs::remove_file(&test_file).unwrap();
    }

    #[test]
    fn test_boundary_generation() {
        let boundary1 = generate_boundary();
        let boundary2 = generate_boundary();
        
        assert!(boundary1.starts_with("----WebKitFormBoundary"));
        assert!(boundary2.starts_with("----WebKitFormBoundary"));
        assert_ne!(boundary1, boundary2);
    }
} 