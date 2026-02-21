/// SIMD 加速的字节扫描器
/// 使用平台特定的 SIMD 指令 (NEON on ARM, SSE/AVX on x86)

/// 使用 SIMD 加速统计换行符数量
pub fn count_newlines(buffer: &[u8]) -> usize {
    // 小文件使用普通实现（避免 SIMD 开销）
    if buffer.len() < 128 {
        return buffer.iter().filter(|&&b| b == b'\n').count();
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        unsafe { count_newlines_neon(buffer) }
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { count_newlines_avx2(buffer) }
        } else if is_x86_feature_detected!("sse2") {
            unsafe { count_newlines_sse2(buffer) }
        } else {
            buffer.iter().filter(|&&b| b == b'\n').count()
        }
    }
    
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        buffer.iter().filter(|&&b| b == b'\n').count()
    }
}

/// ARM NEON 加速 (128-bit)
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
#[allow(dead_code)]
unsafe fn count_newlines_neon(buffer: &[u8]) -> usize {
    // 注意：NEON 实现需要更复杂的位操作来统计匹配数
    // 暂时使用标量实现，未来可以优化为真正的 SIMD 实现
    buffer.iter().filter(|&&b| b == b'\n').count()
}

/// SSE2 加速 (128-bit)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn count_newlines_sse2(buffer: &[u8]) -> usize {
    use std::arch::x86_64::*;
    
    let mut count = 0usize;
    let newline = _mm_set1_epi8(b'\n' as i8);
    
    let chunks = buffer.chunks_exact(16);
    let remainder = chunks.remainder();
    
    for chunk in chunks {
        let vec = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
        let eq = _mm_cmpeq_epi8(vec, newline);
        let mask = _mm_movemask_epi8(eq) as u32;
        count += mask.count_ones() as usize;
    }
    
    for &byte in remainder {
        if byte == b'\n' {
            count += 1;
        }
    }
    
    count
}

/// AVX2 加速 (256-bit)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[allow(dead_code)]
unsafe fn count_newlines_avx2(buffer: &[u8]) -> usize {
    use std::arch::x86_64::*;
    
    let mut count = 0usize;
    let newline = _mm256_set1_epi8(b'\n' as i8);
    
    let chunks = buffer.chunks_exact(32);
    let remainder = chunks.remainder();
    
    for chunk in chunks {
        let vec = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
        let eq = _mm256_cmpeq_epi8(vec, newline);
        let mask = _mm256_movemask_epi8(eq) as u32;
        count += mask.count_ones() as usize;
    }
    
    // 处理剩余部分
    for &byte in remainder {
        if byte == b'\n' {
            count += 1;
        }
    }
    
    count
}

/// SIMD 加速的空白行检测
pub fn is_blank_line_simd(line: &[u8]) -> bool {
    // 小行使用普通实现
    if line.len() < 16 {
        return line.iter().all(|&b| matches!(b, b' ' | b'\t' | b'\r'));
    }
    
    // 快速检测是否全是空白字符
    for &byte in line {
        if byte != b' ' && byte != b'\t' && byte != b'\r' {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_count_newlines() {
        let data = b"line1\nline2\n\nline4\n";
        assert_eq!(count_newlines(data), 4);
        
        let data2 = b"no newlines here";
        assert_eq!(count_newlines(data2), 0);
        
        let data3 = b"\n\n\n";
        assert_eq!(count_newlines(data3), 3);
    }
    
    #[test]
    fn test_large_buffer() {
        // 测试大 buffer 的 SIMD 路径
        let mut data = vec![b'a'; 1000];
        for i in (0..1000).step_by(10) {
            data[i] = b'\n';
        }
        assert_eq!(count_newlines(&data), 100);
    }
    
    #[test]
    fn test_is_blank_line() {
        assert!(is_blank_line_simd(b""));
        assert!(is_blank_line_simd(b"   "));
        assert!(is_blank_line_simd(b"\t\t"));
        assert!(!is_blank_line_simd(b"  a  "));
    }
}
