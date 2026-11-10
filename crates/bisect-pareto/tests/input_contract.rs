use bisect_pareto::{run_nsga2,ParetoConfig};
#[test]
fn rejects_inputs_before_search_or_allocation(){
 let graph=vec![vec![1],vec![0,2],vec![1]];
 let run=|a:&[Vec<usize>],p:&[i64],k,c,d:Option<&[f64]>|run_nsga2(a,p,k,d,None,&[],c);
 for (a,p,k,c) in [
  (vec![],vec![],1,ParetoConfig::default()),
  (graph.clone(),vec![1,1],2,ParetoConfig::default()),
  (graph.clone(),vec![-1,1,1],2,ParetoConfig::default()),
  (graph.clone(),vec![i64::MAX,1,1],2,ParetoConfig::default()),
  (graph.clone(),vec![0,0,0],2,ParetoConfig::default()),
  (graph.clone(),vec![1,1,1],4,ParetoConfig::default()),
  (graph.clone(),vec![1,1,1],2,ParetoConfig{n_population:1,..Default::default()}),
  (graph.clone(),vec![1,1,1],2,ParetoConfig{balance_tolerance:f64::NAN,..Default::default()}),
  (vec![vec![],vec![],vec![]],vec![1,1,1],2,ParetoConfig::default()),
  (vec![vec![1,1],vec![0,2],vec![1]],vec![1,1,1],2,ParetoConfig::default()),
  (vec![vec![1],vec![2],vec![1]],vec![1,1,1],2,ParetoConfig::default()),
 ]{assert!(run(&a,&p,k,c,None).is_err());}
 for shares in [vec![0.5],vec![0.5,1.1,0.5],vec![0.5,f64::NAN,0.5]]{assert!(run(&graph,&[1,1,1],2,ParetoConfig::default(),Some(&shares)).is_err());}
}

#[test]
fn front_rank_does_not_imply_balance(){
 let out=run_nsga2(&[vec![1],vec![0,2],vec![1]],&[1,1,1],2,None,None,&[],ParetoConfig{n_population:2,n_generations:0,balance_tolerance:0.0,..Default::default()}).unwrap();
 assert!(!out.frontier.is_empty());assert!(out.frontier.iter().all(|e|e.validity_status.as_deref()==Some("invalid")));
}
