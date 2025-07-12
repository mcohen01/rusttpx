use std::pin::Pin;
use std::task::{Context, Poll};
use futures::{Stream, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use std::path::PathBuf;

use crate::error::{Error, Result};

/// Streaming response handler
///
/// This provides utilities for handling streaming HTTP responses,
/// including text streams, JSON streams, and file downloads.
pub struct StreamingResponse<T> {
    stream: T,
}

impl<T> StreamingResponse<T>
where
    T: Stream + Unpin,
{
    /// Create a new streaming response
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    /// Get the underlying stream
    pub fn into_inner(self) -> T {
        self.stream
    }

    /// Get a reference to the underlying stream
    pub fn inner(&self) -> &T {
        &self.stream
    }

    /// Get a mutable reference to the underlying stream
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.stream
    }
}

// Note: impl Trait in type aliases is unstable, so we'll use generic types instead
// pub type TextStream = StreamingResponse<impl Stream<Item = Result<String>>>;
// pub type BytesStream = StreamingResponse<impl Stream<Item = Result<Vec<u8>>>>;
// pub type JsonStream = StreamingResponse<impl Stream<Item = Result<Value>>>;

/// Streaming utilities for text streams
impl<T> StreamingResponse<T>
where
    T: Stream<Item = Result<String>> + Unpin,
{
    /// Collect all text chunks into a single string
    pub async fn collect_text(self) -> Result<String> {
        let mut result = String::new();
        let mut stream = self.stream;
        
        while let Some(chunk) = stream.next().await {
            result.push_str(&chunk?);
        }
        
        Ok(result)
    }

    /// Process text chunks with a callback function
    pub async fn for_each_text<F>(mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(&str) -> Result<()>,
    {
        while let Some(chunk) = self.stream.next().await {
            callback(&chunk?)?;
        }
        Ok(())
    }

    /// Filter text chunks based on a predicate
    pub fn filter_text<F>(self, mut predicate: F) -> impl Stream<Item = Result<String>>
    where
        F: FnMut(&str) -> bool + 'static,
    {
        use futures::StreamExt;
        self.stream
            .filter(move |chunk| {
                futures::future::ready(match chunk {
                    Ok(ref c) => predicate(c),
                    Err(_) => true,
                })
            })
            .map(|chunk| chunk)
    }

    /// Transform text chunks with a mapping function
    pub fn map_text<F, U>(self, mapper: F) -> impl Stream<Item = Result<U>>
    where
        F: FnMut(String) -> U + Clone + 'static,
        U: 'static,
    {
        use futures::StreamExt;
        self.stream.map(move |chunk| chunk.map(mapper.clone()))
    }

    /// Take only the first n text chunks
    pub fn take_text(self, n: usize) -> impl Stream<Item = Result<String>> {
        self.stream.take(n)
    }

    /// Skip the first n text chunks
    pub fn skip_text(self, n: usize) -> impl Stream<Item = Result<String>> {
        self.stream.skip(n)
    }
}

/// Streaming utilities for bytes streams
impl<T> StreamingResponse<T>
where
    T: Stream<Item = Result<Vec<u8>>> + Unpin,
{
    /// Collect all bytes into a single vector
    pub async fn collect_bytes(self) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let mut stream = self.stream;
        
        while let Some(chunk) = stream.next().await {
            result.extend(chunk?);
        }
        
        Ok(result)
    }

    /// Process bytes chunks with a callback function
    pub async fn for_each_bytes<F>(mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(&[u8]) -> Result<()>,
    {
        while let Some(chunk) = self.stream.next().await {
            callback(&chunk?)?;
        }
        Ok(())
    }

    /// Write bytes to a writer
    pub async fn write_to<W>(mut self, writer: &mut W) -> Result<u64>
    where
        W: AsyncWrite + Unpin,
    {
        let mut total_written = 0u64;
        
        while let Some(chunk) = self.stream.next().await {
            let bytes = chunk?;
            writer.write_all(&bytes).await.map_err(|e| Error::Custom(format!("IO error: {}", e)))?;
            total_written += bytes.len() as u64;
        }
        
        Ok(total_written)
    }

    /// Save bytes to a file
    pub async fn save_to_file(mut self, path: &str) -> Result<u64> {
        let mut file = tokio::fs::File::create(path).await.map_err(|e| Error::Custom(format!("IO error: {}", e)))?;
        let mut total_written = 0u64;
        
        while let Some(chunk) = self.stream.next().await {
            let bytes = chunk?;
            file.write_all(&bytes).await.map_err(|e| Error::Custom(format!("IO error: {}", e)))?;
            total_written += bytes.len() as u64;
        }
        
        Ok(total_written)
    }

    /// Filter bytes chunks based on a predicate
    pub fn filter_bytes<F>(self, mut predicate: F) -> impl Stream<Item = Result<Vec<u8>>>
    where
        F: FnMut(&[u8]) -> bool + 'static,
    {
        use futures::StreamExt;
        self.stream
            .filter(move |chunk| {
                futures::future::ready(match chunk {
                    Ok(ref c) => predicate(c),
                    Err(_) => true,
                })
            })
            .map(|chunk| chunk)
    }

    /// Transform bytes chunks with a mapping function
    pub fn map_bytes<F, U>(self, mapper: F) -> impl Stream<Item = Result<U>>
    where
        F: FnMut(Vec<u8>) -> U + Clone + 'static,
        U: 'static,
    {
        use futures::StreamExt;
        self.stream.map(move |chunk| chunk.map(mapper.clone()))
    }

    /// Take only the first n bytes chunks
    pub fn take_bytes(self, n: usize) -> impl Stream<Item = Result<Vec<u8>>> {
        self.stream.take(n)
    }

    /// Skip the first n bytes chunks
    pub fn skip_bytes(self, n: usize) -> impl Stream<Item = Result<Vec<u8>>> {
        self.stream.skip(n)
    }
}

/// Streaming utilities for JSON streams
impl<T> StreamingResponse<T>
where
    T: Stream<Item = Result<Value>> + Unpin,
{
    /// Collect all JSON values into a vector
    pub async fn collect_json(self) -> Result<Vec<Value>> {
        let mut result = Vec::new();
        let mut stream = self.stream;
        
        while let Some(chunk) = stream.next().await {
            result.push(chunk?);
        }
        
        Ok(result)
    }

    /// Process JSON chunks with a callback function
    pub async fn for_each_json<F>(mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(&Value) -> Result<()>,
    {
        while let Some(chunk) = self.stream.next().await {
            callback(&chunk?)?;
        }
        Ok(())
    }

    /// Filter JSON chunks based on a predicate
    pub fn filter_json<F>(self, mut predicate: F) -> impl Stream<Item = Result<Value>>
    where
        F: FnMut(&Value) -> bool + 'static,
    {
        use futures::StreamExt;
        self.stream
            .filter(move |chunk| {
                futures::future::ready(match chunk {
                    Ok(ref c) => predicate(c),
                    Err(_) => true,
                })
            })
            .map(|chunk| chunk)
    }

    /// Transform JSON chunks with a mapping function
    pub fn map_json<F, U>(self, mapper: F) -> impl Stream<Item = Result<U>>
    where
        F: FnMut(Value) -> U + Clone + 'static,
        U: 'static,
    {
        use futures::StreamExt;
        self.stream.map(move |chunk| chunk.map(mapper.clone()))
    }

    /// Take only the first n JSON chunks
    pub fn take_json(self, n: usize) -> impl Stream<Item = Result<Value>> {
        self.stream.take(n)
    }

    /// Skip the first n JSON chunks
    pub fn skip_json(self, n: usize) -> impl Stream<Item = Result<Value>> {
        self.stream.skip(n)
    }
}

/// Streaming reader that implements AsyncRead
pub struct StreamingReader<T> {
    stream: T,
    buffer: Vec<u8>,
    position: usize,
}

impl<T> StreamingReader<T>
where
    T: Stream<Item = Result<Vec<u8>>> + Unpin,
{
    /// Create a new streaming reader
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            buffer: Vec::new(),
            position: 0,
        }
    }

    /// Get the underlying stream
    pub fn into_inner(self) -> T {
        self.stream
    }
}

impl<T> AsyncRead for StreamingReader<T>
where
    T: Stream<Item = Result<Vec<u8>>> + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        use std::io::ErrorKind;

        // If we have data in the buffer, read from it
        if self.position < self.buffer.len() {
            let available = self.buffer.len() - self.position;
            let to_copy = std::cmp::min(available, buf.remaining());
            
            buf.put_slice(&self.buffer[self.position..self.position + to_copy]);
            self.position += to_copy;
            
            // If we've consumed all the buffer, clear it
            if self.position >= self.buffer.len() {
                self.buffer.clear();
                self.position = 0;
            }
            
            return Poll::Ready(Ok(()));
        }

        // Try to get more data from the stream
        match self.stream.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                self.buffer = chunk;
                self.position = 0;
                
                // Read from the new buffer
                let to_copy = std::cmp::min(self.buffer.len(), buf.remaining());
                buf.put_slice(&self.buffer[..to_copy]);
                self.position = to_copy;
                
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Err(std::io::Error::new(ErrorKind::Other, e)))
            }
            Poll::Ready(None) => {
                Poll::Ready(Ok(())) // EOF
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Streaming utilities
pub mod utils {
    use super::*;


    /// Create a text stream from a file
    pub async fn text_stream_from_file(path: &str) -> Result<impl Stream<Item = Result<String>>> {
        let file = tokio::fs::File::open(path).await.map_err(|e| Error::Custom(format!("IO error: {}", e)))?;
        let reader = tokio::io::BufReader::new(file);
        
        // Note: tokio::io::Lines doesn't implement Stream directly
        // We'll use a different approach for now
        let stream = futures::stream::once(async move {
            let mut lines = Vec::new();
            let mut lines_iter = tokio::io::AsyncBufReadExt::lines(reader);
            while let Ok(Some(line)) = lines_iter.next_line().await {
                lines.push(Ok(line));
            }
            futures::stream::iter(lines)
        }).flatten();
        
        Ok(stream)
    }

    /// Create a bytes stream from a file
    pub async fn bytes_stream_from_file(path: &str, chunk_size: usize) -> Result<impl Stream<Item = Result<Vec<u8>>>> {
        let file = tokio::fs::File::open(path).await.map_err(|e| Error::Custom(format!("IO error: {}", e)))?;
        let reader = tokio::io::BufReader::new(file);
        
        let stream = futures::stream::unfold(reader, move |mut reader| async move {
            let mut buffer = vec![0u8; chunk_size];
            match reader.read(&mut buffer).await {
                Ok(0) => None, // EOF
                Ok(n) => {
                    buffer.truncate(n);
                    Some((Ok(buffer), reader))
                }
                Err(e) => Some((Err(Error::Custom(format!("IO error: {}", e))), reader)),
            }
        });
        
        Ok(stream)
    }

    /// Create a JSON stream from a file containing JSON lines
    pub async fn json_stream_from_file(path: &str) -> Result<impl Stream<Item = Result<Value>>> {
        let text_stream = text_stream_from_file(path).await?;
        
        // Note: We'll simplify this for now since the filter/map approach has issues
        let json_stream = text_stream.map(|line| {
            line.and_then(|l| {
                if l.trim().is_empty() {
                    Err(Error::Custom("Empty line".to_string()))
                } else {
                    serde_json::from_str(&l).map_err(Error::Json)
                }
            })
        });
        
        Ok(json_stream)
    }

    /// Create a progress callback for streaming downloads
    pub fn progress_callback<F>(mut callback: F) -> impl FnMut(&[u8], u64) -> Result<()>
    where
        F: FnMut(u64, u64) + 'static,
    {
        let mut total_bytes = 0u64;
        move |chunk: &[u8], _chunk_size: u64| {
            total_bytes += chunk.len() as u64;
            callback(total_bytes, 0); // 0 for unknown total
            Ok(())
        }
    }

    /// Create a progress callback with known total size
    pub fn progress_callback_with_total<F>(total_size: u64, mut callback: F) -> impl FnMut(&[u8], u64) -> Result<()>
    where
        F: FnMut(u64, u64) + 'static,
    {
        let mut downloaded_bytes = 0u64;
        move |chunk: &[u8], _chunk_size: u64| {
            downloaded_bytes += chunk.len() as u64;
            callback(downloaded_bytes, total_size);
            Ok(())
        }
    }

    /// Calculate download speed
    pub fn calculate_speed(bytes_downloaded: u64, elapsed_seconds: f64) -> f64 {
        if elapsed_seconds > 0.0 {
            bytes_downloaded as f64 / elapsed_seconds
        } else {
            0.0
        }
    }

    /// Format bytes as human readable string
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.1} {}", size, UNITS[unit_index])
    }

    /// Format speed as human readable string
    pub fn format_speed(bytes_per_second: f64) -> String {
        format!("{}/s", format_bytes(bytes_per_second as u64))
    }
}

/// Streaming download manager
pub struct DownloadManager {
    /// Download directory
    pub download_dir: PathBuf,
    /// Maximum concurrent downloads
    pub max_concurrent: usize,
    /// Download timeout
    pub timeout: std::time::Duration,
}

impl DownloadManager {
    /// Create a new download manager
    pub async fn new(download_dir: &str) -> Result<Self> {
        let download_dir = PathBuf::from(download_dir);
        if !download_dir.exists() {
            tokio::fs::create_dir_all(&download_dir).await.map_err(|e| Error::Custom(format!("IO error: {}", e)))?;
        }
        
        Ok(Self {
            download_dir,
            max_concurrent: 3,
            timeout: std::time::Duration::from_secs(300), // 5 minutes
        })
    }

    /// Set the maximum concurrent downloads
    pub fn max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Set the download timeout
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Download a file from a URL
    pub async fn download_file(&self, url: &str, filename: Option<&str>) -> Result<PathBuf> {
        use tokio::time::timeout;
        
        let client = crate::Client::new();
        let url_parsed = url.parse::<url::Url>()?;
        let response = client.get(url_parsed).send().await?;
        
        let filename = filename.unwrap_or_else(|| {
            url.split('/').last().unwrap_or("download")
        });
        
        let file_path = self.download_dir.join(filename);
        
        let download_future = async {
            let bytes_stream = response.bytes_stream();
            let mut file = tokio::fs::File::create(&file_path).await.map_err(|e| Error::Custom(format!("IO error: {}", e)))?;
            
            let _total_bytes = 0u64;
            tokio::pin!(bytes_stream);
            
            while let Some(chunk) = bytes_stream.next().await {
                let bytes = chunk?;
                file.write_all(&bytes).await.map_err(|e| Error::Custom(format!("IO error: {}", e)))?;
                // total_bytes += bytes.len() as u64; // This line was removed as per the edit hint
            }
            
            Ok(file_path)
        };
        
        timeout(self.timeout, download_future)
            .await
            .map_err(|_| Error::timeout(self.timeout))?
    }

    /// Download multiple files concurrently
    pub async fn download_files(&self, urls: Vec<&str>) -> Result<Vec<PathBuf>> {
        use futures::stream::FuturesUnordered;
        use futures::StreamExt;
        
        let mut downloads = FuturesUnordered::new();
        let mut results = Vec::new();
        
        // Start initial downloads
        for url in urls.iter().take(self.max_concurrent) {
            downloads.push(self.download_file(url, None));
        }
        
        let mut remaining_urls = urls.into_iter().skip(self.max_concurrent);
        
        while let Some(result) = downloads.next().await {
            results.push(result?);
            
            // Start next download if available
            if let Some(url) = remaining_urls.next() {
                downloads.push(self.download_file(url, None));
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[tokio::test]
    async fn test_streaming_response_creation() {
        let data = vec![Ok("hello".to_string()), Ok("world".to_string())];
        let stream = stream::iter(data);
        let response = StreamingResponse::new(stream);
        
        let collected = response.collect_text().await.unwrap();
        assert_eq!(collected, "helloworld");
    }

    #[tokio::test]
    async fn test_bytes_streaming() {
        let data = vec![
            Ok(vec![1, 2, 3]),
            Ok(vec![4, 5, 6]),
        ];
        let stream = stream::iter(data);
        let response = StreamingResponse::new(stream);
        
        let collected = response.collect_bytes().await.unwrap();
        assert_eq!(collected, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_utils() {
        assert_eq!(utils::format_bytes(1024), "1.0 KB");
        assert_eq!(utils::format_bytes(1048576), "1.0 MB");
        assert_eq!(utils::format_speed(1024.0), "1.0 KB/s");
    }
} 