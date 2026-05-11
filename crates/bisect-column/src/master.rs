use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub id: usize,
    pub units: Vec<usize>,
    pub population: i64,
    pub edge_cut: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MasterProblem {
    pub unit_count: usize,
    pub k: usize,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MasterSolution {
    pub selected_column_ids: Vec<usize>,
    pub assignment: Vec<usize>,
    pub objective: usize,
}

impl MasterProblem {
    pub fn new(unit_count: usize, k: usize, columns: Vec<Column>) -> Self {
        Self {
            unit_count,
            k,
            columns,
        }
    }

    pub fn formulation_size(&self) -> MasterFormulationSize {
        let incidence_nnz = self.columns.iter().map(|column| column.units.len()).sum();
        MasterFormulationSize {
            unit_constraints: self.unit_count,
            district_constraints: 1,
            columns: self.columns.len(),
            incidence_nnz,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MasterFormulationSize {
    pub unit_constraints: usize,
    pub district_constraints: usize,
    pub columns: usize,
    pub incidence_nnz: usize,
}
