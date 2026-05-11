#[derive(Debug, Clone)]
pub struct GraphFixture {
    pub name: &'static str,
    pub adjacency: Vec<Vec<usize>>,
    pub weights: Vec<i64>,
    pub k: usize,
    pub tolerance: f64,
}

pub fn path_6_k2() -> GraphFixture {
    GraphFixture {
        name: "path_6_k2",
        adjacency: path_adj(6),
        weights: vec![100; 6],
        k: 2,
        tolerance: 0.05,
    }
}

pub fn grid_3x3_k3() -> GraphFixture {
    GraphFixture {
        name: "grid_3x3_k3",
        adjacency: grid_adj(3, 3),
        weights: vec![100; 9],
        k: 3,
        tolerance: 0.34,
    }
}

pub fn two_clique_bridge_k2() -> GraphFixture {
    let mut adjacency = vec![Vec::new(); 6];
    for clique in [0usize, 3] {
        for a in clique..clique + 3 {
            for b in clique..clique + 3 {
                if a != b {
                    adjacency[a].push(b);
                }
            }
        }
    }
    adjacency[2].push(3);
    adjacency[3].push(2);
    GraphFixture {
        name: "two_clique_bridge_k2",
        adjacency,
        weights: vec![100; 6],
        k: 2,
        tolerance: 0.05,
    }
}

pub fn impossible_capacity_k3() -> GraphFixture {
    GraphFixture {
        name: "impossible_capacity_k3",
        adjacency: path_adj(4),
        weights: vec![100, 100, 100, 100],
        k: 3,
        tolerance: 0.01,
    }
}

pub fn disconnected_assignment() -> (Vec<Vec<usize>>, Vec<usize>) {
    (path_adj(4), vec![0, 1, 0, 1])
}

fn path_adj(n: usize) -> Vec<Vec<usize>> {
    (0..n)
        .map(|i| {
            let mut neighbors = Vec::new();
            if i > 0 {
                neighbors.push(i - 1);
            }
            if i + 1 < n {
                neighbors.push(i + 1);
            }
            neighbors
        })
        .collect()
}

fn grid_adj(rows: usize, cols: usize) -> Vec<Vec<usize>> {
    let mut adjacency = vec![Vec::new(); rows * cols];
    for row in 0..rows {
        for col in 0..cols {
            let node = row * cols + col;
            if row > 0 {
                adjacency[node].push((row - 1) * cols + col);
            }
            if row + 1 < rows {
                adjacency[node].push((row + 1) * cols + col);
            }
            if col > 0 {
                adjacency[node].push(row * cols + col - 1);
            }
            if col + 1 < cols {
                adjacency[node].push(row * cols + col + 1);
            }
        }
    }
    adjacency
}
