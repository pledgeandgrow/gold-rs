//! Goal 216: GPU resource pooling.
//!
//! Pool GPU buffers, textures, and pipelines in wgpu. Reuse across components
//! instead of creating/destroying. Reduces GPU memory pressure and allocation latency.

use std::collections::HashMap;
use std::sync::Mutex;

/// The type of GPU resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuResourceType {
    /// A vertex buffer.
    VertexBuffer,
    /// An index buffer.
    IndexBuffer,
    /// A uniform buffer.
    UniformBuffer,
    /// A storage buffer.
    StorageBuffer,
    /// A 2D texture.
    Texture2d,
    /// A depth texture.
    TextureDepth,
    /// A render pipeline.
    RenderPipeline,
    /// A compute pipeline.
    ComputePipeline,
    /// A bind group.
    BindGroup,
    /// A sampler.
    Sampler,
}

impl GpuResourceType {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            GpuResourceType::VertexBuffer => "VertexBuffer",
            GpuResourceType::IndexBuffer => "IndexBuffer",
            GpuResourceType::UniformBuffer => "UniformBuffer",
            GpuResourceType::StorageBuffer => "StorageBuffer",
            GpuResourceType::Texture2d => "Texture2d",
            GpuResourceType::TextureDepth => "TextureDepth",
            GpuResourceType::RenderPipeline => "RenderPipeline",
            GpuResourceType::ComputePipeline => "ComputePipeline",
            GpuResourceType::BindGroup => "BindGroup",
            GpuResourceType::Sampler => "Sampler",
        }
    }

    /// Check if this resource is a buffer.
    pub fn is_buffer(&self) -> bool {
        matches!(self, GpuResourceType::VertexBuffer | GpuResourceType::IndexBuffer | GpuResourceType::UniformBuffer | GpuResourceType::StorageBuffer)
    }

    /// Check if this resource is a texture.
    pub fn is_texture(&self) -> bool {
        matches!(self, GpuResourceType::Texture2d | GpuResourceType::TextureDepth)
    }

    /// Check if this resource is a pipeline.
    pub fn is_pipeline(&self) -> bool {
        matches!(self, GpuResourceType::RenderPipeline | GpuResourceType::ComputePipeline)
    }
}

/// A pooled GPU resource.
#[derive(Debug, Clone)]
pub struct PooledGpuResource {
    /// The resource ID.
    pub id: u64,
    /// The resource type.
    pub resource_type: GpuResourceType,
    /// The size in bytes (for buffers/textures).
    pub size_bytes: u64,
    /// Whether the resource is currently in use.
    pub in_use: bool,
    /// The number of times this resource has been reused.
    pub reuse_count: u32,
}

impl PooledGpuResource {
    /// Create a new pooled resource.
    pub fn new(id: u64, resource_type: GpuResourceType, size_bytes: u64) -> Self {
        Self {
            id,
            resource_type,
            size_bytes,
            in_use: false,
            reuse_count: 0,
        }
    }
}

/// Pool statistics.
#[derive(Debug, Clone, Default)]
pub struct GpuPoolStats {
    /// Total allocations.
    pub allocations: u64,
    /// Total reuses.
    pub reuses: u64,
    /// Total deallocations.
    pub deallocations: u64,
    /// Current pool size.
    pub pool_size: usize,
    /// Number of resources in use.
    pub in_use: u32,
    /// Total memory in pool (bytes).
    pub total_memory: u64,
}

impl GpuPoolStats {
    /// Get the reuse rate (0.0-1.0).
    pub fn reuse_rate(&self) -> f64 {
        let total = self.allocations + self.reuses;
        if total == 0 {
            return 0.0;
        }
        self.reuses as f64 / total as f64
    }
}

/// A pool for a specific resource type.
struct ResourceTypePool {
    resources: Vec<PooledGpuResource>,
    next_id: u64,
}

/// The GPU resource pool — manages reusable GPU resources.
pub struct GpuResourcePool {
    pools: Mutex<HashMap<GpuResourceType, ResourceTypePool>>,
    stats: Mutex<GpuPoolStats>,
    max_pool_size_per_type: usize,
}

impl GpuResourcePool {
    /// Create a new GPU resource pool.
    pub fn new(max_pool_size_per_type: usize) -> Self {
        Self {
            pools: Mutex::new(HashMap::new()),
            stats: Mutex::new(GpuPoolStats::default()),
            max_pool_size_per_type,
        }
    }

    /// Acquire a resource from the pool (or create a new one).
    pub fn acquire(&self, resource_type: GpuResourceType, size_bytes: u64) -> PooledGpuResource {
        let mut pools = self.pools.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let pool = pools.entry(resource_type).or_insert_with(|| ResourceTypePool {
            resources: Vec::new(),
            next_id: 0,
        });

        // Try to find an idle resource of matching size
        if let Some(resource) = pool.resources.iter_mut().find(|r| !r.in_use && r.size_bytes == size_bytes) {
            resource.in_use = true;
            resource.reuse_count += 1;
            stats.reuses += 1;
            stats.in_use += 1;
            return resource.clone();
        }

        // Try to find an idle resource with >= size (can sub-allocate)
        if let Some(resource) = pool.resources.iter_mut().find(|r| !r.in_use && r.size_bytes >= size_bytes) {
            resource.in_use = true;
            resource.reuse_count += 1;
            stats.reuses += 1;
            stats.in_use += 1;
            return resource.clone();
        }

        // Create a new resource
        let id = pool.next_id;
        pool.next_id += 1;
        let mut resource = PooledGpuResource::new(id, resource_type, size_bytes);
        resource.in_use = true;
        pool.resources.push(resource.clone());

        stats.allocations += 1;
        stats.in_use += 1;
        stats.pool_size = pool.resources.len();
        stats.total_memory += size_bytes;

        resource
    }

    /// Release a resource back to the pool.
    pub fn release(&self, resource_id: u64, resource_type: GpuResourceType) -> bool {
        let mut pools = self.pools.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let pool = match pools.get_mut(&resource_type) {
            Some(p) => p,
            None => return false,
        };

        if let Some(resource) = pool.resources.iter_mut().find(|r| r.id == resource_id) {
            if resource.in_use {
                resource.in_use = false;
                stats.in_use = stats.in_use.saturating_sub(1);
                return true;
            }
        }
        false
    }

    /// Shrink the pool by removing idle resources.
    pub fn shrink(&self, resource_type: GpuResourceType) -> usize {
        let mut pools = self.pools.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let pool = match pools.get_mut(&resource_type) {
            Some(p) => p,
            None => return 0,
        };

        let before = pool.resources.len();
        let mut freed_memory = 0u64;
        pool.resources.retain(|r| {
            if !r.in_use {
                freed_memory += r.size_bytes;
                false
            } else {
                true
            }
        });
        let removed = before - pool.resources.len();

        stats.deallocations += removed as u64;
        stats.pool_size = pool.resources.len();
        stats.total_memory = stats.total_memory.saturating_sub(freed_memory);

        removed
    }

    /// Shrink all pools.
    pub fn shrink_all(&self) -> usize {
        let types: Vec<GpuResourceType> = self.pools.lock().unwrap().keys().copied().collect();
        types.iter().map(|t| self.shrink(*t)).sum()
    }

    /// Get the pool size for a resource type.
    pub fn pool_size(&self, resource_type: GpuResourceType) -> usize {
        self.pools.lock().unwrap().get(&resource_type).map(|p| p.resources.len()).unwrap_or(0)
    }

    /// Get the number of idle resources for a type.
    pub fn idle_count(&self, resource_type: GpuResourceType) -> usize {
        self.pools.lock().unwrap()
            .get(&resource_type)
            .map(|p| p.resources.iter().filter(|r| !r.in_use).count())
            .unwrap_or(0)
    }

    /// Get the number of in-use resources for a type.
    pub fn in_use_count(&self, resource_type: GpuResourceType) -> usize {
        self.pools.lock().unwrap()
            .get(&resource_type)
            .map(|p| p.resources.iter().filter(|r| r.in_use).count())
            .unwrap_or(0)
    }

    /// Get pool statistics.
    pub fn stats(&self) -> GpuPoolStats {
        self.stats.lock().unwrap().clone()
    }

    /// Clear all pools.
    pub fn clear(&self) {
        self.pools.lock().unwrap().clear();
        let mut stats = self.stats.lock().unwrap();
        stats.pool_size = 0;
        stats.in_use = 0;
        stats.total_memory = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_resource_type_display_name() {
        assert_eq!(GpuResourceType::VertexBuffer.display_name(), "VertexBuffer");
        assert_eq!(GpuResourceType::Texture2d.display_name(), "Texture2d");
    }

    #[test]
    fn test_gpu_resource_type_is_buffer() {
        assert!(GpuResourceType::VertexBuffer.is_buffer());
        assert!(GpuResourceType::UniformBuffer.is_buffer());
        assert!(!GpuResourceType::Texture2d.is_buffer());
    }

    #[test]
    fn test_gpu_resource_type_is_texture() {
        assert!(GpuResourceType::Texture2d.is_texture());
        assert!(!GpuResourceType::VertexBuffer.is_texture());
    }

    #[test]
    fn test_gpu_resource_type_is_pipeline() {
        assert!(GpuResourceType::RenderPipeline.is_pipeline());
        assert!(!GpuResourceType::VertexBuffer.is_pipeline());
    }

    #[test]
    fn test_pooled_gpu_resource_new() {
        let r = PooledGpuResource::new(1, GpuResourceType::VertexBuffer, 1024);
        assert_eq!(r.id, 1);
        assert!(!r.in_use);
        assert_eq!(r.reuse_count, 0);
    }

    #[test]
    fn test_gpu_pool_stats_reuse_rate() {
        let mut stats = GpuPoolStats::default();
        stats.allocations = 30;
        stats.reuses = 70;
        assert_eq!(stats.reuse_rate(), 0.7);
    }

    #[test]
    fn test_gpu_pool_stats_empty_reuse_rate() {
        let stats = GpuPoolStats::default();
        assert_eq!(stats.reuse_rate(), 0.0);
    }

    #[test]
    fn test_gpu_pool_acquire_new() {
        let pool = GpuResourcePool::new(100);
        let resource = pool.acquire(GpuResourceType::VertexBuffer, 1024);
        assert_eq!(resource.size_bytes, 1024);
        assert!(resource.in_use);

        let stats = pool.stats();
        assert_eq!(stats.allocations, 1);
        assert_eq!(stats.in_use, 1);
    }

    #[test]
    fn test_gpu_pool_acquire_reuse() {
        let pool = GpuResourcePool::new(100);
        let r1 = pool.acquire(GpuResourceType::VertexBuffer, 1024);
        pool.release(r1.id, GpuResourceType::VertexBuffer);

        let r2 = pool.acquire(GpuResourceType::VertexBuffer, 1024);
        assert_eq!(r2.id, r1.id);
        assert_eq!(r2.reuse_count, 1);

        let stats = pool.stats();
        assert_eq!(stats.allocations, 1);
        assert_eq!(stats.reuses, 1);
    }

    #[test]
    fn test_gpu_pool_release() {
        let pool = GpuResourcePool::new(100);
        let r = pool.acquire(GpuResourceType::UniformBuffer, 512);
        assert!(pool.release(r.id, GpuResourceType::UniformBuffer));
        assert_eq!(pool.in_use_count(GpuResourceType::UniformBuffer), 0);
    }

    #[test]
    fn test_gpu_pool_release_not_found() {
        let pool = GpuResourcePool::new(100);
        assert!(!pool.release(999, GpuResourceType::VertexBuffer));
    }

    #[test]
    fn test_gpu_pool_release_already_idle() {
        let pool = GpuResourcePool::new(100);
        let r = pool.acquire(GpuResourceType::VertexBuffer, 256);
        pool.release(r.id, GpuResourceType::VertexBuffer);
        assert!(!pool.release(r.id, GpuResourceType::VertexBuffer));
    }

    #[test]
    fn test_gpu_pool_shrink() {
        let pool = GpuResourcePool::new(100);
        let r1 = pool.acquire(GpuResourceType::VertexBuffer, 256);
        let r2 = pool.acquire(GpuResourceType::VertexBuffer, 512);
        pool.release(r1.id, GpuResourceType::VertexBuffer);
        pool.release(r2.id, GpuResourceType::VertexBuffer);

        let removed = pool.shrink(GpuResourceType::VertexBuffer);
        assert_eq!(removed, 2);
        assert_eq!(pool.pool_size(GpuResourceType::VertexBuffer), 0);
    }

    #[test]
    fn test_gpu_pool_shrink_keeps_in_use() {
        let pool = GpuResourcePool::new(100);
        let r1 = pool.acquire(GpuResourceType::VertexBuffer, 256);
        let _r2 = pool.acquire(GpuResourceType::VertexBuffer, 512);
        pool.release(r1.id, GpuResourceType::VertexBuffer);

        let removed = pool.shrink(GpuResourceType::VertexBuffer);
        assert_eq!(removed, 1);
        assert_eq!(pool.pool_size(GpuResourceType::VertexBuffer), 1);
    }

    #[test]
    fn test_gpu_pool_shrink_all() {
        let pool = GpuResourcePool::new(100);
        let r1 = pool.acquire(GpuResourceType::VertexBuffer, 256);
        let r2 = pool.acquire(GpuResourceType::Texture2d, 1024);
        pool.release(r1.id, GpuResourceType::VertexBuffer);
        pool.release(r2.id, GpuResourceType::Texture2d);

        let removed = pool.shrink_all();
        assert_eq!(removed, 2);
    }

    #[test]
    fn test_gpu_pool_idle_count() {
        let pool = GpuResourcePool::new(100);
        let r1 = pool.acquire(GpuResourceType::VertexBuffer, 256);
        let _r2 = pool.acquire(GpuResourceType::VertexBuffer, 512);
        pool.release(r1.id, GpuResourceType::VertexBuffer);

        assert_eq!(pool.idle_count(GpuResourceType::VertexBuffer), 1);
        assert_eq!(pool.in_use_count(GpuResourceType::VertexBuffer), 1);
    }

    #[test]
    fn test_gpu_pool_clear() {
        let pool = GpuResourcePool::new(100);
        let _r = pool.acquire(GpuResourceType::VertexBuffer, 256);
        pool.clear();
        assert_eq!(pool.pool_size(GpuResourceType::VertexBuffer), 0);
    }

    #[test]
    fn test_gpu_pool_acquire_larger_resource() {
        let pool = GpuResourcePool::new(100);
        let r1 = pool.acquire(GpuResourceType::VertexBuffer, 1024);
        pool.release(r1.id, GpuResourceType::VertexBuffer);

        // Request smaller size — should reuse the larger one
        let r2 = pool.acquire(GpuResourceType::VertexBuffer, 512);
        assert_eq!(r2.id, r1.id);
        assert_eq!(r2.reuse_count, 1);
    }
}
