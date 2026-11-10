//! Shared native administrative-zone co-membership kernel. No source loading or spatial join.
use std::collections::HashMap;
pub type EdgeMap=HashMap<(usize,usize),f64>;

pub struct ZoneMembershipWeighter {
    /// Per-edge precomputed co-membership score in [0.0, 1.0].
    scores: HashMap<(usize, usize), f64>,
    /// Boost factor (default 1.0). alpha=1.0 means full-shared zones -> 2x weight.
    alpha: f64,
}

impl ZoneMembershipWeighter {
    /// Build from per-node zone vectors. Each node has a `Vec<Option<String>>`
    /// where index = zone_type, value = zone_id (`None` if zone unavailable).
    ///
    /// `score(u,v) = shared / available` where `available` counts zone types where
    /// both nodes have a non-None assignment, and `shared` counts types where both
    /// assignments are equal.
    pub fn from_zone_assignments(
        assignments: &[Vec<Option<String>>],
        edges: &[(usize, usize)],
        alpha: f64,
    ) -> Self {
        let mut scores = HashMap::new();
        for &(u, v) in edges {
            let zu = &assignments[u];
            let zv = &assignments[v];
            let n = zu.len().max(zv.len());
            if n == 0 {
                scores.insert((u, v), 0.0);
                continue;
            }
            let mut shared = 0usize;
            let mut available = 0usize;
            for i in 0..n.min(zu.len()).min(zv.len()) {
                if let (Some(a), Some(b)) = (&zu[i], &zv[i]) {
                    available += 1;
                    if a == b {
                        shared += 1;
                    }
                }
            }
            let score = if available == 0 {
                0.0
            } else {
                shared as f64 / available as f64
            };
            scores.insert((u, v), score);
        }
        Self { scores, alpha }
    }
}

impl ZoneMembershipWeighter {
    pub fn apply(&self, weights: EdgeMap) -> EdgeMap {
        weights
            .into_iter()
            .map(|((u, v), w)| {
                let score = self.scores.get(&(u, v)).copied().unwrap_or(0.0);
                // Additive boost: w * (1 + alpha * score)
                // score=1 (all zones shared) and alpha=1.0 -> w * 2 (max bond)
                // score=0 (no zones shared) -> w * 1 (unmodified)
                let w_new = w * (1.0 + self.alpha * score);
                ((u, v), w_new)
            })
            .collect()
    }
}


#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn observed_intersection_and_ragged_rows_keep_native_scores(){
  let a=vec![vec![Some("school".into()),None,Some("".into())],vec![Some("school".into()),Some("utility".into()),Some("".into())],vec![Some("other".into())],vec![]];
  let edges=vec![(0,1),(1,2),(2,3),(0,3)];
  let result=ZoneMembershipWeighter::from_zone_assignments(&a,&edges,1.).apply(HashMap::from([((0,1),100.),((1,2),200.),((2,3),300.),((0,3),0.)]));
  assert_eq!(result[&(0,1)],200.);assert_eq!(result[&(1,2)],200.);assert_eq!(result[&(2,3)],300.);assert_eq!(result[&(0,3)],0.);
 }
 #[test]
 fn fractional_shared_score_and_unspecified_orientation(){
  let a=vec![vec![Some("a".into()),Some("x".into()),None],vec![Some("a".into()),Some("y".into()),Some("z".into())]];
  let result=ZoneMembershipWeighter::from_zone_assignments(&a,&[(0,1)],0.5).apply(HashMap::from([((0,1),80.),((1,0),80.)]));
  assert_eq!(result[&(0,1)],100.);assert_eq!(result[&(1,0)],80.);
 }
 #[test]
 fn zero_alpha_and_no_observations_are_identity(){
  let map=HashMap::from([((0,1),10.)]);
  for (a,alpha) in [(vec![vec![Some("a".into())],vec![Some("a".into())]],0.),(vec![vec![None],vec![None]],100.)] {
   assert_eq!(ZoneMembershipWeighter::from_zone_assignments(&a,&[(0,1)],alpha).apply(map.clone()),map);
  }
 }
}
