//! Shared native governmental-subdivision augmentation. Missing membership stays unknown.
use std::collections::HashMap;
pub type EdgeMap=HashMap<(usize,usize),f64>;
pub struct SubdivisionWeighter {
    county: Vec<Option<String>>, // county FIPS per vertex — from GEOID[0..5]
    mcd: Vec<Option<String>>,    // county subdivision FIPS — from cousub file
    place: Vec<Option<String>>,  // incorporated place FIPS — from place file
    vtd: Vec<Option<String>>,    // voting district FIPS — from vtd file

    pub alpha_county: f64,
    pub alpha_mcd: f64,
    pub alpha_place: f64,
    pub alpha_vtd: f64,
}

impl SubdivisionWeighter {
    /// Phase 1: county only, derived free from GEOID.
    pub fn county_only(
        index_to_geoid: &HashMap<usize, String>,
        n_vertices: usize,
        alpha_county: f64,
    ) -> Self {
        let mut county = vec![None; n_vertices];
        for (&idx, geoid) in index_to_geoid {
            if geoid.len() >= 5 && idx < n_vertices {
                county[idx] = Some(geoid[..5].to_string());
            }
        }
        Self {
            county,
            mcd: vec![None; n_vertices],
            place: vec![None; n_vertices],
            vtd: vec![None; n_vertices],
            alpha_county,
            alpha_mcd: 0.0,
            alpha_place: 0.0,
            alpha_vtd: 0.0,
        }
    }

    /// Phase 2+: full hierarchy from pre-loaded TIGER join data.
    pub fn full(
        county: Vec<Option<String>>,
        mcd: Vec<Option<String>>,
        place: Vec<Option<String>>,
        vtd: Vec<Option<String>>,
        alpha_county: f64,
        alpha_mcd: f64,
        alpha_place: f64,
        alpha_vtd: f64,
    ) -> Self {
        Self {
            county,
            mcd,
            place,
            vtd,
            alpha_county,
            alpha_mcd,
            alpha_place,
            alpha_vtd,
        }
    }

    fn bonus(&self, u: usize, v: usize) -> f64 {
        let mut b = 0.0;
        if self.alpha_county > 1e-10 {
            if same(&self.county, u, v) {
                b += self.alpha_county;
            }
        }
        if self.alpha_mcd > 1e-10 {
            if same(&self.mcd, u, v) {
                b += self.alpha_mcd;
            }
        }
        if self.alpha_place > 1e-10 {
            if same(&self.place, u, v) {
                b += self.alpha_place;
            }
        }
        if self.alpha_vtd > 1e-10 {
            if same(&self.vtd, u, v) {
                b += self.alpha_vtd;
            }
        }
        b
    }
}

impl SubdivisionWeighter {
    pub fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights
            .into_iter()
            .map(|((u, v), w)| {
                let b = self.bonus(u, v);
                ((u, v), if b > 1e-10 { w * (1.0 + b) } else { w })
            })
            .collect()
    }
}

fn same(tbl: &[Option<String>], u: usize, v: usize) -> bool {
    match (tbl.get(u), tbl.get(v)) {
        (Some(Some(a)), Some(Some(b))) => a == b,
        _ => false,
    }
}

