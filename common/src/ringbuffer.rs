use std::collections::VecDeque;

// 一个不高兴动脑筋的ringbuffer实现

#[derive(Debug)]
pub struct RingBuffer<T> {
    queue: VecDeque<T>,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
        }
    }

    pub fn get(&self, index: isize) -> Option<&T> {
        if index >= 0 {
            self.queue.get(index as usize)
        } else {
            self.queue.get((self.queue.len() as isize + index) as usize)
        }
    }

    pub fn get_mut(&mut self, index: isize) -> Option<&mut T> {
        if index >= 0 {
            self.queue.get_mut(index as usize)
        } else {
            self.queue
                .get_mut((self.queue.len() as isize + index) as usize)
        }
    }

    pub fn push(&mut self, value: T) {
        let len = self.queue.len();
        let cap = self.queue.capacity();
        if len >= cap {
            self.queue.pop_front();
        }
        self.queue.push_back(value)
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.queue.pop_front()
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.queue.pop_back()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn clear(&mut self) {
        self.queue.clear()
    }
}

#[test]
fn test_ringbuffer() {
    let mut w = RingBuffer::<i32>::new(3);
    w.push(10);
    w.push(20);
    w.push(30);
    assert_eq!(*w.get(-1).unwrap(), 30i32);
    assert_eq!(*w.get(-2).unwrap(), 20i32);
    assert_eq!(*w.get(-3).unwrap(), 10i32);
    assert_eq!(w.len(), 3);
    assert_eq!(*w.get(0).unwrap(), 10i32);
    w.push(40);
    assert_eq!(w.len(), 3);
    assert_eq!(*w.get(-1).unwrap(), 40i32);
    assert_eq!(*w.get(-2).unwrap(), 30i32);
    assert_eq!(*w.get(-3).unwrap(), 20i32);
    assert_eq!(*w.get(0).unwrap(), 20i32);
    w.pop_front();
    assert_eq!(w.len(), 2);
    assert_eq!(*w.get(-1).unwrap(), 40i32);
    assert_eq!(*w.get(-2).unwrap(), 30i32);
    assert_eq!(*w.get(0).unwrap(), 30i32);
    assert_eq!(*w.get(1).unwrap(), 40i32);
}
