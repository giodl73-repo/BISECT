/// Bucket-sort gain table for O(1) max-gain lookup.
///
/// Gains ∈ `[-max_gain, +max_gain]`. Uses offset indexing so all array
/// indices are non-negative: `bucket_idx(gain) = gain + max_gain`.
pub struct GainTable {
    buckets:    Vec<Vec<u32>>,
    position:   Vec<Option<(i32, usize)>>,
    pub max_gain: i32,
    top_bucket:   i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_table_max_is_correct() {
        let mut gt = GainTable::new(5, 10);
        gt.insert(0, 3);
        gt.insert(1, -2);
        gt.insert(2, 7);
        let (v, g) = gt.peek_max().unwrap();
        assert_eq!((v, g), (2, 7));
    }

    #[test]
    fn gain_table_remove_max() {
        let mut gt = GainTable::new(5, 10);
        gt.insert(0, 5); gt.insert(1, 3); gt.insert(2, 8);
        let (v, g) = gt.pop_max().unwrap();
        assert_eq!((v, g), (2, 8));
        let (v2, g2) = gt.pop_max().unwrap();
        assert_eq!((v2, g2), (0, 5));
    }

    #[test]
    fn gain_table_update_gain() {
        let mut gt = GainTable::new(3, 5);
        gt.insert(0, 2);
        gt.update(0, 4);
        let (v, g) = gt.peek_max().unwrap();
        assert_eq!((v, g), (0, 4));
    }

    #[test]
    fn gain_table_negative_gain() {
        let mut gt = GainTable::new(3, 5);
        gt.insert(0, -3);
        gt.insert(1, -1);
        let (v, _) = gt.peek_max().unwrap();
        assert_eq!(v, 1, "vertex 1 has gain -1 which is > -3");
    }

    #[test]
    fn gain_table_empty_returns_none() {
        let gt = GainTable::new(4, 10);
        assert!(gt.peek_max().is_none());
        assert!(gt.is_empty());
    }
}

impl GainTable {
    pub fn new(n_vertices: usize, max_gain: i32) -> Self {
        let size = (2 * max_gain + 1) as usize;
        Self {
            buckets:    vec![Vec::new(); size],
            position:   vec![None; n_vertices],
            max_gain,
            top_bucket: i32::MIN,
        }
    }

    fn bucket_idx(&self, gain: i32) -> usize {
        (gain + self.max_gain) as usize
    }

    pub fn insert(&mut self, vertex: u32, gain: i32) {
        let bi = self.bucket_idx(gain);
        let pos = self.buckets[bi].len();
        self.buckets[bi].push(vertex);
        self.position[vertex as usize] = Some((gain, pos));
        if gain > self.top_bucket { self.top_bucket = gain; }
    }

    pub fn remove(&mut self, vertex: u32) {
        if let Some((gain, pos)) = self.position[vertex as usize].take() {
            let bi = self.bucket_idx(gain);
            let last = self.buckets[bi].len() - 1;
            if pos < last {
                let swap_v = self.buckets[bi][last];
                self.buckets[bi][pos] = swap_v;
                self.position[swap_v as usize] = Some((gain, pos));
            }
            self.buckets[bi].pop();
        }
    }

    pub fn update(&mut self, vertex: u32, new_gain: i32) {
        self.remove(vertex);
        self.insert(vertex, new_gain);
    }

    pub fn peek_max(&self) -> Option<(u32, i32)> {
        let mut g = self.top_bucket;
        while g >= -self.max_gain {
            let bi = self.bucket_idx(g);
            if let Some(&v) = self.buckets[bi].last() { return Some((v, g)); }
            g -= 1;
        }
        None
    }

    pub fn pop_max(&mut self) -> Option<(u32, i32)> {
        let (v, g) = self.peek_max()?;
        self.remove(v);
        self.top_bucket = g;
        Some((v, g))
    }

    pub fn is_empty(&self) -> bool { self.peek_max().is_none() }

    pub fn contains(&self, vertex: u32) -> bool {
        self.position[vertex as usize].is_some()
    }
}
