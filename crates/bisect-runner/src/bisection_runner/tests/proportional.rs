use super::*;

#[test]
fn proportional_children_follow_selected_seat_counts() {
    let (adj,pop) = small_grid(16,16);
    let votes = vec![750.0;pop.len()];
    let weights = HashMap::new();
    let result = run_all_splits_proportional(&adj,&pop,&weights,&votes,4,0.1,100,Some(42),None).unwrap();
    assert_eq!(result.len(),pop.len());
    for district in 1..=4 {
        let vertices: HashSet<_> = result.iter().filter_map(|(&v,&d)| (d==district).then_some(v)).collect();
        assert!(!vertices.is_empty());
        assert_eq!(connected_components_of(&adj,&vertices).len(),1);
        let district_pop: i64 = vertices.iter().map(|&v| pop[v]).sum();
        assert!((district_pop as f64/64000.0-1.0).abs() <= 0.1,"district {district}: {district_pop}");
    }
    for _ in 0..3 {
        assert_eq!(result,run_all_splits_proportional(&adj,&pop,&weights,&votes,4,0.1,100,Some(42),None).unwrap());
    }
}

#[test]
fn proportional_rejects_invalid_dimensions_and_votes_before_partitioning() {
    let (adj,pop) = small_grid(4,4);
    let weights = HashMap::new();
    let call = |votes:&[f64],k| run_all_splits_proportional(&adj,&pop,&weights,votes,k,0.1,100,Some(42),None);
    assert!(call(&[500.0],2).is_err());
    for value in [-0.0,-1.0,1001.0,f64::NAN,f64::INFINITY] {
        let mut votes=vec![500.0;16];votes[0]=value;assert!(call(&votes,2).is_err());
    }
    assert!(call(&vec![500.0;16],0).is_err());
    assert!(call(&vec![500.0;16],17).is_err());
    assert_eq!(call(&vec![0.0;16],1).unwrap().len(),16);
    let empty:Vec<Vec<usize>>=vec![];
    assert!(run_all_splits_proportional(&empty,&[],&weights,&[],1,0.1,100,Some(42),None).is_err());
}

#[test]
fn proportional_section_quota_rounding_keeps_two_positive_children() {
    for seats in [2,3,4,6,14,500] {
        for d in [0.0,0.01,0.25,0.5,0.75,0.99,1.0] {
            let (left,right)=proportional_section_seat_counts(d,seats).unwrap();
            assert_eq!(left+right,seats);assert!(left>0 && right>0);
        }
    }
    assert_eq!(proportional_section_seat_counts(0.99,2).unwrap(),(1,1));
    assert_eq!(proportional_section_seat_counts(0.75,4).unwrap(),(3,1));
    // At the geometric mean, keep the lower count; crossing it rounds up.
    let boundary=6.0_f64.sqrt()/4.0;
    assert_eq!(proportional_section_seat_counts(boundary,4).unwrap(),(2,2));
    assert_eq!(proportional_section_seat_counts(boundary+1e-12,4).unwrap(),(3,1));
    for d in [f64::NAN,f64::INFINITY,-0.1,1.1] {assert!(proportional_section_seat_counts(d,4).is_err());}
    assert!(proportional_section_seat_counts(0.5,1).is_err());
}

#[test]
fn proportional_section_validates_election_inputs_and_one_seat_scope() {
    let (adj,pop)=small_grid(4,4);let edges=HashMap::new();
    let dem=vec![300.0;16];let two=vec![500.0;16];
    let result=run_proportional_section(&adj,&pop,&dem,&two,&edges,1,0.1,100,1,1.1,None).unwrap();
    assert_eq!((result.1,result.2),(1,0));assert_eq!(result.4,0.6);
    let call=|d:&[f64],t:&[f64],eta,seeds|run_proportional_section(&adj,&pop,d,t,&edges,4,0.1,100,seeds,eta,None);
    assert!(call(&dem[..15],&two,1.1,1).is_err());
    assert!(call(&dem,&two[..15],1.1,1).is_err());
    for value in [-0.0,-1.0,501.0,f64::NAN,f64::INFINITY] {
        let mut bad=dem.clone();bad[0]=value;assert!(call(&bad,&two,1.1,1).is_err());
    }
    assert!(call(&vec![0.0;16],&two,1.1,1).is_err());
    assert!(call(&dem,&two,0.9,1).is_err());assert!(call(&dem,&two,1.1,0).is_err());
}

#[test]
fn proportional_intermediates_cover_unsplit_leaves() {
    let (adj,pop)=small_grid(16,16);let votes=vec![750.0;256];let weights=HashMap::new();
    let dir=tempfile::tempdir().unwrap();
    let result=run_all_splits_proportional(&adj,&pop,&weights,&votes,4,0.1,100,Some(42),Some(dir.path())).unwrap();
    for depth in 0..=3 {
        let bytes=std::fs::read(dir.path().join(format!("depth_{depth:02}/assignments.json"))).unwrap();
        let snapshot: HashMap<usize,usize>=serde_json::from_slice(&bytes).unwrap();
        assert_eq!(snapshot.len(),256);
        assert_eq!(snapshot.values().copied().collect::<HashSet<_>>().len(),depth+1);
        if depth==3 {assert_eq!(snapshot,result);}
    }
}

#[test]
fn proportional_section_executes_repeatable_root_and_descendants() {
    let (adj,pop)=small_grid(16,16);let votes=vec![250.0;256];let two=vec![500.0;256];
    let weights:HashMap<_,_>=adj.iter().enumerate().flat_map(|(u,ns)|ns.iter().filter(move |&&v|u<v).map(move |&v|((u,v),100.0+u as f64/1000.0))).collect();
    let run=||run_proportional_section(&adj,&pop,&votes,&two,&weights,4,0.1,100,3,1.1,None).unwrap();
    let result=run();assert_eq!((result.1,result.2),(2,2));assert_eq!(result.4,0.5);
    assert_eq!(result.0.len(),256);assert_eq!(result,run());
    for district in 1..=4 {
        let vertices:HashSet<_>=result.0.iter().filter_map(|(&v,&d)|(d==district).then_some(v)).collect();
        assert_eq!(connected_components_of(&adj,&vertices).len(),1);
        let total:i64=vertices.iter().map(|&v|pop[v]).sum();
        assert!((total as f64/64000.0-1.0).abs()<=0.1);
    }
}
