use crate::utils::error::{ErpError, ErpResult};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};

/// 대량 데이터 처리를 위한 성능 최적화 유틸리티
pub struct PerformanceOptimizer {
    /// 동시 처리 제한을 위한 세마포어
    semaphore: Arc<Semaphore>,
    /// 메모리 사용량 모니터링
    memory_monitor: Arc<RwLock<MemoryMonitor>>,
    /// 배치 처리 설정
    batch_config: BatchConfig,
}

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub max_batch_size: usize,
    pub min_batch_size: usize,
    pub batch_timeout_ms: u64,
    pub memory_threshold_mb: u64,
    pub concurrent_batches: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            min_batch_size: 10,
            batch_timeout_ms: 5000,
            memory_threshold_mb: 512,
            concurrent_batches: 4,
        }
    }
}

#[derive(Debug)]
#[derive(Default)]
struct MemoryMonitor {
    current_usage_mb: u64,
    peak_usage_mb: u64,
    allocations: u64,
    deallocations: u64,
}

impl PerformanceOptimizer {
    pub fn new(config: BatchConfig) -> Self {
        let concurrent_limit = config.concurrent_batches;
        Self {
            semaphore: Arc::new(Semaphore::new(concurrent_limit)),
            memory_monitor: Arc::new(RwLock::new(MemoryMonitor::default())),
            batch_config: config,
        }
    }

    /// 대량 데이터를 배치로 나누어 처리
    pub async fn process_in_batches<T, F, R, Fut>(
        &self,
        data: Vec<T>,
        processor: F,
    ) -> ErpResult<Vec<R>>
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = ErpResult<Vec<R>>> + Send,
        T: Send + 'static + Clone,
        R: Send + 'static,
    {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        info!("대량 데이터 처리 시작: {} 건", data.len());

        // 메모리 사용량 체크
        self.check_memory_usage().await?;

        // 배치 크기 동적 조정
        let batch_size = self.calculate_optimal_batch_size(data.len()).await;
        debug!("최적 배치 크기: {}", batch_size);

        let mut results = Vec::new();
        let mut tasks = Vec::new();

        // 데이터를 배치로 분할
        for chunk in data.chunks(batch_size) {
            let chunk_data = chunk.to_vec();
            let processor_clone = processor.clone();
            let semaphore = Arc::clone(&self.semaphore);
            let memory_monitor = Arc::clone(&self.memory_monitor);

            let task = tokio::spawn(async move {
                // 동시 실행 제한
                let _permit = semaphore.acquire().await.unwrap();

                // 메모리 사용량 업데이트
                {
                    let mut monitor = memory_monitor.write().await;
                    monitor.allocations += 1;
                    // 실제 메모리 사용량은 시스템 정보로부터 가져와야 함
                    // 여기서는 예시로 간단한 추정치 사용
                    let estimated_mb = chunk_data.len() * std::mem::size_of::<T>() / 1024 / 1024;
                    monitor.current_usage_mb += estimated_mb as u64;
                    if monitor.current_usage_mb > monitor.peak_usage_mb {
                        monitor.peak_usage_mb = monitor.current_usage_mb;
                    }
                }

                let result = processor_clone(chunk_data).await;

                // 메모리 해제 기록
                {
                    let mut monitor = memory_monitor.write().await;
                    monitor.deallocations += 1;
                }

                result
            });

            tasks.push(task);
        }

        // 모든 배치 작업 완료 대기
        for task in tasks {
            match task.await {
                Ok(batch_result) => match batch_result {
                    Ok(mut batch_data) => {
                        results.append(&mut batch_data);
                    }
                    Err(e) => {
                        warn!("배치 처리 중 오류 발생: {}", e);
                        return Err(e);
                    }
                },
                Err(e) => {
                    return Err(ErpError::internal(format!("배치 작업 실행 오류: {}", e)));
                }
            }
        }

        info!("대량 데이터 처리 완료: {} 건 처리됨", results.len());
        self.log_memory_stats().await;

        Ok(results)
    }

    /// 스트리밍 방식으로 대량 데이터 처리
    pub async fn process_stream<T, F, R, Fut>(
        &self,
        mut data_stream: tokio::sync::mpsc::Receiver<T>,
        processor: F,
    ) -> ErpResult<tokio::sync::mpsc::Receiver<R>>
    where
        F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = ErpResult<R>> + Send,
        T: Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = tokio::sync::mpsc::channel(self.batch_config.max_batch_size);
        let semaphore = Arc::clone(&self.semaphore);

        tokio::spawn(async move {
            while let Some(item) = data_stream.recv().await {
                let processor_clone = processor.clone();
                let tx_clone = tx.clone();
                let semaphore_clone = Arc::clone(&semaphore);

                tokio::spawn(async move {
                    let _permit = semaphore_clone.acquire().await.unwrap();

                    match processor_clone(item).await {
                        Ok(result) => {
                            if (tx_clone.send(result).await).is_err() {
                                debug!("스트림 수신자가 종료됨");
                            }
                        }
                        Err(e) => {
                            warn!("스트림 처리 중 오류: {}", e);
                        }
                    }
                });
            }
        });

        Ok(rx)
    }

    /// 메모리 풀을 사용한 객체 재사용
    pub fn create_object_pool<T: Clone + Default>(&self, initial_size: usize) -> ObjectPool<T> {
        ObjectPool::new(initial_size)
    }

    /// 캐시 최적화된 데이터 구조 변환
    pub fn optimize_for_cache<T: Clone>(
        &self,
        data: Vec<T>,
        chunk_size: Option<usize>,
    ) -> Vec<Vec<T>> {
        let chunk_size = chunk_size.unwrap_or_else(|| {
            // CPU 캐시 라인 크기를 고려한 최적 청크 크기 계산
            let cache_line_size = 64; // bytes
            let item_size = std::mem::size_of::<T>();
            std::cmp::max(1, cache_line_size / item_size)
        });

        data.chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// 메모리 사용량 체크
    async fn check_memory_usage(&self) -> ErpResult<()> {
        let monitor = self.memory_monitor.read().await;

        if monitor.current_usage_mb > self.batch_config.memory_threshold_mb {
            warn!(
                "메모리 사용량이 임계치를 초과했습니다: {}MB > {}MB",
                monitor.current_usage_mb, self.batch_config.memory_threshold_mb
            );

            // 가비지 컬렉션 유도 (Rust에서는 명시적 GC가 없으므로 메모리 해제 권장사항)
            tokio::task::yield_now().await;
        }

        Ok(())
    }

    /// 최적 배치 크기 계산
    async fn calculate_optimal_batch_size(&self, total_items: usize) -> usize {
        let monitor = self.memory_monitor.read().await;
        let available_memory = self
            .batch_config
            .memory_threshold_mb
            .saturating_sub(monitor.current_usage_mb);

        // 메모리 기반 배치 크기 조정
        let memory_based_size = if available_memory > 0 {
            // 사용 가능한 메모리의 80%만 사용
            ((available_memory * 1024 * 1024 * 8) / 10) as usize / std::mem::size_of::<usize>()
        } else {
            self.batch_config.min_batch_size
        };

        // CPU 코어 수 기반 조정
        let cpu_cores = num_cpus::get();
        let cpu_based_size = total_items / (cpu_cores * 2);

        // 최종 배치 크기 결정
        [
            memory_based_size,
            cpu_based_size,
            self.batch_config.max_batch_size,
        ]
        .iter()
        .filter(|&&size| size >= self.batch_config.min_batch_size)
        .min()
        .copied()
        .unwrap_or(self.batch_config.min_batch_size)
    }

    /// 메모리 통계 로깅
    async fn log_memory_stats(&self) {
        let monitor = self.memory_monitor.read().await;
        debug!(
            "메모리 사용 통계 - 현재: {}MB, 최대: {}MB, 할당: {}, 해제: {}",
            monitor.current_usage_mb,
            monitor.peak_usage_mb,
            monitor.allocations,
            monitor.deallocations
        );
    }
}

/// 객체 풀 구현
pub struct ObjectPool<T> {
    pool: Arc<RwLock<Vec<T>>>,
    factory: fn() -> T,
}

impl<T: Clone + Default> ObjectPool<T> {
    fn new(initial_size: usize) -> Self {
        let mut pool = Vec::with_capacity(initial_size);
        for _ in 0..initial_size {
            pool.push(T::default());
        }

        Self {
            pool: Arc::new(RwLock::new(pool)),
            factory: T::default,
        }
    }

    pub async fn get(&self) -> T {
        let mut pool = self.pool.write().await;
        pool.pop().unwrap_or_else(|| (self.factory)())
    }

    pub async fn return_object(&self, obj: T) {
        let mut pool = self.pool.write().await;
        pool.push(obj);
    }

    pub async fn size(&self) -> usize {
        let pool = self.pool.read().await;
        pool.len()
    }
}

/// 성능 측정 유틸리티
pub struct PerformanceMeasurer {
    start_time: std::time::Instant,
    checkpoints: Vec<(String, std::time::Duration)>,
}

impl Default for PerformanceMeasurer {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMeasurer {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            checkpoints: Vec::new(),
        }
    }

    pub fn checkpoint(&mut self, name: &str) {
        let elapsed = self.start_time.elapsed();
        self.checkpoints.push((name.to_string(), elapsed));
        debug!("성능 체크포인트 '{}': {:?}", name, elapsed);
    }

    pub fn finish(mut self, operation_name: &str) -> PerformanceReport {
        let total_duration = self.start_time.elapsed();
        self.checkpoint("완료");

        info!(
            "성능 측정 완료 '{}': 총 시간 {:?}",
            operation_name, total_duration
        );

        PerformanceReport {
            operation_name: operation_name.to_string(),
            total_duration,
            checkpoints: self.checkpoints,
        }
    }
}

#[derive(Debug)]
pub struct PerformanceReport {
    pub operation_name: String,
    pub total_duration: std::time::Duration,
    pub checkpoints: Vec<(String, std::time::Duration)>,
}

impl PerformanceReport {
    pub fn print_summary(&self) {
        println!("\n=== 성능 보고서: {} ===", self.operation_name);
        println!("총 실행 시간: {:?}", self.total_duration);
        println!("\n체크포인트:");
        for (name, duration) in &self.checkpoints {
            println!("  {}: {:?}", name, duration);
        }
        println!("================================\n");
    }
}

/// 대용량 데이터를 위한 압축 유틸리티
pub struct DataCompressor;

impl DataCompressor {
    /// 메모리 효율적인 데이터 압축
    pub fn compress_batch<T: serde::Serialize>(data: &[T]) -> ErpResult<Vec<u8>> {
        let json = serde_json::to_vec(data)
            .map_err(|e| ErpError::internal(format!("JSON 직렬화 오류: {}", e)))?;

        // 간단한 압축 구현 (실제로는 더 고급 압축 알고리즘 사용)
        Ok(Self::simple_compress(&json))
    }

    /// 압축 해제
    pub fn decompress_batch<T: serde::de::DeserializeOwned>(
        compressed: &[u8],
    ) -> ErpResult<Vec<T>> {
        let decompressed = Self::simple_decompress(compressed);
        let data = serde_json::from_slice(&decompressed)
            .map_err(|e| ErpError::internal(format!("JSON 역직렬화 오류: {}", e)))?;
        Ok(data)
    }

    fn simple_compress(data: &[u8]) -> Vec<u8> {
        // 실제 구현에서는 zlib, gzip, lz4 등의 압축 라이브러리 사용
        // 여기서는 예시로 간단한 RLE(Run Length Encoding) 구현
        let mut compressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            let current_byte = data[i];
            let mut count = 1u8;

            while i + (count as usize) < data.len()
                && data[i + (count as usize)] == current_byte
                && count < 255
            {
                count += 1;
            }

            compressed.push(count);
            compressed.push(current_byte);
            i += count as usize;
        }

        compressed
    }

    fn simple_decompress(compressed: &[u8]) -> Vec<u8> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i + 1 < compressed.len() {
            let count = compressed[i];
            let byte_value = compressed[i + 1];

            for _ in 0..count {
                decompressed.push(byte_value);
            }

            i += 2;
        }

        decompressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_batch_processing() {
        let optimizer = PerformanceOptimizer::new(BatchConfig::default());
        let data: Vec<i32> = (1..=100).collect();

        let processor = |batch: Vec<i32>| async move {
            sleep(Duration::from_millis(10)).await;
            Ok(batch.into_iter().map(|x| x * 2).collect::<Vec<i32>>())
        };

        let results = optimizer.process_in_batches(data, processor).await.unwrap();

        assert_eq!(results.len(), 100);
        assert_eq!(results[0], 2);
        assert_eq!(results[99], 200);
    }

    #[tokio::test]
    async fn test_object_pool() {
        let pool: ObjectPool<String> = ObjectPool::new(5);

        assert_eq!(pool.size().await, 5);

        let obj = pool.get().await;
        assert_eq!(pool.size().await, 4);

        pool.return_object(obj).await;
        assert_eq!(pool.size().await, 5);
    }

    #[test]
    fn test_performance_measurer() {
        let mut measurer = PerformanceMeasurer::new();

        std::thread::sleep(std::time::Duration::from_millis(10));
        measurer.checkpoint("중간점");

        std::thread::sleep(std::time::Duration::from_millis(10));
        let report = measurer.finish("테스트 작업");

        assert!(report.total_duration.as_millis() >= 20);
        assert_eq!(report.checkpoints.len(), 2); // 중간점 + 완료
    }

    #[test]
    fn test_data_compression() {
        let data = vec![1, 1, 1, 2, 2, 3, 3, 3, 3];
        let compressed = DataCompressor::simple_compress(&data);
        let decompressed = DataCompressor::simple_decompress(&compressed);

        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len()); // 압축 효과 확인
    }
}
