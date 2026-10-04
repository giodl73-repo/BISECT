use super::*;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ConvergenceSummary {
    pub schema_version: &'static str,
    pub method: &'static str,
    pub objective: &'static str,
    #[serde(serialize_with="seed_string")]
    pub base_seed: u64,
    #[serde(serialize_with="seed_string")]
    pub selected_seed: u64,
    pub selected_index: usize,
    pub attempted: usize,
    pub rejected: usize,
    pub threshold: usize,
    pub consecutive_non_improving: usize,
    pub seed_limit: Option<usize>,
    pub halt: &'static str,
    pub best_normalized_cut: f64,
    #[serde(skip_serializing_if="Option::is_none")]
    pub metis_refinement: Option<serde_json::Value>,
}

fn seed_string<S:serde::Serializer>(seed:&u64,serializer:S)->Result<S::Ok,S::Error>{
    serializer.serialize_str(&seed.to_string())
}

/// Fixed floor/ceil recursive schedule: each edge is charged at the split
/// separating its leaf districts, divided by sqrt(min(left_k,right_k)).
/// Canonical edge order makes the floating-point score portable and repeatable.
pub fn recursive_normalized_cut(
    adjacency: &[Vec<usize>], edge_weights: &HashMap<(usize,usize),f64>,
    assignment: &HashMap<usize,usize>, k: usize,
) -> Result<f64,String> {
    if k==0 || assignment.len()!=adjacency.len() || (0..adjacency.len()).any(|i| !assignment.contains_key(&i) || !(1..=k).contains(&assignment[&i])) {
        return Err("Convergence score requires complete, in-range assignments.".into());
    }
    let mut edges:Vec<_>=adjacency.iter().enumerate().flat_map(|(u,ns)| ns.iter().filter(move |&&v| u<v).map(move |&v|(u,v))).collect();
    edges.sort_unstable();edges.dedup();
    let mut total=0.0;
    for (u,v) in edges {
        if v>=adjacency.len(){return Err("Invalid convergence adjacency.".into());}
        let (du,dv)=(assignment[&u],assignment[&v]);if du==dv{continue;}
        let mut first=1;let mut count=k;
        let denominator=loop {
            let left=count/2;let middle=first+left;
            if (du<middle)!=(dv<middle){break (left.min(count-left) as f64).sqrt();}
            if du<middle{count=left;}else{first=middle;count-=left;}
        };
        let weight=edge_weights.get(&(u,v)).copied().unwrap_or(1.0);
        if !weight.is_finite() || weight<=0.0 {return Err("Invalid convergence boundary weight.".into());}
        total+=weight/denominator;
    }
    if !total.is_finite(){return Err("Convergence score exceeds finite numeric range.".into());}
    Ok(total)
}

fn sweep<F>(base_seed:u64,threshold:usize,limit:Option<usize>,mut candidate:F)
    -> Result<(HashMap<usize,usize>,ConvergenceSummary),String>
where F:FnMut(u64)->Result<(HashMap<usize,usize>,f64),String> {
    if threshold==0 || limit==Some(0){return Err("Convergence requires positive threshold and seed limit.".into());}
    let mut best:Option<(usize,f64,HashMap<usize,usize>)>=None;
    let (mut attempted,mut rejected,mut stale)=(0,0,0);
    while stale<threshold && limit.map_or(true,|cap|attempted<cap) {
        let index=attempted;attempted+=1;
        match candidate(base_seed.wrapping_add(index as u64)) {
            Ok((plan,score)) if score.is_finite() && score>=0.0 => {
                if best.as_ref().map_or(true,|(_,best_score,_)|score<*best_score) {
                    best=Some((index,score,plan));stale=0;
                }else{stale+=1;}
            }
            _=>{rejected+=1;stale+=1;}
        }
    }
    let Some((selected_index,score,plan))=best else{return Err(format!("Convergence search found no feasible candidate in {attempted} seeds."));};
    let summary=ConvergenceSummary {
        schema_version:"bisect-convergence-summary-v1",method:"convergence",
        objective:"recursive-normalized-weighted-cut",base_seed,
        selected_seed:base_seed.wrapping_add(selected_index as u64),selected_index,
        attempted,rejected,threshold,consecutive_non_improving:stale,seed_limit:limit,
        halt:if stale>=threshold{"threshold"}else{"seed-limit"},best_normalized_cut:score,metis_refinement:None,
    };
    Ok((plan,summary))
}

/// Search complete standard-bisection plans from consecutive seeds. Reject
/// candidates that violate the final population or connectivity contract.
/// A capped search returns its best observed plan with halt=seed-limit,
/// never describing that limit as convergence or global optimality.
#[allow(clippy::too_many_arguments)]
pub fn run_all_splits_convergence(
    adjacency:&[Vec<usize>],population:&[i64],edges:&HashMap<(usize,usize),f64>,
    k:usize,tolerance:f64,niter:u32,base_seed:u64,threshold:usize,limit:Option<usize>,
) -> Result<(HashMap<usize,usize>,ConvergenceSummary),String> {
    run_all_splits_convergence_profile(adjacency,population,edges,k,tolerance,niter,base_seed,threshold,limit,None)
}

/// Tuned refinement preserves the full-plan score and stopping schedule.
#[allow(clippy::too_many_arguments)]
pub fn run_all_splits_convergence_tuned(
    adjacency:&[Vec<usize>],population:&[i64],edges:&HashMap<(usize,usize),f64>,
    k:usize,tolerance:f64,niter:u32,base_seed:u64,threshold:usize,limit:Option<usize>,
    objective:&str,trials:u32,
) -> Result<(HashMap<usize,usize>,ConvergenceSummary),String> {
    if !["cut","volume"].contains(&objective) || !(1..=100).contains(&trials) {
        return Err("METIS refinement requires cut or volume and 1..=100 trials.".into());
    }
    run_all_splits_convergence_profile(adjacency,population,edges,k,tolerance,niter,base_seed,threshold,limit,Some((objective,trials)))
}

#[allow(clippy::too_many_arguments)]
fn run_all_splits_convergence_profile(
    adjacency:&[Vec<usize>],population:&[i64],edges:&HashMap<(usize,usize),f64>,
    k:usize,tolerance:f64,niter:u32,base_seed:u64,threshold:usize,limit:Option<usize>,
    refinement:Option<(&str,u32)>,
) -> Result<(HashMap<usize,usize>,ConvergenceSummary),String> {
    if k==0 || k>adjacency.len() || adjacency.len()!=population.len()
        || !tolerance.is_finite() || !(0.0..=1.0).contains(&tolerance)
        || population.iter().any(|&p|p<0)
        || population.iter().try_fold(0i64,|total,&p|total.checked_add(p)).filter(|&p|p>0).is_none() {
        return Err("Invalid convergence graph, population or tolerance.".into());
    }
    let total=population.iter().sum::<i64>() as f64;let ideal=total/k as f64;
    let (plan,mut summary)=sweep(base_seed,threshold,limit,|seed|{
        let (objective,trials)=refinement.unwrap_or(("cut",1));
        let plan=run_all_splits_tuned(adjacency,population,edges,k,tolerance,niter,Some(seed),None,objective,trials)?;
        if plan.len()!=population.len() || (0..population.len()).any(|i|!plan.contains_key(&i)||!(1..=k).contains(&plan[&i])) {return Err("Incomplete convergence candidate.".into());}
        for district in 1..=k {
            let vertices:HashSet<_>=(0..population.len()).filter(|i|plan[i]==district).collect();
            let pop=vertices.iter().map(|&v|population[v]).sum::<i64>();
            if vertices.is_empty() || !is_connected_subset(adjacency,&vertices) || ((pop as f64)/ideal-1.0).abs()>tolerance {
                return Err("Infeasible convergence candidate.".into());
            }
        }
        let score=recursive_normalized_cut(adjacency,edges,&plan,k)?;
        Ok((plan,score))
    })?;
    if let Some((objective,trials))=refinement {
        summary.metis_refinement=Some(serde_json::json!({"objective":objective,"internal_trials_per_candidate":trials,"iterations":niter,"scope":"each-floor-ceil-tree-node","internal_trial_selection":"population-excess-then-edge-cut","post_refinement":"native-population-rebalance"}));
    }
    Ok((plan,summary))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tuning_rejects_invalid_controls_before_search() {
        for (objective,trials) in [("bad",1),("cut",0),("volume",101)] {
            assert!(run_all_splits_convergence_tuned(&[vec![]],&[1],&HashMap::new(),1,0.1,10,42,2,Some(3),objective,trials).unwrap_err().contains("METIS refinement"));
        }
    }
    #[test]
    fn legacy_summary_omits_refinement() {
        let (_,summary)=sweep(42,1,Some(1),|_|Ok((HashMap::new(),1.0))).unwrap();
        assert!(serde_json::to_value(summary).unwrap().get("metis_refinement").is_none());
    }
    #[test]
    fn improvements_reset_tail_and_ties_keep_earliest() {
        let costs=[10.0,9.0,9.0,8.0,8.0,8.0,1.0];let mut index=0;
        let (plan,summary)=sweep(42,2,Some(20),|_|{let i=index;index+=1;Ok((HashMap::from([(0,i)]),costs[i]))}).unwrap();
        assert_eq!(summary.attempted,6);assert_eq!(summary.selected_seed,45);
        assert_eq!(summary.consecutive_non_improving,2);assert_eq!(summary.halt,"threshold");assert_eq!(plan[&0],3);
    }
    #[test]
    fn seed_limit_is_not_convergence_and_wrap_is_fixed_width() {
        let (_,summary)=sweep(u64::MAX,4,Some(2),|seed|Ok((HashMap::new(),if seed==0{1.0}else{2.0}))).unwrap();
        assert_eq!(summary.selected_seed,0);assert_eq!(summary.selected_index,1);assert_eq!(summary.halt,"seed-limit");
    }
    #[test]
    fn rejection_does_not_replace_incumbent() {
        let mut index=0;
        let (_,summary)=sweep(42,2,None,|_|{index+=1;if index==1{Ok((HashMap::new(),1.0))}else{Err("rejected".into())}}).unwrap();
        assert_eq!(summary.rejected,2);assert_eq!(summary.attempted,3);assert_eq!(summary.selected_index,0);
        assert!(sweep(42,2,Some(20),|_|Err("rejected".into())).is_err());
        assert!(sweep(42,0,None,|_|unreachable!()).is_err());
    }
    #[test]
    fn normalized_cut_uses_each_edges_recursive_level() {
        let adjacency=vec![vec![1],vec![0,2],vec![1,3],vec![2]];
        let plan=HashMap::from([(0,1),(1,2),(2,3),(3,4)]);
        let score=recursive_normalized_cut(&adjacency,&HashMap::new(),&plan,4).unwrap();
        assert!((score-(2.0+1.0/2.0_f64.sqrt())).abs()<1e-12);
        let invalid=HashMap::from([(0,0)]);assert!(recursive_normalized_cut(&adjacency,&HashMap::new(),&invalid,4).is_err());
    }
}
