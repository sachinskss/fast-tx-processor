use std::sync::Arc;
use tokio::sync::Mutex;
use std::cmp::Ordering;
use crate::models::TransferRequest;

/// Transaction priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl Priority {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Priority::Low),
            2 => Some(Priority::Normal),
            3 => Some(Priority::High),
            4 => Some(Priority::Critical),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone)]
pub struct PrioritizedTransfer {
    pub request: TransferRequest,
    pub priority: Priority,
    pub enqueued_at: std::time::Instant,
}

impl Ord for PrioritizedTransfer {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then by FIFO (earlier enqueue time)
        match other.priority.cmp(&self.priority) {
            Ordering::Equal => self.enqueued_at.cmp(&other.enqueued_at),
            other_ordering => other_ordering,
        }
    }
}

impl PartialOrd for PrioritizedTransfer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PrioritizedTransfer {}

impl PartialEq for PrioritizedTransfer {
    fn eq(&self, other: &Self) -> bool {
        self.request.idempotency_key == other.request.idempotency_key
    }
}

/// Priority queue for managing transactions
pub struct PriorityQueue {
    queue: Vec<PrioritizedTransfer>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        PriorityQueue {
            queue: Vec::new(),
        }
    }

    pub fn push(&mut self, transfer: PrioritizedTransfer) {
        self.queue.push(transfer);
        // Simple O(n) insertion sort for simplicity
        let len = self.queue.len();
        for i in (0..len - 1).rev() {
            if self.queue[i] > self.queue[i + 1] {
                self.queue.swap(i, i + 1);
            }
        }
    }

    pub fn pop(&mut self) -> Option<PrioritizedTransfer> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn peek(&self) -> Option<&PrioritizedTransfer> {
        self.queue.first()
    }
}

pub type SharedPriorityQueue = Arc<Mutex<PriorityQueue>>;

pub fn create_shared_queue() -> SharedPriorityQueue {
    Arc::new(Mutex::new(PriorityQueue::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use uuid::Uuid;

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }

    #[test]
    fn test_priority_from_u8() {
        assert_eq!(Priority::from_u8(1), Some(Priority::Low));
        assert_eq!(Priority::from_u8(2), Some(Priority::Normal));
        assert_eq!(Priority::from_u8(3), Some(Priority::High));
        assert_eq!(Priority::from_u8(4), Some(Priority::Critical));
        assert_eq!(Priority::from_u8(5), None);
    }

    #[test]
    fn test_priority_queue_fifo() {
        let mut queue = PriorityQueue::new();
        
        let transfer1 = PrioritizedTransfer {
            request: TransferRequest {
                idempotency_key: Uuid::new_v4(),
                from_account: 1,
                to_account: 2,
                amount: 100,
                priority: 2,
            },
            priority: Priority::Normal,
            enqueued_at: std::time::Instant::now(),
        };
        
        thread::sleep(Duration::from_millis(1));
        
        let transfer2 = PrioritizedTransfer {
            request: TransferRequest {
                idempotency_key: Uuid::new_v4(),
                from_account: 1,
                to_account: 2,
                amount: 100,
                priority: 2,
            },
            priority: Priority::Normal,
            enqueued_at: std::time::Instant::now(),
        };

        queue.push(transfer1.clone());
        queue.push(transfer2.clone());

        assert_eq!(queue.pop().unwrap().request.idempotency_key, transfer1.request.idempotency_key);
        assert_eq!(queue.pop().unwrap().request.idempotency_key, transfer2.request.idempotency_key);
    }

    #[test]
    fn test_priority_queue_priority_ordering() {
        let mut queue = PriorityQueue::new();
        
        let key_low = Uuid::new_v4();
        let key_high = Uuid::new_v4();

        let transfer_low = PrioritizedTransfer {
            request: TransferRequest {
                idempotency_key: key_low,
                from_account: 1,
                to_account: 2,
                amount: 100,
                priority: 1,
            },
            priority: Priority::Low,
            enqueued_at: std::time::Instant::now(),
        };

        thread::sleep(Duration::from_millis(1));

        let transfer_high = PrioritizedTransfer {
            request: TransferRequest {
                idempotency_key: key_high,
                from_account: 1,
                to_account: 2,
                amount: 100,
                priority: 3,
            },
            priority: Priority::High,
            enqueued_at: std::time::Instant::now(),
        };

        queue.push(transfer_low);
        queue.push(transfer_high.clone());

        // High priority should be popped first even though it was added second
        assert_eq!(queue.pop().unwrap().request.idempotency_key, key_high);
    }

    #[test]
    fn test_priority_queue_empty() {
        let mut queue = PriorityQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.pop(), None);
    }
}