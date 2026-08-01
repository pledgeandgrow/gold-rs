//! Goal 218: Render coalescing.
//!
//! When multiple signals update in the same frame, coalesce all DOM mutations
//! into a single batch. Frame-aware scheduling using `requestAnimationFrame`.

use std::collections::HashMap;
use std::sync::Mutex;

/// A pending DOM mutation.
#[derive(Debug, Clone, PartialEq)]
pub struct DomMutation {
    /// The node ID to mutate.
    pub node_id: u64,
    /// The mutation type.
    pub mutation_type: MutationType,
    /// The new value (if applicable).
    pub value: Option<String>,
}

/// The type of DOM mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MutationType {
    /// Set text content.
    SetText,
    /// Set an attribute.
    SetAttribute,
    /// Remove an attribute.
    RemoveAttribute,
    /// Set a style property.
    SetStyle,
    /// Insert a child node.
    InsertChild,
    /// Remove a child node.
    RemoveChild,
    /// Replace a child node.
    ReplaceChild,
    /// Move a child node.
    MoveChild,
}

impl MutationType {
    /// Check if this mutation can be coalesced with another.
    pub fn can_coalesce(&self, other: &MutationType) -> bool {
        matches!(
            (self, other),
            (MutationType::SetText, MutationType::SetText)
                | (MutationType::SetAttribute, MutationType::SetAttribute)
                | (MutationType::SetStyle, MutationType::SetStyle)
        )
    }
}

/// A coalesced batch of DOM mutations.
#[derive(Debug, Clone)]
pub struct MutationBatch {
    /// The mutations in this batch.
    pub mutations: Vec<DomMutation>,
    /// The frame number this batch was scheduled for.
    pub frame_number: u64,
}

impl MutationBatch {
    /// Create a new empty batch.
    pub fn new(frame_number: u64) -> Self {
        Self {
            mutations: Vec::new(),
            frame_number,
        }
    }

    /// Add a mutation to the batch (with coalescing).
    pub fn add(&mut self, mutation: DomMutation) {
        // Try to coalesce with an existing mutation
        for existing in self.mutations.iter_mut() {
            if existing.node_id == mutation.node_id
                && existing.mutation_type == mutation.mutation_type
                && existing.mutation_type.can_coalesce(&mutation.mutation_type)
            {
                // Replace the value (last write wins)
                existing.value = mutation.value;
                return;
            }
        }
        self.mutations.push(mutation);
    }

    /// Get the number of mutations in the batch.
    pub fn len(&self) -> usize {
        self.mutations.len()
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.mutations.is_empty()
    }

    /// Clear the batch.
    pub fn clear(&mut self) {
        self.mutations.clear();
    }
}

/// Coalescing statistics.
#[derive(Debug, Clone, Default)]
pub struct CoalescingStats {
    /// Total mutations submitted.
    pub total_mutations: u64,
    /// Total mutations after coalescing.
    pub coalesced_mutations: u64,
    /// Total batches flushed.
    pub batches_flushed: u64,
    /// Total frames scheduled.
    pub frames_scheduled: u64,
    /// Mutations saved by coalescing.
    pub mutations_saved: u64,
}

impl CoalescingStats {
    /// Get the coalescing rate (0.0-1.0).
    pub fn coalescing_rate(&self) -> f64 {
        if self.total_mutations == 0 {
            return 0.0;
        }
        self.mutations_saved as f64 / self.total_mutations as f64
    }
}

/// The render coalescer — batches DOM mutations per frame.
pub struct RenderCoalescer {
    pending: Mutex<Vec<DomMutation>>,
    current_batch: Mutex<MutationBatch>,
    current_frame: Mutex<u64>,
    stats: Mutex<CoalescingStats>,
    scheduled: Mutex<bool>,
}

impl RenderCoalescer {
    /// Create a new render coalescer.
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(Vec::new()),
            current_batch: Mutex::new(MutationBatch::new(0)),
            current_frame: Mutex::new(0),
            stats: Mutex::new(CoalescingStats::default()),
            scheduled: Mutex::new(false),
        }
    }

    /// Submit a DOM mutation for coalescing.
    pub fn submit(&self, mutation: DomMutation) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_mutations += 1;
        drop(stats);

        self.pending.lock().unwrap().push(mutation);

        if !*self.scheduled.lock().unwrap() {
            *self.scheduled.lock().unwrap() = true;
            *self.current_frame.lock().unwrap() += 1;
            self.stats.lock().unwrap().frames_scheduled += 1;
        }
    }

    /// Flush the pending mutations into a coalesced batch.
    pub fn flush(&self) -> MutationBatch {
        let frame = *self.current_frame.lock().unwrap();
        let mut batch = MutationBatch::new(frame);

        let mut pending = self.pending.lock().unwrap();
        let pending_count = pending.len();

        for mutation in pending.drain(..) {
            batch.add(mutation);
        }

        let coalesced_count = batch.len();
        let saved = pending_count.saturating_sub(coalesced_count);

        let mut stats = self.stats.lock().unwrap();
        stats.coalesced_mutations += coalesced_count as u64;
        stats.mutations_saved += saved as u64;
        stats.batches_flushed += 1;

        *self.scheduled.lock().unwrap() = false;

        batch
    }

    /// Check if a flush is pending.
    pub fn is_pending(&self) -> bool {
        !self.pending.lock().unwrap().is_empty()
    }

    /// Get the number of pending mutations.
    pub fn pending_count(&self) -> usize {
        self.pending.lock().unwrap().len()
    }

    /// Get the current frame number.
    pub fn current_frame(&self) -> u64 {
        *self.current_frame.lock().unwrap()
    }

    /// Get coalescing statistics.
    pub fn stats(&self) -> CoalescingStats {
        self.stats.lock().unwrap().clone()
    }

    /// Clear all pending mutations.
    pub fn clear(&self) {
        self.pending.lock().unwrap().clear();
        *self.scheduled.lock().unwrap() = false;
    }

    /// Generate the JavaScript frame-aware scheduling script.
    pub fn generate_coalescing_script(&self) -> String {
        r#"(function(){var s=window.__ryeCoalesce={pending:[],scheduled:false,frame:0};
s.schedule=function(){if(s.scheduled)return;s.scheduled=true;
requestAnimationFrame(function(){s.frame++;var batch=s.pending;s.pending=[];s.scheduled=false;
for(var i=0;i<batch.length;i++){var m=batch[i];var el=document.querySelector('[data-rye-id="'+m.nodeId+'"]');
if(el){if(m.type==='setText'){el.textContent=m.value;}else if(m.type==='setAttribute'){el.setAttribute(m.attr,m.value);}
else if(m.type==='removeAttribute'){el.removeAttribute(m.attr);}}}batch.length;});};};
})();"#.to_string()
    }
}

impl Default for RenderCoalescer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mutation(node_id: u64, mtype: MutationType, value: &str) -> DomMutation {
        DomMutation {
            node_id,
            mutation_type: mtype,
            value: Some(value.to_string()),
        }
    }

    #[test]
    fn test_mutation_type_can_coalesce() {
        assert!(MutationType::SetText.can_coalesce(&MutationType::SetText));
        assert!(MutationType::SetAttribute.can_coalesce(&MutationType::SetAttribute));
        assert!(!MutationType::SetText.can_coalesce(&MutationType::SetAttribute));
        assert!(!MutationType::InsertChild.can_coalesce(&MutationType::RemoveChild));
    }

    #[test]
    fn test_mutation_batch_new() {
        let batch = MutationBatch::new(1);
        assert_eq!(batch.frame_number, 1);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_mutation_batch_add() {
        let mut batch = MutationBatch::new(0);
        batch.add(make_mutation(1, MutationType::SetText, "hello"));
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_mutation_batch_coalesce_same_node_same_type() {
        let mut batch = MutationBatch::new(0);
        batch.add(make_mutation(1, MutationType::SetText, "first"));
        batch.add(make_mutation(1, MutationType::SetText, "second"));
        assert_eq!(batch.len(), 1); // Coalesced
        assert_eq!(batch.mutations[0].value, Some("second".to_string())); // Last write wins
    }

    #[test]
    fn test_mutation_batch_no_coalesce_different_nodes() {
        let mut batch = MutationBatch::new(0);
        batch.add(make_mutation(1, MutationType::SetText, "a"));
        batch.add(make_mutation(2, MutationType::SetText, "b"));
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_mutation_batch_no_coalesce_different_types() {
        let mut batch = MutationBatch::new(0);
        batch.add(make_mutation(1, MutationType::SetText, "a"));
        batch.add(make_mutation(1, MutationType::SetStyle, "color:red"));
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_mutation_batch_clear() {
        let mut batch = MutationBatch::new(0);
        batch.add(make_mutation(1, MutationType::SetText, "a"));
        batch.clear();
        assert!(batch.is_empty());
    }

    #[test]
    fn test_coalescer_submit() {
        let coalescer = RenderCoalescer::new();
        coalescer.submit(make_mutation(1, MutationType::SetText, "hello"));
        assert!(coalescer.is_pending());
        assert_eq!(coalescer.pending_count(), 1);
    }

    #[test]
    fn test_coalescer_flush() {
        let coalescer = RenderCoalescer::new();
        coalescer.submit(make_mutation(1, MutationType::SetText, "a"));
        coalescer.submit(make_mutation(1, MutationType::SetText, "b"));
        coalescer.submit(make_mutation(2, MutationType::SetText, "c"));

        let batch = coalescer.flush();
        assert_eq!(batch.len(), 2); // Two coalesced into one for node 1
        assert!(!coalescer.is_pending());
    }

    #[test]
    fn test_coalescer_flush_empty() {
        let coalescer = RenderCoalescer::new();
        let batch = coalescer.flush();
        assert!(batch.is_empty());
    }

    #[test]
    fn test_coalescer_stats() {
        let coalescer = RenderCoalescer::new();
        coalescer.submit(make_mutation(1, MutationType::SetText, "a"));
        coalescer.submit(make_mutation(1, MutationType::SetText, "b"));
        coalescer.flush();

        let stats = coalescer.stats();
        assert_eq!(stats.total_mutations, 2);
        assert_eq!(stats.coalesced_mutations, 1);
        assert_eq!(stats.mutations_saved, 1);
        assert_eq!(stats.batches_flushed, 1);
    }

    #[test]
    fn test_coalescer_coalescing_rate() {
        let coalescer = RenderCoalescer::new();
        coalescer.submit(make_mutation(1, MutationType::SetText, "a"));
        coalescer.submit(make_mutation(1, MutationType::SetText, "b"));
        coalescer.flush();

        let stats = coalescer.stats();
        assert_eq!(stats.coalescing_rate(), 0.5);
    }

    #[test]
    fn test_coalescer_clear() {
        let coalescer = RenderCoalescer::new();
        coalescer.submit(make_mutation(1, MutationType::SetText, "a"));
        coalescer.clear();
        assert!(!coalescer.is_pending());
    }

    #[test]
    fn test_coalescer_current_frame() {
        let coalescer = RenderCoalescer::new();
        coalescer.submit(make_mutation(1, MutationType::SetText, "a"));
        let frame1 = coalescer.current_frame();
        coalescer.flush();
        coalescer.submit(make_mutation(1, MutationType::SetText, "b"));
        let frame2 = coalescer.current_frame();
        assert!(frame2 > frame1);
    }

    #[test]
    fn test_coalescer_generate_script() {
        let coalescer = RenderCoalescer::new();
        let script = coalescer.generate_coalescing_script();
        assert!(script.contains("requestAnimationFrame"));
        assert!(script.contains("pending"));
    }
}
