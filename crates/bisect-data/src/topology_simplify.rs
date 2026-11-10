//! Conservative shared-boundary display simplification. Source WKB is unchanged.
use geo::Contains;
use geo_types::{Coord, Line, LineString, MultiPolygon, Point, Polygon};
use rstar::{RTree, RTreeObject, AABB};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct Segment { index:usize, line:Line<f64> }
impl RTreeObject for Segment {
 type Envelope=AABB<[f64;2]>;
 fn envelope(&self)->Self::Envelope {bounds(self.line.start,self.line.end)}
}
#[derive(Clone)]
struct Vertex { index:usize, coordinate:Coord<f64> }
impl RTreeObject for Vertex {
 type Envelope=AABB<[f64;2]>;
 fn envelope(&self)->Self::Envelope {AABB::from_point([self.coordinate.x,self.coordinate.y])}
}
fn bounds(a:Coord<f64>,b:Coord<f64>)->AABB<[f64;2]>{AABB::from_corners([a.x.min(b.x),a.y.min(b.y)],[a.x.max(b.x),a.y.max(b.y)])}
fn key(c:Coord<f64>)->(u64,u64){((if c.x==0.{0.}else{c.x}).to_bits(),(if c.y==0.{0.}else{c.y}).to_bits())}
fn charge(work:&mut usize,n:usize)->Result<(),String>{*work=work.checked_add(n).ok_or("Display simplification work overflow")?;if *work>200_000_000{return Err("Shared-boundary display simplification exceeds its 200-million work budget".into());}Ok(())}
fn distance_squared(point:Coord<f64>,a:Coord<f64>,b:Coord<f64>)->f64{
 let dx=b.x-a.x;let dy=b.y-a.y;let length=dx*dx+dy*dy;
 let t=if length==0.{0.}else{((point.x-a.x)*dx+(point.y-a.y)*dy)/length}.clamp(0.,1.);
 let x=point.x-(a.x+t*dx);let y=point.y-(a.y+t*dy);x*x+y*y
}

struct Network {
 coordinates:Vec<Coord<f64>>, edges:Vec<(usize,usize)>, neighbours:Vec<Vec<(usize,usize)>>,
 segments:RTree<Segment>, vertices:RTree<Vertex>, accepted:RTree<Segment>, work:usize,
}
impl Network {
 fn shortcut(&mut self,path:&[usize])->Result<bool,String>{
  use geo::line_intersection::{line_intersection,LineIntersection};
  let a=self.coordinates[path[0]];let b=self.coordinates[*path.last().unwrap()];let chord=Line::new(a,b);
  for previous in self.accepted.locate_in_envelope_intersecting(&bounds(a,b)){
   charge(&mut self.work,1)?;
   match line_intersection(chord,previous.line){
    None=>{},
    Some(LineIntersection::SinglePoint{intersection,is_proper:false}) if (intersection==a||intersection==b)&&(intersection==previous.line.start||intersection==previous.line.end)=>{},
    _=>return Ok(false),
   }
  }
  let own:HashSet<_>=path.windows(2).map(|pair|(pair[0].min(pair[1]),pair[0].max(pair[1]))).collect();
  let straight=path.iter().all(|&id|distance_squared(self.coordinates[id],a,b)==0.);
  for segment in self.segments.locate_in_envelope_intersecting(&bounds(a,b)){
   charge(&mut self.work,1)?;
   let edge=self.edges[segment.index];let belonging=own.contains(&edge);
   match line_intersection(chord,segment.line){
    None=>{},
    Some(LineIntersection::SinglePoint{intersection,is_proper})=>{
     if belonging&&straight {continue;}
     if is_proper||!((intersection==a||intersection==b)&&(intersection==segment.line.start||intersection==segment.line.end)){return Ok(false);}
    },
    Some(LineIntersection::Collinear{intersection})=>{
     if belonging&&straight {continue;}
     if intersection.start!=intersection.end||!((intersection.start==a||intersection.start==b)&&(intersection.start==segment.line.start||intersection.start==segment.line.end)){return Ok(false);}
    },
   }
  }
  // Noncrossing shortcuts can still swallow a whole disconnected hole/island.
  // Keep every foreign source vertex outside the swept region as well.
  if !straight {
   let mut ring=path.iter().map(|&id|self.coordinates[id]).collect::<Vec<_>>();ring.push(a);
   let swept=Polygon::new(LineString(ring),vec![]);let ids:HashSet<_>=path.iter().copied().collect();
   let mut minimum=a;let mut maximum=a;
   for &id in path{let c=self.coordinates[id];minimum.x=minimum.x.min(c.x);minimum.y=minimum.y.min(c.y);maximum.x=maximum.x.max(c.x);maximum.y=maximum.y.max(c.y);}
   for vertex in self.vertices.locate_in_envelope_intersecting(&bounds(minimum,maximum)){
    charge(&mut self.work,1)?;if !ids.contains(&vertex.index)&&swept.contains(&Point(vertex.coordinate)){return Ok(false);}
   }
  }
  self.accepted.insert(Segment{index:0,line:chord});Ok(true)
 }
 fn simplify(&mut self,path:&[usize],epsilon:f64,retained:&mut HashSet<usize>)->Result<(),String>{
  let mut pending=vec![(0,path.len()-1)];
  while let Some((start,end))=pending.pop(){
   retained.insert(path[start]);retained.insert(path[end]);if end-start<=1{continue;}
   charge(&mut self.work,end-start)?;
   let a=self.coordinates[path[start]];let b=self.coordinates[path[end]];
   let mut split=start+1;let mut maximum=-1.;
   for i in start+1..end{let distance=distance_squared(self.coordinates[path[i]],a,b);if distance>maximum{maximum=distance;split=i;}}
   if maximum<=epsilon*epsilon&&self.shortcut(&path[start..=end])?{continue;}
   pending.push((split,end));pending.push((start,split));
  }Ok(())
 }
}

/// All coincident segment chains share the same retained vertices. Original
/// edges remain in the guard index, making decisions conservative after other
/// shortcuts. This is a display approximation, not repaired source geometry.
pub fn simplify_shared(polygons:&[MultiPolygon<f64>],epsilon:f64)->Result<Vec<MultiPolygon<f64>>,String>{
 if !epsilon.is_finite()||epsilon<=0.{return Err("Display tolerance must be finite and positive".into());}
 let mut point_ids=HashMap::new();let mut coordinates=vec![];let mut edges=vec![];let mut edge_ids=HashMap::new();let mut encoded=vec![];let mut positions=0usize;
 for geometry in polygons{
  let mut parts=vec![];
  for polygon in &geometry.0{
   let mut rings=vec![];
   for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()){
    let mut ids=vec![];
    for &c in &ring.0{
     if !c.x.is_finite()||!c.y.is_finite(){return Err("Invalid display source coordinate".into());}
     positions+=1;if positions>8_000_000{return Err("Shared-boundary display source exceeds 8,000,000 positions".into());}
     let id=*point_ids.entry(key(c)).or_insert_with(||{let id=coordinates.len();coordinates.push(c);id});
     if ids.last()!=Some(&id){ids.push(id);}
    }
    if ids.len()<4||ids.first()!=ids.last(){return Err("Shared-boundary display requires closed source rings".into());}
    for pair in ids.windows(2){let edge=(pair[0].min(pair[1]),pair[0].max(pair[1]));edge_ids.entry(edge).or_insert_with(||{let id=edges.len();edges.push(edge);id});}
    rings.push(ids);
   }parts.push(rings);
  }encoded.push(parts);
 }
 let mut neighbours=vec![vec![];coordinates.len()];
 for (index,&(a,b)) in edges.iter().enumerate(){neighbours[a].push((b,index));neighbours[b].push((a,index));}
 let segments=RTree::bulk_load(edges.iter().enumerate().map(|(index,&(a,b))|Segment{index,line:Line::new(coordinates[a],coordinates[b])}).collect());
 let vertices=RTree::bulk_load(coordinates.iter().enumerate().map(|(index,&coordinate)|Vertex{index,coordinate}).collect());
 let mut network=Network{coordinates,edges,neighbours,segments,vertices,accepted:RTree::new(),work:0};let mut visited=vec![false;network.edges.len()];let mut retained=HashSet::new();
 // Simplification must not turn a crossing source into an apparently valid
 // partition. Shared and partially noded coincident edges remain allowed.
 for segment in network.segments.iter(){for other in network.segments.locate_in_envelope_intersecting(&segment.envelope()){
  charge(&mut network.work,1)?;if other.index>=segment.index{continue;}
  if matches!(geo::line_intersection::line_intersection(segment.line,other.line),Some(geo::line_intersection::LineIntersection::SinglePoint{is_proper:true,..})){
   return Err("Shared-boundary display source has crossing boundaries; simplification does not repair invalid sources".into());
  }
 }}
 // Junctions split arcs. Process them first, then isolated degree-two cycles.
 for junction_pass in [true,false]{for initial in 0..network.edges.len(){
  if visited[initial]{continue;}let (a,b)=network.edges[initial];
  let start=if network.neighbours[a].len()!=2{a}else if network.neighbours[b].len()!=2{b}else{if junction_pass{continue;}a};
  let mut path=vec![start];let mut current=start;let mut edge=initial;
  loop{
   visited[edge]=true;let (u,v)=network.edges[edge];let next=if u==current{v}else{u};path.push(next);
   if next==start||network.neighbours[next].len()!=2{break;}
   let next_edge=network.neighbours[next].iter().find(|(_,index)|*index!=edge).unwrap().1;
   if visited[next_edge]{break;}current=next;edge=next_edge;
  }
  if path.first()==path.last(){
   let anchor=network.coordinates[start];let split=(1..path.len()-1).max_by(|&i,&j|distance_squared(network.coordinates[path[i]],anchor,anchor).total_cmp(&distance_squared(network.coordinates[path[j]],anchor,anchor))).ok_or("Empty display cycle")?;
   let other=network.coordinates[path[split]];
   let third=(1..path.len()-1).filter(|&i|i!=split).max_by(|&i,&j|distance_squared(network.coordinates[path[i]],anchor,other).total_cmp(&distance_squared(network.coordinates[path[j]],anchor,other))).ok_or("Degenerate display cycle")?;
   let first=split.min(third);let second=split.max(third);
   network.simplify(&path[..=first],epsilon,&mut retained)?;network.simplify(&path[first..=second],epsilon,&mut retained)?;network.simplify(&path[second..],epsilon,&mut retained)?;
  }else{network.simplify(&path,epsilon,&mut retained)?;}
 }}
 encoded.into_iter().map(|parts|parts.into_iter().map(|rings|{
  let mut lines=rings.into_iter().map(|ids|{let mut points=ids[..ids.len()-1].iter().filter(|id|retained.contains(id)).map(|id|network.coordinates[*id]).collect::<Vec<_>>();if let Some(first)=points.first().copied(){points.push(first);}if points.len()<4{return Err("Display simplification collapsed a source ring".into());}Ok(LineString(points))}).collect::<Result<Vec<_>,String>>()?;
  let shell=lines.remove(0);Ok(Polygon::new(shell,lines))
 }).collect::<Result<Vec<_>,String>>().map(MultiPolygon)).collect()
}

#[cfg(test)]
mod tests {
 use super::*;
 use geo::Intersects;
 #[test]fn shared_boundary_is_reused_and_a_disconnected_hole_is_not_swept(){
  let a=Polygon::new(LineString::from(vec![(0.,0.),(1.,0.),(1.001,0.5),(1.,1.),(0.,1.),(0.,0.)]),vec![]);
  let b=Polygon::new(LineString::from(vec![(1.,0.),(2.,0.),(2.,1.),(1.,1.),(1.001,0.5),(1.,0.)]),vec![]);
  let simplified=simplify_shared(&[MultiPolygon(vec![a]),MultiPolygon(vec![b])],0.01).unwrap();
  assert_eq!(simplified[0].0[0].exterior().0.len(),5);assert_eq!(simplified[1].0[0].exterior().0.len(),5);
  assert!(!simplified[0].intersects(&Point::new(1.0005,0.5)));
  let shell=Polygon::new(LineString::from(vec![(0.,0.),(1.,0.),(1.01,0.5),(1.,1.),(0.,1.),(0.,0.)]),vec![LineString::from(vec![(1.002,0.45),(1.004,0.45),(1.004,0.55),(1.002,0.55),(1.002,0.45)])]);
  let guarded=simplify_shared(&[MultiPolygon(vec![shell])],0.02).unwrap();assert!(guarded[0].0[0].exterior().0.contains(&Coord{x:1.01,y:0.5}));
 }
 #[test]fn crossing_source_is_rejected_instead_of_simplified_into_a_triangle(){
  let crossing=Polygon::new(LineString::from(vec![(0.,0.),(1.,1.),(0.,1.),(1.,0.),(0.,0.)]),vec![]);
  assert!(simplify_shared(&[MultiPolygon(vec![crossing])],2.).unwrap_err().contains("crossing boundaries"));
 }
}
