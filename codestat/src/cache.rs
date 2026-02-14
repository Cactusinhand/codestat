/// 增量统计缓存系统
/// 通过缓存文件元数据和统计结果，只处理变更的文件

use crate::language::Language;
use crate::stats::FileStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// 缓存文件路径
pub const CACHE_FILENAME: &str = ".codestat-cache.json";

/// 缓存版本号，用于兼容性检查
const CACHE_VERSION: u32 = 1;

/// 单个文件的缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// 文件路径（相对于项目根目录）
    pub path: PathBuf,
    /// 文件修改时间 (unix timestamp)
    pub mtime: u64,
    /// 文件大小
    pub size: u64,
    /// 可选：文件内容哈希（用于精确检测变更）
    pub hash: Option<String>,
    /// 检测到的语言
    pub language: Language,
    /// 统计结果
    pub stats: FileStats,
    /// 缓存创建时间
    pub cached_at: u64,
}

/// 缓存结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cache {
    /// 缓存版本
    pub version: u32,
    /// 缓存创建工具版本
    pub tool_version: String,
    /// 项目根目录（用于验证）
    pub root_path: PathBuf,
    /// 文件缓存条目
    pub entries: HashMap<PathBuf, CacheEntry>,
    /// 总统计结果（汇总）
    pub total_stats: Option<crate::stats::TotalStats>,
    /// 最后更新时间
    pub last_updated: u64,
}

impl Cache {
    /// 创建新的空缓存
    pub fn new(root_path: &Path) -> Self {
        Self {
            version: CACHE_VERSION,
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            root_path: root_path.to_path_buf(),
            entries: HashMap::new(),
            total_stats: None,
            last_updated: now_unix(),
        }
    }

    /// 从文件加载缓存
    pub fn load(root_path: &Path) -> Option<Self> {
        let cache_path = root_path.join(CACHE_FILENAME);
        if !cache_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&cache_path).ok()?;
        let cache: Cache = serde_json::from_str(&content).ok()?;

        // 验证缓存版本
        if cache.version != CACHE_VERSION {
            eprintln!("Cache version mismatch, rebuilding...");
            return None;
        }

        // 验证项目路径
        if cache.root_path != root_path {
            eprintln!("Cache root path mismatch, rebuilding...");
            return None;
        }

        Some(cache)
    }

    /// 保存缓存到文件
    pub fn save(&self, root_path: &Path) -> std::io::Result<()> {
        let cache_path = root_path.join(CACHE_FILENAME);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&cache_path, content)?;
        
        // 更新文件时间为隐藏文件（Unix）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&cache_path)?.permissions();
            perms.set_mode(0o600); // 只有所有者可读写
            fs::set_permissions(&cache_path, perms)?;
        }

        Ok(())
    }

    /// 检查缓存是否有效
    pub fn is_valid(&self, path: &Path, metadata: &Metadata) -> Option<&CacheEntry> {
        let entry = self.entries.get(path)?;

        // 检查文件大小
        if entry.size != metadata.len() {
            return None;
        }

        // 检查修改时间
        let mtime = metadata.modified().ok()?;
        let mtime_unix = mtime.duration_since(SystemTime::UNIX_EPOCH).ok()?.as_secs();
        
        if entry.mtime != mtime_unix {
            return None;
        }

        Some(entry)
    }

    /// 添加或更新缓存条目
    pub fn update(&mut self, path: PathBuf, language: Language, stats: FileStats, metadata: &Metadata) {
        let mtime = metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let entry = CacheEntry {
            path: path.clone(),
            mtime,
            size: metadata.len(),
            hash: None, // 可选：计算文件哈希
            language,
            stats,
            cached_at: now_unix(),
        };

        self.entries.insert(path, entry);
        self.last_updated = now_unix();
    }

    /// 清理不存在的文件缓存
    pub fn cleanup(&mut self, root_path: &Path) {
        self.entries.retain(|path, _| {
            root_path.join(path).exists()
        });
    }

    /// 获取缓存命中率统计
    pub fn hit_rate(&self, checked: usize, hits: usize) -> f64 {
        if checked == 0 {
            0.0
        } else {
            (hits as f64 / checked as f64) * 100.0
        }
    }

    /// 缓存条目数量
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 获取当前 Unix 时间戳
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

/// 增量统计上下文
pub struct IncrementalContext {
    pub cache: Cache,
    pub root_path: PathBuf,
    pub hits: usize,
    pub misses: usize,
    pub new_files: usize,
}

impl IncrementalContext {
    pub fn new(root_path: &Path, use_cache: bool) -> Self {
        let cache = if use_cache {
            Cache::load(root_path).unwrap_or_else(|| Cache::new(root_path))
        } else {
            Cache::new(root_path)
        };

        Self {
            cache,
            root_path: root_path.to_path_buf(),
            hits: 0,
            misses: 0,
            new_files: 0,
        }
    }

    /// 尝试从缓存获取统计结果
    pub fn try_get(&self, path: &Path) -> Option<(Language, FileStats)> {
        let metadata = std::fs::metadata(&self.root_path.join(path)).ok()?;
        let entry = self.cache.is_valid(path, &metadata)?;
        
        Some((entry.language, entry.stats.clone()))
    }

    /// 更新缓存
    pub fn update(&mut self, path: PathBuf, language: Language, stats: FileStats) {
        if let Ok(metadata) = std::fs::metadata(&self.root_path.join(&path)) {
            self.cache.update(path, language, stats, &metadata);
        }
    }

    /// 记录命中
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// 记录未命中
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// 记录新文件
    pub fn record_new(&mut self) {
        self.new_files += 1;
        self.misses += 1;
    }

    /// 获取命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// 打印统计信息
    pub fn print_stats(&self) {
        if self.hits + self.misses > 0 {
            println!("\n  Cache Statistics:");
            println!("    Hits:      {}", self.hits);
            println!("    Misses:    {}", self.misses);
            println!("    New files: {}", self.new_files);
            println!("    Hit rate:  {:.1}%", self.hit_rate());
        }
    }

    /// 保存缓存
    pub fn save(&self) -> std::io::Result<()> {
        self.cache.save(&self.root_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_cache_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();
        
        let mut cache = Cache::new(root);
        
        // 创建测试文件
        let test_file = root.join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();
        
        let metadata = fs::metadata(&test_file).unwrap();
        cache.update(
            PathBuf::from("test.rs"),
            Language::Rust,
            FileStats::default(),
            &metadata
        );
        
        // 保存
        cache.save(root).unwrap();
        
        // 加载
        let loaded = Cache::load(root).unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded.entries.contains_key(&PathBuf::from("test.rs")));
    }

    #[test]
    fn test_incremental_context() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();
        
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        
        let ctx = IncrementalContext::new(root, false);
        assert_eq!(ctx.hit_rate(), 0.0);
        
        // 新文件应该是 miss
        let result = ctx.try_get(&PathBuf::from("main.rs"));
        assert!(result.is_none());
    }
}
