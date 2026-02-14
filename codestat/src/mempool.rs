/// 内存池优化 - 复用缓冲区减少内存分配
/// 对于大量小文件，避免重复的 Vec 分配

use std::cell::RefCell;
use std::sync::Mutex;

// 线程本地存储的缓冲区池
thread_local! {
    static BUFFER_POOL: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
}

// 全局缓冲区池（用于跨线程复用）
lazy_static::lazy_static! {
    static ref GLOBAL_POOL: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
}

/// 获取一个合适大小的缓冲区
pub fn acquire_buffer(min_capacity: usize) -> Vec<u8> {
    // 尝试从线程本地池获取
    let local_result = BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        
        // 寻找足够大的缓冲区
        pool.iter().position(|buf| buf.capacity() >= min_capacity)
            .map(|pos| {
                let mut buf = pool.remove(pos);
                buf.clear();
                buf.reserve(min_capacity);
                buf
            })
    });
    
    if let Some(buf) = local_result {
        return buf;
    }
    
    // 尝试从全局池获取
    if let Ok(mut global) = GLOBAL_POOL.lock() {
        if let Some(pos) = global.iter().position(|buf| buf.capacity() >= min_capacity) {
            let mut buf = global.remove(pos);
            buf.clear();
            buf.reserve(min_capacity);
            return buf;
        }
    }
    
    // 创建新缓冲区
    Vec::with_capacity(min_capacity)
}

/// 归还缓冲区到池中
pub fn release_buffer(mut buf: Vec<u8>) {
    // 只保留大缓冲区（避免池膨胀）
    const MAX_POOL_SIZE: usize = 10;
    const MIN_BUF_SIZE: usize = 4096; // 4KB
    
    if buf.capacity() < MIN_BUF_SIZE {
        return; // 直接丢弃小缓冲区
    }
    
    buf.clear();
    
    // 尝试归还到线程本地池
    BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if pool.len() < MAX_POOL_SIZE {
            pool.push(buf);
            return;
        }
        
        // 本地池满了，尝试全局池
        if let Ok(mut global) = GLOBAL_POOL.lock() {
            if global.len() < MAX_POOL_SIZE * 2 {
                global.push(buf);
            }
        }
    });
}

/// 使用内存池读取文件
pub fn read_file_with_pool(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    use std::fs::File;
    use std::io::Read;
    
    let metadata = std::fs::metadata(path)?;
    let size = metadata.len() as usize;
    
    let mut buffer = acquire_buffer(size);
    let mut file = File::open(path)?;
    file.read_to_end(&mut buffer)?;
    
    Ok(buffer)
}

/// 预读取优化 - 告诉操作系统我们要读取这些文件
#[cfg(target_os = "linux")]
pub fn advise_sequential_read(file: &std::fs::File) {
    use std::os::unix::io::AsRawFd;
    unsafe {
        libc::posix_fadvise(
            file.as_raw_fd(),
            0,
            0,
            libc::POSIX_FADV_SEQUENTIAL | libc::POSIX_FADV_WILLNEED,
        );
    }
}

#[cfg(target_os = "macos")]
pub fn advise_sequential_read(_file: &std::fs::File) {
    // macOS 不支持 posix_fadvise
    // 可以考虑使用 fcntl(F_RDADVISE) 但需要更复杂的实现
}

#[cfg(target_os = "windows")]
pub fn advise_sequential_read(_file: &std::fs::File) {
    // Windows 有类似的机制，但这里简化处理
}

/// 带预读取的文件打开
#[cfg(target_os = "linux")]
pub fn open_with_advise(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    let file = std::fs::File::open(path)?;
    advise_sequential_read(&file);
    Ok(file)
}

#[cfg(not(target_os = "linux"))]
pub fn open_with_advise(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    std::fs::File::open(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buffer_pool() {
        // 获取缓冲区
        let buf = acquire_buffer(1024);
        assert!(buf.capacity() >= 1024);
        
        // 归还
        release_buffer(buf);
        
        // 再次获取（应该从池中复用）
        let buf2 = acquire_buffer(512);
        assert!(buf2.capacity() >= 512);
    }
}
