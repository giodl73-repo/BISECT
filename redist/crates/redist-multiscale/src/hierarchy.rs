//! HierarchyLevel: adjacency + population at one geographic resolution.

/// One level in the multi-resolution hierarchy (tract or block-group).
#[derive(Clone, Debug)]
pub struct HierarchyLevel {
    /// Adjacency graph at this resolution. adj[v] = neighbours of coarse node v.
    pub adj: Vec<Vec<u32>>,
    /// Population of each coarse node (sum of constituent fine tracts).
    pub pop: Vec<i64>,
    /// For each coarse node: list of fine tract indices it contains.
    pub coarse_to_fine: Vec<Vec<usize>>,
    /// For each fine tract: which coarse node it belongs to.
    pub fine_to_coarse: Vec<usize>,
}

impl HierarchyLevel {
    /// Number of nodes at this resolution.
    pub fn n(&self) -> usize { self.adj.len() }

    /// Build from fine-level adjacency + a partition of fine tracts into coarse nodes.
    /// coarse_partition[fine_tract] = coarse_node_index (0-based).
    pub fn from_fine(
        fine_adj: &[Vec<usize>],
        fine_pop: &[i64],
        coarse_partition: &[usize],
        n_coarse: usize,
    ) -> Self {
        // Build coarse_to_fine
        let mut coarse_to_fine = vec![vec![]; n_coarse];
        for (fine, &coarse) in coarse_partition.iter().enumerate() {
            coarse_to_fine[coarse].push(fine);
        }

        // Build coarse populations
        let pop: Vec<i64> = coarse_to_fine.iter()
            .map(|tracts| tracts.iter().map(|&t| fine_pop[t]).sum())
            .collect();

        // Build coarse adjacency (two coarse nodes are adjacent if any fine tracts are adjacent)
        let mut adj_set: Vec<std::collections::HashSet<u32>> = vec![std::collections::HashSet::new(); n_coarse];
        for (fine_v, fine_nbs) in fine_adj.iter().enumerate() {
            let cv = coarse_partition[fine_v];
            for &fine_nb in fine_nbs {
                let cnb = coarse_partition[fine_nb];
                if cv != cnb {
                    adj_set[cv].insert(cnb as u32);
                    adj_set[cnb].insert(cv as u32);
                }
            }
        }
        let adj: Vec<Vec<u32>> = adj_set.into_iter()
            .map(|s| { let mut v: Vec<u32> = s.into_iter().collect(); v.sort_unstable(); v })
            .collect();

        Self {
            adj,
            pop,
            coarse_to_fine,
            fine_to_coarse: coarse_partition.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_adj_usize(n: usize) -> Vec<Vec<usize>> {
        (0..n).map(|i| {
            let mut nb = Vec::new();
            if i > 0 { nb.push(i-1); }
            if i < n-1 { nb.push(i+1); }
            nb
        }).collect()
    }

    #[test]
    fn hierarchy_from_fine_4_tracts_2_coarse() {
        // 4-tract path: 0-1-2-3 grouped as {0,1} -> coarse 0, {2,3} -> coarse 1
        let fine_adj = path_adj_usize(4);
        let fine_pop = vec![100i64; 4];
        let partition = vec![0usize, 0, 1, 1];
        let level = HierarchyLevel::from_fine(&fine_adj, &fine_pop, &partition, 2);
        assert_eq!(level.n(), 2);
        assert_eq!(level.pop[0], 200);
        assert_eq!(level.pop[1], 200);
        assert!(level.adj[0].contains(&1), "coarse 0 adjacent to coarse 1");
        assert_eq!(level.coarse_to_fine[0], vec![0, 1]);
        assert_eq!(level.fine_to_coarse[2], 1);
    }

    #[test]
    fn hierarchy_coarse_pop_sums_to_fine_total() {
        let fine_adj = path_adj_usize(6);
        let fine_pop = vec![100i64, 200, 150, 75, 125, 50];
        let partition = vec![0, 0, 1, 1, 2, 2];
        let level = HierarchyLevel::from_fine(&fine_adj, &fine_pop, &partition, 3);
        let total_fine: i64 = fine_pop.iter().sum();
        let total_coarse: i64 = level.pop.iter().sum();
        assert_eq!(total_fine, total_coarse);
    }
}
