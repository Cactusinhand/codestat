/// 异步 I/O 版本的文件统计
/// 使用 Tokio 实现并发文件读取

use crate::language::Language;
use crate::stats::FileStats;
use std::path::Path;

/// 使用异步 I/O 读取和统计文件
pub async fn count_file_async(path: &Path, language: Language) -> Result<FileStats, std::io::Error> {
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    
    let metadata = tokio::fs::metadata(path).await?;
    let file_size = metadata.len();
    
    if file_size == 0 {
        return Ok(FileStats::default());
    }
    
    // 异步读取文件
    let mut file = File::open(path).await?;
    let mut buffer = Vec::with_capacity(file_size as usize);
    file.read_to_end(&mut buffer).await?;
    
    // 复用同步版本的解析逻辑
    let stats = crate::counter::analyze_bytes_async(&buffer, language);
    
    Ok(FileStats {
        lines: stats.lines,
        code_lines: stats.code_lines,
        comment_lines: stats.comment_lines,
        blank_lines: stats.blank_lines,
        bytes: file_size,
    })
}

/// 批量异步处理多个文件
pub async fn count_files_batch_async(
    files: Vec<(std::path::PathBuf, Language, u64)>,
) -> Vec<(std::path::PathBuf, Language, Result<FileStats, std::io::Error>)> {
    use futures::future::join_all;
    
    let futures = files.into_iter().map(|(path, lang, _size)| {
        let path_clone = path.clone();
        async move {
            let result = count_file_async(&path, lang).await;
            (path_clone, lang, result)
        }
    });
    
    join_all(futures).await
}

/// 混合模式：小文件用异步 I/O，大文件用内存映射
pub async fn count_file_hybrid(path: &Path, language: Language) -> Result<FileStats, std::io::Error> {
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    use memmap2::Mmap;
    use std::fs::File as StdFile;
    
    const MMAP_THRESHOLD: u64 = 1024 * 1024; // 1MB
    
    let metadata = tokio::fs::metadata(path).await?;
    let file_size = metadata.len();
    
    if file_size == 0 {
        return Ok(FileStats::default());
    }
    
    // 大文件使用内存映射（同步，但更高效）
    if file_size > MMAP_THRESHOLD {
        let file = StdFile::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let stats = crate::counter::analyze_bytes_async(&mmap, language);
        return Ok(FileStats {
            lines: stats.lines,
            code_lines: stats.code_lines,
            comment_lines: stats.comment_lines,
            blank_lines: stats.blank_lines,
            bytes: file_size,
        });
    }
    
    // 小文件使用异步 I/O
    let mut file = File::open(path).await?;
    let mut buffer = Vec::with_capacity(file_size as usize);
    file.read_to_end(&mut buffer).await?;
    
    let stats = crate::counter::analyze_bytes_async(&buffer, language);
    
    Ok(FileStats {
        lines: stats.lines,
        code_lines: stats.code_lines,
        comment_lines: stats.comment_lines,
        blank_lines: stats.blank_lines,
        bytes: file_size,
    })
}
