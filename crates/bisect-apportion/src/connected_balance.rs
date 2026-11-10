//! Bounded population refinement of an already connected partition.
//! Moves preserve both connected parts. Neutral population-error moves may
//! carry excess toward a deficit through a full intermediate part.
use metis_core::CsrGraph;
use std::collections::VecDeque;

pub(crate) fn refine(g:&CsrGraph,mut a:Vec<u32>,shares:&[u32],ufactor:u32,niter:u32)->Vec<u32>{
 let k=shares.len();if k<2||shares.contains(&0){return a;}
 let Ok(initial)=metis_core::Partition::new(a.clone(),k as u32)else{return a;};
 if initial.validate_for_graph(g).is_err()||metis_core::check_contiguity(g,&initial).is_err(){return a;}
 let s:i128=shares.iter().map(|&v|i128::from(v)).sum();
 let total:i128=g.vwgt().iter().map(|&v|i128::from(v)).sum();
 let mut pop=vec![0i128;k];let mut counts=vec![0usize;k];
 for(v,&p)in a.iter().enumerate(){pop[p as usize]+=i128::from(g.vwgt()[v]);counts[p as usize]+=1;}
 let mut seen=vec![0u32;g.n()];let mut mark=0u32;
 for _ in 0..g.n().saturating_mul(niter.max(1) as usize){
  let diff:Vec<_>=pop.iter().zip(shares).map(|(&p,&t)|p*s-total*i128::from(t)).collect();
  if diff.iter().zip(shares).all(|(&d,&t)|d.abs()*1000<=total*i128::from(t)*i128::from(ufactor)){break;}
  let mut part_adj=vec![Vec::new();k];
  for v in 0..g.n(){for &u in &g.adjncy()[g.xadj()[v] as usize..g.xadj()[v+1] as usize]{let(f,t)=(a[v]as usize,a[u as usize]as usize);if f!=t{part_adj[f].push(t);}}}
  for ns in &mut part_adj{ns.sort_unstable();ns.dedup();}
  let mut distance=vec![usize::MAX;k];let mut queue=VecDeque::new();
  for p in 0..k{if diff[p]<0{distance[p]=0;queue.push_back(p);}}
  while let Some(p)=queue.pop_front(){for &q in &part_adj[p]{if distance[q]==usize::MAX{distance[q]=distance[p]+1;queue.push_back(q);}}}
  let mut candidates=Vec::new();
  for v in 0..g.n(){let from=a[v]as usize;if counts[from]<=1{continue;}
   let w=i128::from(g.vwgt()[v])*s;let mut targets=Vec::new();
   for &u in &g.adjncy()[g.xadj()[v]as usize..g.xadj()[v+1]as usize]{let to=a[u as usize]as usize;if to!=from{targets.push(to);}}
   targets.sort_unstable();targets.dedup();
   for to in targets{
    // Expanded pair-wise square difference avoids subtracting large squares.
    let Some(delta)=w.checked_mul(diff[to]-diff[from]).and_then(|x|x.checked_add(w.checked_mul(w)?)).and_then(|x|x.checked_mul(2))else{continue;};
    let neutral=delta==0&&diff[from]>0&&w<=diff[from]&&distance[to]<distance[from];
    if delta>0||delta==0&&!neutral{continue;}
    let cut:i64=(g.xadj()[v]as usize..g.xadj()[v+1]as usize).map(|j|{let t=a[g.adjncy()[j]as usize];let weight=g.adjwgt().map_or(1,|weights|weights[j])as i64;if t==from as u32{weight}else if t==to as u32{-weight}else{0}}).sum();
    candidates.push((delta,cut,v,from,to));
   }
  }
  candidates.sort_unstable();let mut moved=false;
  for (_,_,v,from,to)in candidates{
   mark=mark.wrapping_add(1);if mark==0{seen.fill(0);mark=1;}
   let Some(start)=(g.xadj()[v]as usize..g.xadj()[v+1]as usize).map(|j|g.adjncy()[j]as usize).find(|&u|u!=v&&a[u]as usize==from)else{continue;};
   let mut stack=vec![start];seen[start]=mark;let mut reached=0;
   while let Some(u)=stack.pop(){reached+=1;for &z in &g.adjncy()[g.xadj()[u]as usize..g.xadj()[u+1]as usize]{let z=z as usize;if z!=v&&a[z]as usize==from&&seen[z]!=mark{seen[z]=mark;stack.push(z);}}}
   if reached!=counts[from]-1{continue;}
   a[v]=to as u32;let w=i128::from(g.vwgt()[v]);pop[from]-=w;pop[to]+=w;counts[from]-=1;counts[to]+=1;moved=true;break;
  }
  if !moved{break;}
 }
 a
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn carries_excess_through_full_middle_part_without_disconnect(){
  let n=24;let mut x=vec![0];let mut edges=Vec::new();for v in 0..n{if v>0{edges.push(v-1);}if v+1<n{edges.push(v+1);}x.push(edges.len()as u32);}
  let g=CsrGraph::new(x,edges,1,vec![5;n as usize],None).unwrap();
  let a=(0..n).map(|v|if v<11{0}else if v<19{1}else{2}).collect();
  let result=refine(&g,a,&[1,1,1],1,10);let mut populations=[0;3];for &p in &result{populations[p as usize]+=5;}
  assert_eq!(populations,[40,40,40]);let p=metis_core::Partition::new(result,3).unwrap();assert!(metis_core::check_contiguity(&g,&p).is_ok());
 }
 #[test]fn never_moves_an_articulation_vertex(){
  let g=CsrGraph::new(vec![0,4,5,6,7,9,10],vec![1,2,3,4,0,0,0,0,5,4],1,vec![20,10,10,10,10,10],None).unwrap();
  let a=vec![0,0,0,0,1,1];let result=refine(&g,a.clone(),&[1,1],1,10);assert_eq!(result,a);let p=metis_core::Partition::new(result,2).unwrap();assert!(metis_core::check_contiguity(&g,&p).is_ok());
 }
}
