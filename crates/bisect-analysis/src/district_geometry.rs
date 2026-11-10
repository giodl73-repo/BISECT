//! Portable native dissolve and compactness reporting. No file or renderer dependency.
use geo::{Area, BooleanOps, Contains, Intersects, Relate, ConvexHull, BoundingRect};
use geo_types::{Coord, Geometry, LineString, MultiPolygon, Polygon};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use rstar::{RTree, RTreeObject, AABB};
pub const MAX_UNIT_GEOMETRY_BYTES:usize=64*1024*1024;
pub const MAX_UNIT_GEOMETRY_POSITIONS:usize=2_000_000;
/// Read only the root profile; skipped coordinate arrays are not retained.
/// Larger sources must explicitly opt in while legacy callers keep their bound.
pub fn unit_geometry_byte_limit(text:&str,default_limit:usize)->Result<usize,String>{
    if text.len()>MAX_UNIT_GEOMETRY_BYTES{return Err("Unit geometry source exceeds 64 MiB.".into());}
    #[derive(Deserialize)]struct Profile {#[serde(default)]geometry_profile:Option<String>}
    let profile:Profile=serde_json::from_str(text).map_err(|e|format!("Invalid unit geometry: {e}"))?;
    Ok(if matches!(profile.geometry_profile.as_deref(),Some("source-exact-large"|"display-topology")){MAX_UNIT_GEOMETRY_BYTES}else{default_limit})
}


#[derive(Clone)]
struct IndexedSegment { index:usize,line:geo_types::Line<f64> }
impl RTreeObject for IndexedSegment {
    type Envelope=AABB<[f64;2]>;
    fn envelope(&self)->Self::Envelope {
        AABB::from_corners([self.line.start.x.min(self.line.end.x),self.line.start.y.min(self.line.end.y)],
            [self.line.start.x.max(self.line.end.x),self.line.start.y.max(self.line.end.y)])
    }
}
/// Broad-phase envelopes only prune disjoint pairs. Remaining pairs use the
/// same native intersection predicate as the previous exhaustive ring check.
fn validate_ring_segments(ring:&LineString<f64>,work:&mut impl GeometryWork)->Result<(),String>{
    let segments:Vec<_>=ring.lines().enumerate().map(|(index,line)|IndexedSegment{index,line}).collect();
    let n=segments.len();
    add_work(work,n.checked_mul(n.max(1).ilog2() as usize+1).ok_or("Geometry work overflow.")?)?;
    if segments.iter().any(|s|s.line.start==s.line.end){return Err("Ring has a zero-length edge.".into());}
    let tree=RTree::bulk_load(segments.clone());
    for current in &segments{
        for other in tree.locate_in_envelope_intersecting(&current.envelope()){
            add_work(work,1)?;
            let i=current.index;let j=other.index;
            if j>=i||i==j+1||(j==0&&i==n-1){continue;}
            if current.line.intersects(&other.line){return Err("Ring self-intersects or touches itself.".into());}
        }
    }Ok(())
}

/// The supplied coordinates are measured directly; this declaration never reprojects them.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CoordinateBasis {
    Wgs84Degrees,
    Nad83Degrees,
    ProjectedMetres,
}
#[derive(Clone,Copy,Debug,Default,Deserialize,Serialize,PartialEq)]
#[serde(rename_all="kebab-case")]
pub enum DissolveMethod { #[default] Sequential, Balanced, SpatialBalanced, Batch }

pub fn dissolve_polygons(polys: &[Polygon<f64>]) -> MultiPolygon<f64> {
    let Some(first) = polys.first() else { return MultiPolygon(vec![]); };
    let mut acc = MultiPolygon(vec![first.clone()]);
    for p in &polys[1..] { acc = acc.union(&MultiPolygon(vec![p.clone()])); }
    acc
}

pub fn dissolve_geometries(geoms: &[Geometry<f64>]) -> MultiPolygon<f64> {
    let polys: Vec<_> = geoms.iter().flat_map(|g| match g {
        Geometry::Polygon(p) => vec![p.clone()],
        Geometry::MultiPolygon(mp) => mp.0.clone(),
        _ => vec![],
    }).collect();
    dissolve_polygons(&polys)
}

/// Preserve native input-order unions and one-based district labels.
pub fn group_dissolve(geoms: &[Geometry<f64>], assignments: &[usize], _k: usize)
    -> HashMap<usize, MultiPolygon<f64>> {
    let mut groups: HashMap<usize, Vec<Geometry<f64>>> = HashMap::new();
    for (i, &district) in assignments.iter().enumerate() {
        if let Some(g) = geoms.get(i) { groups.entry(district).or_default().push(g.clone()); }
    }
    groups.into_iter().map(|(d,g)| (d,dissolve_geometries(&g))).collect()
}

pub fn largest_component(mp: &MultiPolygon<f64>) -> Option<&Polygon<f64>> {
    mp.0.iter().max_by(|a,b| a.unsigned_area().total_cmp(&b.unsigned_area()))
}

/// Exact-ID unit geometry import for browser and native in-memory analysis.
/// Bounds cover parsing, topology checks, sequential unions and quadratic rectangle scoring.
/// The envelope resembles GeoJSON; projected coordinates are explicitly not RFC 7946 GeoJSON.
pub fn read_unit_geometries(text: &str, assignments: &HashMap<String,usize>,
    basis: CoordinateBasis) -> Result<(HashMap<usize,MultiPolygon<f64>>, usize), String> {
    read_unit_geometries_with_method(text,assignments,basis,DissolveMethod::Sequential)
}
pub fn read_unit_geometries_with_method(text:&str,assignments:&HashMap<String,usize>,basis:CoordinateBasis,
    method:DissolveMethod)->Result<(HashMap<usize,MultiPolygon<f64>>,usize),String>{
    if text.len()>MAX_UNIT_GEOMETRY_BYTES {return Err("Unit geometry source exceeds 64 MiB.".into());}
    // Derived struct readers reject duplicate semantic fields, including escaped spellings.
    // Foreign members are allowed, but never used for identity or geometry.
    #[derive(Deserialize)] #[allow(dead_code)] struct Properties { #[serde(rename="GEOID",alias="geoid")] id:String }
    #[derive(Deserialize)] #[allow(dead_code)] struct Shape { #[serde(rename="type")] kind:String,coordinates:Value }
    #[derive(Deserialize)] #[allow(dead_code)] struct Feature { #[serde(rename="type")] kind:String,properties:Properties,geometry:Shape }
    #[derive(Deserialize)] #[allow(dead_code)] struct Collection { #[serde(rename="type")] kind:String,features:Vec<Feature>,#[serde(default)]geometry_profile:Option<String> }
    let _:Collection=serde_json::from_str(text).map_err(|e|e.to_string())?;
    let value:Value=serde_json::from_str(text).map_err(|e|e.to_string())?;
    let large_source=value["geometry_profile"]=="source-exact-large"||value["geometry_profile"]=="display-topology";
    if !large_source && text.len()>18*1024*1024 {return Err("Unit geometry source exceeds 18 MiB.".into());}
    let source_limit=if large_source{MAX_UNIT_GEOMETRY_POSITIONS}else{500_000};
    if value["type"]!="FeatureCollection" {return Err("Supply a unit FeatureCollection.".into());}
    let features=value["features"].as_array().ok_or("Missing geometry features.")?;
    if features.is_empty() || features.len()>100_000 {return Err("Supply 1–100,000 unit features.".into());}
    let mut seen=std::collections::BTreeSet::new(); let mut matched=0; let mut extra=0;
    // Bound total pair work before topology routines, including holes and multipart intersections.
    let mut source_positions=0usize;let mut work=SourceGeometryWork{used:0,limit:if large_source{200_000_000}else{50_000_000}};
    for f in features {
        let mut feature_positions=0usize;
        let g=&f["geometry"];let c=g["coordinates"].as_array().ok_or("Missing polygon coordinates.")?;
        let rings:Vec<&Value>=match g["type"].as_str(){
            Some("Polygon")=>c.iter().collect(),
            Some("MultiPolygon")=>c.iter().map(|p|p.as_array().ok_or("Invalid MultiPolygon component.")).collect::<Result<Vec<_>,_>>()?.into_iter().flatten().collect(),
            _=>return Err("Only Polygon and MultiPolygon unit geometries are supported.".into()),
        };
        for ring in rings {let n=ring.as_array().ok_or("Invalid ring.")?.len();feature_positions=feature_positions.checked_add(n).ok_or("Geometry size overflow.")?;source_positions=source_positions.checked_add(n).ok_or("Geometry size overflow.")?;}
        // Ring self-intersection work is charged by its spatial index below.
        add_work(&mut work,feature_positions)?;
        if source_positions>source_limit{return Err(if large_source{"Geometry exceeds 2,000,000 positions."}else{"Geometry exceeds 500,000 positions."}.into());}
    }
    let mut positions=0usize;
    let mut geoms=Vec::new(); let mut labels=Vec::new();
    for f in features {
        if f["type"]!="Feature" {return Err("Every unit must be a Feature.".into());}
        let id=f["properties"].get("GEOID").or_else(||f["properties"].get("geoid")).and_then(Value::as_str).ok_or("Unit feature requires string properties.GEOID or properties.geoid.")?;
        if id.is_empty() || id.len()>200 || !seen.insert(id.to_string()) {return Err("Empty, oversized or duplicate geometry unit ID.".into());}
        let g=&f["geometry"]; let c=g["coordinates"].as_array().ok_or("Missing polygon coordinates.")?;
        let polys=match g["type"].as_str() {
            Some("Polygon")=>vec![read_polygon(c,basis,&mut positions,&mut work)?],
            Some("MultiPolygon")=>{
                if c.is_empty(){return Err("Empty MultiPolygon.".into());}
                c.iter().map(|p|read_polygon(p.as_array().ok_or("Invalid MultiPolygon component.")?,basis,&mut positions,&mut work)).collect::<Result<Vec<_>,_>>()?
            },
            _=>return Err("Only Polygon and MultiPolygon unit geometries are supported.".into()),
        };
        // Multipart interiors must be disjoint and boundaries may share isolated points.
        // Shared edges, crossings and containment still fail; source coordinates are retained.
        for i in 0..polys.len(){for j in 0..i{
            let n=polys[i].exterior().0.len()+polys[i].interiors().iter().map(|r|r.0.len()).sum::<usize>();
            let m=polys[j].exterior().0.len()+polys[j].interiors().iter().map(|r|r.0.len()).sum::<usize>();
            let pair=n.checked_mul(m).ok_or("Geometry work overflow.")?;
            add_work(&mut work,pair)?;
            if polys[i].intersects(&polys[j]){
                // Charge the additional relation predicate only for intersecting envelopes.
                add_work(&mut work,pair)?;
                // DE-9IM: disjoint interiors and interior/boundary pairs; a zero-dimensional
                // boundary intersection permits isolated point contacts, never a shared edge.
                if !polys[i].relate(&polys[j]).matches("FF*F0****").map_err(|e|e.to_string())?{
                    return Err("Multipart unit components intersect.".into());
                }
            }
        }}
        if let Some(&label)=assignments.get(id){matched+=1;geoms.push(Geometry::MultiPolygon(MultiPolygon(polys)));labels.push(label);}else{extra+=1;}
    }
    if matched!=assignments.len(){return Err(format!("Geometry covers {matched} of {} plan units. Supply every unit explicitly.",assignments.len()));}
    // Preserve input-order unions, charging each actual pair before BooleanOps.
    reject_overlapping_units(&geoms,&mut work).map_err(|error|format!("Cross-unit topology: {error}"))?;
    let districts=match method{DissolveMethod::Sequential=>group_dissolve_bounded(&geoms,&labels,&mut work)
        .map_err(|error|format!("Sequential dissolve: {error}"))?,
        DissolveMethod::Batch=>group_dissolve_batch(&geoms,&labels,&mut work)
            .map_err(|e|format!("Batch dissolve: {e}"))?,
        DissolveMethod::SpatialBalanced=>group_dissolve_spatial(&geoms,&labels,&mut work)
            .map_err(|e|format!("Spatial balanced dissolve: {e}"))?,
        DissolveMethod::Balanced=>group_dissolve_balanced(&geoms,&labels,&mut work)
        .map_err(|error|format!("Balanced dissolve: {error}"))?};
    let output_positions:usize=districts.values().map(multipolygon_positions).sum();
    if output_positions>MAX_UNIT_GEOMETRY_POSITIONS{return Err("Dissolved geometry exceeds 2,000,000 positions.".into());}
    add_work(&mut work,output_positions.checked_mul(output_positions.max(1).ilog2() as usize+1).ok_or("Geometry work overflow.")?)?;
    for mp in districts.values(){if let Some(poly)=largest_component(mp){
        let hull=poly.convex_hull();let n=hull.exterior().0.len();
        add_work(&mut work,n.checked_mul(n).ok_or("Geometry work overflow.")?)?;
    }}
    Ok((districts,extra))
}

#[derive(Clone)]
struct IndexedPolygon { index:usize,envelope:AABB<[f64;2]> }
impl RTreeObject for IndexedPolygon {
    type Envelope=AABB<[f64;2]>;
    fn envelope(&self)->Self::Envelope{self.envelope}
}
struct BoundaryIndex {
    bounds:geo_types::Rect<f64>,segments:Vec<IndexedSegment>,ring_ids:Vec<usize>,ring_count:usize,
    sides:Vec<f64>,tree:RTree<IndexedSegment>,center:Option<geo_types::Point<f64>>,
}
fn boundary_index(p:&Polygon<f64>,work:&mut impl GeometryWork)->Result<BoundaryIndex,String>{
    let mut segments=Vec::new();let mut sides=Vec::new();let mut ring_ids=Vec::new();let mut positions=0;
    for (ring_index,ring) in std::iter::once(p.exterior()).chain(p.interiors()).enumerate(){
        positions+=ring.0.len();
        let side=Polygon::new(ring.clone(),vec![]).signed_area().signum()*if ring_index==0{1.}else{-1.};
        for line in ring.lines(){segments.push(IndexedSegment{index:segments.len(),line});sides.push(side);ring_ids.push(ring_index);}
    }
    add_work(work,positions.checked_mul(positions.max(1).ilog2() as usize+2).ok_or("Geometry work overflow.")?)?;
    let tree=RTree::bulk_load(segments.clone());
    Ok(BoundaryIndex{bounds:p.bounding_rect().ok_or("Empty polygon boundary.")?,segments,ring_ids,ring_count:1+p.interiors().len(),sides,tree,center:geo::Centroid::centroid(p)})
}
fn contains_charged(index:&BoundaryIndex,point:geo_types::Point<f64>,work:&mut impl GeometryWork)->Result<bool,String>{
    if !index.bounds.intersects(&point){return Ok(false);}
    add_work(work,index.ring_count)?;
    let mut winding=vec![0i32;index.ring_count];let coord=point.0;
    // Every winding contribution or boundary hit crosses the point's y slab.
    // Retain geo's robust orientation and half-open endpoint convention.
    let slab=AABB::from_corners([index.bounds.min().x,coord.y],[index.bounds.max().x,coord.y]);
    for segment in index.tree.locate_in_envelope_intersecting(&slab){
        add_work(work,1)?;let edge=segment.line;
        use geo::kernels::{Kernel,Orientation,RobustKernel};
        let direction=RobustKernel::orient2d(edge.start,edge.end,coord);
        if direction==Orientation::Collinear&&coord.x>=edge.start.x.min(edge.end.x)&&coord.x<=edge.start.x.max(edge.end.x){return Ok(false);}
        let ring=index.ring_ids[segment.index];
        if edge.start.y<=coord.y&&edge.end.y>coord.y&&direction==Orientation::CounterClockwise{winding[ring]+=1;}
        if edge.start.y>coord.y&&edge.end.y<=coord.y&&direction==Orientation::Clockwise{winding[ring]-=1;}
    }
    Ok(winding[0]!=0&&winding[1..].iter().all(|n|*n==0))
}

/// Broad phases prune disjoint polygon/segment envelopes; native predicates
/// still reject crossing boundaries, overlapping interiors and duplicate units.
fn reject_overlapping_units(geoms:&[Geometry<f64>],work:&mut impl GeometryWork)->Result<(),String>{
    let polygons:Vec<_>=geoms.iter().enumerate().flat_map(|(unit,g)|match g{Geometry::MultiPolygon(mp)=>mp.0.iter().map(move|p|(unit,p)).collect::<Vec<_>>(),Geometry::Polygon(p)=>vec![(unit,p)],_=>vec![]}).collect();
    let count=polygons.len();add_work(work,count.checked_mul(count.max(1).ilog2() as usize+1).ok_or("Geometry work overflow.")?)?;
    let indices=polygons.iter().map(|(_,p)|boundary_index(p,work)).collect::<Result<Vec<_>,_>>()?;
    let boxes:Vec<_>=indices.iter().enumerate().map(|(index,b)|IndexedPolygon{index,envelope:AABB::from_corners([b.bounds.min().x,b.bounds.min().y],[b.bounds.max().x,b.bounds.max().y])}).collect();
    let tree=RTree::bulk_load(boxes.clone());
    for i in 0..polygons.len(){for other in tree.locate_in_envelope_intersecting(&boxes[i].envelope){
        add_work(work,1)?;let j=other.index;
        if j>=i||polygons[i].0==polygons[j].0{continue;}
        let p=polygons[i].1;let q=polygons[j].1;let a=&indices[i];let b=&indices[j];
        for x in &a.segments{for y in b.tree.locate_in_envelope_intersecting(&x.envelope()){
            add_work(work,1)?;
            match geo::line_intersection::line_intersection(x.line,y.line){
                Some(geo::line_intersection::LineIntersection::SinglePoint{is_proper:true,..})=>return Err("Assigned unit polygon interiors overlap; display simplification may not preserve a tract partition.".into()),
                Some(geo::line_intersection::LineIntersection::Collinear{intersection}) if intersection.start!=intersection.end=>{
                    let dot=(x.line.end.x-x.line.start.x)*(y.line.end.x-y.line.start.x)+(x.line.end.y-x.line.start.y)*(y.line.end.y-y.line.start.y);
                    if a.sides[x.index]*b.sides[y.index]*dot>0.{return Err("Assigned unit polygon interiors overlap along a shared boundary.".into());}
                },
                _=>{},
            }
        }}
        for point in q.exterior().points(){if contains_charged(a,point,work)?{return Err("Assigned unit polygon interiors overlap or duplicate geometry.".into());}}
        for point in p.exterior().points(){if contains_charged(b,point,work)?{return Err("Assigned unit polygon interiors overlap or duplicate geometry.".into());}}
        for point in [a.center,b.center].into_iter().flatten(){
            if contains_charged(a,point,work)?&&contains_charged(b,point,work)?{return Err("Assigned unit polygon interiors overlap or duplicate geometry.".into());}
        }
    }}Ok(())
}

fn multipolygon_positions(mp:&MultiPolygon<f64>)->usize{
    mp.0.iter().map(|p|p.exterior().0.len()+p.interiors().iter().map(|r|r.0.len()).sum::<usize>()).sum()
}
/// Call only after per-unit topology and cross-unit non-overlap validation.
/// One native union sweep avoids repeatedly reprocessing intermediate boundaries.
fn group_dissolve_batch(geoms:&[Geometry<f64>],labels:&[usize],work:&mut impl GeometryWork)->Result<HashMap<usize,MultiPolygon<f64>>,String>{
    let mut groups:std::collections::BTreeMap<usize,Vec<Polygon<f64>>>=std::collections::BTreeMap::new();
    for(g,&label)in geoms.iter().zip(labels){match g{
        Geometry::Polygon(p)=>groups.entry(label).or_default().push(p.clone()),
        Geometry::MultiPolygon(mp)=>groups.entry(label).or_default().extend(mp.0.iter().cloned()),
        _=>return Err("Only polygon geometry can be dissolved.".into()),
    }}
    let mut result=HashMap::new();
    for(label,parts)in groups{
        let input=MultiPolygon(parts);let positions=multipolygon_positions(&input);
        // No cross-interior intersections remain after the shared guard. This
        // charges a single validated-partition sweep model, not a runtime proof.
        add_work(work,positions.checked_mul(positions.max(1).ilog2() as usize+1).and_then(|n|n.checked_mul(3)).ok_or("Geometry work overflow.")?)?;
        let union=input.union(&MultiPolygon(vec![]));
        if multipolygon_positions(&union)>MAX_UNIT_GEOMETRY_POSITIONS{return Err("Intermediate dissolve exceeds 2,000,000 positions.".into());}
        result.insert(label,union);
    }Ok(result)
}
fn charge_indexed_union(a:&MultiPolygon<f64>,b:&MultiPolygon<f64>,work:&mut impl GeometryWork)->Result<(),String>{
    let segments=|mp:&MultiPolygon<f64>|mp.0.iter().flat_map(|p|std::iter::once(p.exterior()).chain(p.interiors())).flat_map(|r|r.lines()).enumerate().map(|(index,line)|IndexedSegment{index,line}).collect::<Vec<_>>();
    let left=segments(a);let right=segments(b);let size=left.len().checked_add(right.len()).ok_or("Geometry work overflow.")?;
    let levels=size.max(1).ilog2() as usize+1;
    add_work(work,size.checked_mul(levels).ok_or("Geometry work overflow.")?)?;
    let tree=RTree::bulk_load(right);let mut candidates=0usize;
    for line in left{for _ in tree.locate_in_envelope_intersecting(&line.envelope()){
        add_work(work,1)?;candidates=candidates.checked_add(1).ok_or("Geometry work overflow.")?;
    }}
    // Charged sweep model, not a wall-clock or allocation certificate. Segment
    // envelopes conservatively count possible cross-input intersection events.
    add_work(work,size.checked_add(candidates).and_then(|n|n.checked_mul(levels)).and_then(|n|n.checked_mul(3)).ok_or("Geometry work overflow.")?)
}
// Sequential union order is the native algorithm. Admission may use either
// existing conservative work model, whichever is cheaper. Capping the estimate
// just above the largest source budget makes rejection portable to wasm32.
const MAX_SEQUENTIAL_CHARGE:usize=200_000_001;
struct SequentialEstimate {used:usize,cap:usize}
impl GeometryWork for SequentialEstimate {
    fn charge(&mut self,n:usize)->Result<(),String>{
        let next=self.used.checked_add(n).ok_or("Sequential estimate cap")?;
        if next>self.cap{return Err("Sequential estimate cap".into());}self.used=next;Ok(())
    }
}
fn charge_sequential_union(a:&MultiPolygon<f64>,b:&MultiPolygon<f64>,work:&mut impl GeometryWork)->Result<(),String>{
    let pair=(multipolygon_positions(a) as u64).checked_mul(multipolygon_positions(b) as u64).ok_or("Geometry work overflow.")?;
    let ceiling=pair.min(MAX_SEQUENTIAL_CHARGE as u64) as usize;
    let mut estimate=SequentialEstimate{used:0,cap:ceiling};
    // An indexed estimate above the cap, including checked arithmetic overflow,
    // cannot improve on the Cartesian model (or fit any supported source budget).
    let charge=if charge_indexed_union(a,b,&mut estimate).is_ok(){estimate.used}else{ceiling};
    add_work(work,charge)
}
fn group_dissolve_balanced(geoms:&[Geometry<f64>],labels:&[usize],work:&mut impl GeometryWork)->Result<HashMap<usize,MultiPolygon<f64>>,String>{
    group_dissolve_pairs(geoms,labels,work,false)
}
fn group_dissolve_spatial(geoms:&[Geometry<f64>],labels:&[usize],work:&mut impl GeometryWork)->Result<HashMap<usize,MultiPolygon<f64>>,String>{
    group_dissolve_pairs(geoms,labels,work,true)
}
fn spatial_order(level:&mut [MultiPolygon<f64>],work:&mut impl GeometryWork)->Result<(),String>{
    add_work(work,level.iter().map(multipolygon_positions).sum())?;
    add_work(work,level.len().checked_mul(level.len().max(1).ilog2() as usize+1).ok_or("Geometry work overflow.")?)?;
    let bounds=level.iter().map(|mp|mp.bounding_rect().ok_or("Empty polygon boundary.")).collect::<Result<Vec<_>,_>>()?;
    let min_x=bounds.iter().map(|r|r.min().x).fold(f64::INFINITY,f64::min);
    let min_y=bounds.iter().map(|r|r.min().y).fold(f64::INFINITY,f64::min);
    let width=bounds.iter().map(|r|r.max().x).fold(f64::NEG_INFINITY,f64::max)-min_x;
    let height=bounds.iter().map(|r|r.max().y).fold(f64::NEG_INFINITY,f64::max)-min_y;
    let quantize=|v:f64,min:f64,span:f64|if span>0.{(((v-min)/span).clamp(0.,1.)*65535.).floor() as u32}else{0};
    // Quantization is used only for ordering keys. Source coordinates are untouched.
    let spread=|n:u32|{let mut x=n&0xffff;x=(x|(x<<8))&0x00ff00ff;x=(x|(x<<4))&0x0f0f0f0f;x=(x|(x<<2))&0x33333333;(x|(x<<1))&0x55555555};
    let mut keyed=level.iter().cloned().zip(bounds).enumerate().map(|(index,(mp,r))|{
        let center=r.center();let key=spread(quantize(center.x,min_x,width))|(spread(quantize(center.y,min_y,height))<<1);
        (key,index,mp)
    }).collect::<Vec<_>>();
    keyed.sort_by_key(|(key,index,_)|(*key,*index));
    for(slot,(_,_,mp))in level.iter_mut().zip(keyed){*slot=mp;}Ok(())
}
fn group_dissolve_pairs(geoms:&[Geometry<f64>],labels:&[usize],work:&mut impl GeometryWork,spatial:bool)->Result<HashMap<usize,MultiPolygon<f64>>,String>{
    let mut groups:std::collections::BTreeMap<usize,Vec<MultiPolygon<f64>>>=std::collections::BTreeMap::new();
    for(g,&label)in geoms.iter().zip(labels){match g{
        Geometry::Polygon(p)=>groups.entry(label).or_default().push(MultiPolygon(vec![p.clone()])),
        Geometry::MultiPolygon(mp)=>groups.entry(label).or_default().extend(mp.0.iter().map(|p|MultiPolygon(vec![p.clone()]))),
        _=>return Err("Only polygon geometry can be dissolved.".into()),
    }}
    let mut result=HashMap::new();
    for(label,mut level)in groups{
        if spatial&&!level.is_empty(){spatial_order(&mut level,work)?;}
        while level.len()>1{
            let mut next=Vec::with_capacity(level.len().div_ceil(2));let mut inputs=level.into_iter();
            while let Some(a)=inputs.next(){if let Some(b)=inputs.next(){
                let union=if spatial{
                    add_work(work,multipolygon_positions(&a).checked_add(multipolygon_positions(&b)).ok_or("Geometry work overflow.")?)?;
                    let left=a.bounding_rect().ok_or("Empty polygon boundary.")?;let right=b.bounding_rect().ok_or("Empty polygon boundary.")?;
                    if left.max().x<right.min().x||right.max().x<left.min().x||left.max().y<right.min().y||right.max().y<left.min().y{
                        let mut parts=a.0;parts.extend(b.0);MultiPolygon(parts)
                    }else{charge_indexed_union(&a,&b,work)?;a.union(&b)}
                }else{charge_indexed_union(&a,&b,work)?;a.union(&b)};
                if multipolygon_positions(&union)>MAX_UNIT_GEOMETRY_POSITIONS{return Err("Intermediate dissolve exceeds 2,000,000 positions.".into());}
                next.push(union);
            }else{next.push(a);}}
            level=next;
        }
        if let Some(mp)=level.pop(){result.insert(label,mp);}
    }Ok(result)
}
fn group_dissolve_bounded(geoms:&[Geometry<f64>],labels:&[usize],work:&mut impl GeometryWork)->Result<HashMap<usize,MultiPolygon<f64>>,String>{
    let mut groups:std::collections::BTreeMap<usize,Vec<Polygon<f64>>>=std::collections::BTreeMap::new();
    for (g,&label) in geoms.iter().zip(labels){match g{
        Geometry::Polygon(p)=>groups.entry(label).or_default().push(p.clone()),
        Geometry::MultiPolygon(mp)=>groups.entry(label).or_default().extend(mp.0.iter().cloned()),
        _=>return Err("Only polygon geometry can be dissolved.".into()),
    }}
    let mut result=HashMap::new();
    for (label,polys) in groups{
        let mut parts=polys.into_iter();let Some(first)=parts.next()else{continue;};
        let mut acc=MultiPolygon(vec![first]);
        for next in parts{
            let input=MultiPolygon(vec![next]);
            charge_sequential_union(&acc,&input,work)?;
            acc=acc.union(&input);
            if multipolygon_positions(&acc)>MAX_UNIT_GEOMETRY_POSITIONS{return Err("Intermediate dissolve exceeds 2,000,000 positions.".into());}
        }
        result.insert(label,acc);
    }
    Ok(result)
}

trait GeometryWork {fn charge(&mut self,n:usize)->Result<(),String>;}
struct SourceGeometryWork {used:usize,limit:usize}
impl GeometryWork for SourceGeometryWork {
    fn charge(&mut self,n:usize)->Result<(),String>{
        self.used=self.used.checked_add(n).ok_or("Geometry work overflow.")?;
        if self.used>self.limit{return Err(format!("Geometry exceeds the {}-million work budget; use a documented analysis-quality simplification.",self.limit/1_000_000));}Ok(())
    }
}
// Existing small-source helpers and oracle tests retain their 50-million cap.
impl GeometryWork for usize {
    fn charge(&mut self,n:usize)->Result<(),String>{
        *self=self.checked_add(n).ok_or("Geometry work overflow.")?;
        if *self>50_000_000{return Err("Geometry exceeds the 50-million work budget; use a documented analysis-quality simplification.".into());}Ok(())
    }
}
fn add_work(work:&mut impl GeometryWork,n:usize)->Result<(),String>{work.charge(n)}

// OGC/JTS allows one isolated shell/hole contact. Multiple contacts can
// disconnect the polygon interior; shared edges and proper crossings fail.
fn at_most_one_ring_contact(shell:&LineString<f64>,hole:&LineString<f64>)->bool{
    use geo::line_intersection::{line_intersection,LineIntersection};
    let mut contact=None;
    for a in shell.lines(){for b in hole.lines(){
        let point=match line_intersection(a,b){
            None=>continue,
            Some(LineIntersection::SinglePoint{is_proper:true,..})=>return false,
            Some(LineIntersection::SinglePoint{intersection,..})=>intersection,
            Some(LineIntersection::Collinear{intersection})=>{
                if intersection.start!=intersection.end{return false;}intersection.start
            }
        };
        if contact.is_some_and(|prior|prior!=point){return false;}contact=Some(point);
    }}true
}
// A cycle of ring contacts can disconnect the polygon interior. Reject it;
// this conservative guard does not claim complete OGC validity coverage.
fn record_ring_contact(parents:&mut [usize],a:usize,b:usize)->Result<(),String>{
    let root=|mut i:usize|{while parents[i]!=i{i=parents[i];}i};
    let left=root(a);let right=root(b);
    if left==right{return Err("Ring contact cycle may disconnect polygon interior.".into());}
    parents[right]=left;Ok(())
}
fn read_polygon(rings:&[Value],basis:CoordinateBasis,positions:&mut usize,work:&mut impl GeometryWork)->Result<Polygon<f64>,String>{
    if rings.is_empty(){return Err("Polygon has no exterior ring.".into());}
    let lines=rings.iter().map(|r|{
        let raw=r.as_array().ok_or("Invalid ring.")?;
        if raw.len()<4{return Err("Rings require at least four closed positions.".into());}
        *positions=positions.checked_add(raw.len()).ok_or("Geometry size overflow.")?;
        if *positions>MAX_UNIT_GEOMETRY_POSITIONS{return Err("Geometry exceeds 2,000,000 positions.".into());}
        let coords=raw.iter().map(|v|{
            let v=v.as_array().ok_or("Invalid position.")?;
            if v.len()!=2{return Err("Supply exactly two coordinates per position.".into());}
            let x=v[0].as_f64().ok_or("Invalid x coordinate.")?;let y=v[1].as_f64().ok_or("Invalid y coordinate.")?;
            if !x.is_finite()||!y.is_finite()||x.abs()>1e9||y.abs()>1e9{return Err("Non-finite or oversized coordinate.".into());}
            if basis!=CoordinateBasis::ProjectedMetres && (!( -180.0..=180.0).contains(&x)||!(-90.0..=90.0).contains(&y)){return Err("Geographic position out of longitude/latitude range.".into());}
            Ok(Coord{x,y})
        }).collect::<Result<Vec<_>,String>>()?;
        if coords.first()!=coords.last(){return Err("Polygon rings must be explicitly closed.".into());}
        let ring=LineString(coords);validate_ring_segments(&ring,work)?;Ok(ring)
    }).collect::<Result<Vec<_>,String>>()?;
    let outer=Polygon::new(lines[0].clone(),vec![]);
    let mut touch_components:Vec<usize>=(0..lines.len()).collect();
    for (i,hole) in lines[1..].iter().enumerate(){
        let pair=lines[0].0.len().checked_mul(hole.0.len()).and_then(|n|n.checked_mul(3)).ok_or("Geometry work overflow.")?;
        add_work(work,pair)?;
        if !outer.contains(&Polygon::new(hole.clone(),vec![])) || !at_most_one_ring_contact(&lines[0],hole){return Err("Hole must be inside exterior with at most one isolated boundary contact.".into());}
        if lines[0].intersects(hole){record_ring_contact(&mut touch_components,0,i+1)?;}
        for (j,other) in lines[1..i+1].iter().enumerate(){
            add_work(work,other.0.len().checked_mul(hole.0.len()).and_then(|n|n.checked_mul(4)).ok_or("Geometry work overflow.")?)?;
            let a=Polygon::new(other.clone(),vec![]);let b=Polygon::new(hole.clone(),vec![]);
            if a.intersects(&b){
                if a.contains(&b)||b.contains(&a)||!at_most_one_ring_contact(other,hole){return Err("Holes intersect or nest.".into());}
                record_ring_contact(&mut touch_components,j+1,i+1)?;
            }
        }
    }
    let p=Polygon::new(lines[0].clone(),lines[1..].to_vec());
    if !p.unsigned_area().is_finite() || p.unsigned_area()<=0.0{return Err("Polygon has zero or invalid area.".into());}Ok(p)
}

/// Native largest-component policy, with omissions and coordinate units explicit.
/// Failed/empty districts remain visible instead of silently disappearing from the report.
pub fn compactness_report(districts: &HashMap<usize, MultiPolygon<f64>>, state: &str,
    year: &str, basis: CoordinateBasis, crs_label: &str) -> Value {
    let mut ids: Vec<_> = districts.keys().copied().collect(); ids.sort_unstable();
    let rows: Vec<_> = ids.into_iter().map(|id| {
        let mp = &districts[&id];
        let total_area = mp.unsigned_area();
        let Some(poly) = largest_component(mp) else {
            return json!({"district":id,"status":"unavailable","error":"empty dissolved geometry"});
        };
        match crate::all_metrics(id,poly) {
            Err(e) => json!({"district":id,"status":"unavailable","error":e.to_string()}),
            Ok(m) => json!({"district":id,"status":"measured","polsby_popper":m.polsby_popper,
                "reock":m.reock,"convex_hull_ratio":m.convex_hull_ratio,"schwartzberg":m.schwartzberg,
                "length_width_ratio":m.length_width_ratio,"component_count":mp.0.len(),
                "measured_area_fraction":if total_area>0.0 {Some(m.area_m2/total_area)}else{None},
                "area_coordinate_units_squared":m.area_m2,"perimeter_coordinate_units":m.perimeter_m,
                "area_m2":if basis==CoordinateBasis::ProjectedMetres {Some(m.area_m2)}else{None},
                "perimeter_m":if basis==CoordinateBasis::ProjectedMetres {Some(m.perimeter_m)}else{None}}),
        }
    }).collect();
    json!({"analyzer":"compactness","state":state,"year":year,"coordinate_basis":basis,
        "crs_label":crs_label,"component_policy":"largest-area-component",
        "length_width_method":"minimum-area-hull-edge-box-minimum-aspect-roundoff-ties-v1","reock_method":"centroid-radius-proxy","perimeter_policy":"exterior-ring-only","perimeter_length_method":"portable-libm-hypot-input-order-v1","districts":rows,
        "crs_note":match basis {
            CoordinateBasis::Wgs84Degrees|CoordinateBasis::Nad83Degrees=>"Planar scores on longitude/latitude degrees are distorted. Area and perimeter are degrees squared/degrees, not metres. Supply projected equal-area metre coordinates for physical measurements.",
            CoordinateBasis::ProjectedMetres=>"Measured directly on supplied metre coordinates. CRS declaration is not verified; no reprojection is performed. Use an appropriate equal-area projection.",
        }})
}

/// Add hull-reduced native minimum-circle scores with a shared candidate-work cap.
pub fn add_exact_reock(report: &mut Value, districts: &HashMap<usize,MultiPolygon<f64>>,
    work_limit: Option<usize>) -> Result<(),String> {
    let mut work=0usize;
    report["exact_reock_method"]=json!("convex-hull-diameter-proof-pruned-vertex-circles-v2");
    for row in report["districts"].as_array_mut().ok_or("Missing compactness district rows.")? {
        let id=row["district"].as_u64().ok_or("Invalid compactness district label.")? as usize;
        if let Some(poly)=districts.get(&id).and_then(largest_component) {
            match crate::compactness::exact_reock_bounded(poly,&mut work,work_limit) {
                Ok(score)=>{row["exact_reock"]=json!(score);row["exact_reock_status"]=json!("measured");},
                Err(crate::compactness::CompactnessError::CircleWorkLimit)=>return Err("Exact Reock exceeds its shared 50-million search work budget.".into()),
                Err(error)=>{row["exact_reock"]=Value::Null;row["exact_reock_status"]=json!("unavailable");row["exact_reock_error"]=json!(error.to_string());}
            }
        }else{row["exact_reock"]=Value::Null;row["exact_reock_status"]=json!("unavailable");}
    }
    report["exact_reock_work_charged"]=json!(work.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::polygon;
    fn square(x:f64,w:f64)->Polygon<f64>{polygon![(x:x,y:0.),(x:x+w,y:0.),(x:x+w,y:w),(x:x,y:w),(x:x,y:0.)]}
    #[test]fn original_alaska_multipart_island_point_contact_retains_area_and_all_components(){
        let text=include_str!("../tests/fixtures/ak-isolated-multipart-contact.geojson");
        let source:Value=serde_json::from_str(text).unwrap();
        let expected_parts=source["features"][0]["geometry"]["coordinates"].as_array().unwrap().len();
        let expected_area=source["independent_expected_area_degrees2"].as_f64().unwrap();
        let assignments=HashMap::from([("02105000400".to_string(),1)]);
        let (districts,extra)=read_unit_geometries_with_method(text,&assignments,CoordinateBasis::Wgs84Degrees,DissolveMethod::Batch).unwrap();
        assert_eq!(extra,0);assert_eq!(districts[&1].0.len(),expected_parts);
        assert!((districts[&1].unsigned_area()-expected_area).abs()<1e-10);
    }
    #[test]fn multipart_point_contacts_preserve_components_without_admitting_overlap_or_shared_edges(){
        use geo::Translate;
        let text=|polygons:Vec<Polygon<f64>>|{
            let coordinates:Vec<Vec<Vec<Vec<f64>>>>=polygons.iter().map(|p|std::iter::once(p.exterior()).chain(p.interiors()).map(|r|r.0.iter().map(|c|vec![c.x,c.y]).collect()).collect()).collect();
            json!({"type":"FeatureCollection","features":[{"type":"Feature","properties":{"GEOID":"u"},"geometry":{"type":"MultiPolygon","coordinates":coordinates}}]}).to_string()
        };
        let assignments=HashMap::from([("u".to_string(),1)]);
        let vertex_touch=text(vec![square(0.,1.),square(0.,1.).translate(1.,1.)]);
        let vertex_edge_touch=text(vec![square(0.,1.),polygon![(x:0.5,y:1.),(x:0.75,y:1.5),(x:0.25,y:1.5),(x:0.5,y:1.)]]);
        for method in [DissolveMethod::Sequential,DissolveMethod::Balanced,DissolveMethod::SpatialBalanced,DissolveMethod::Batch]{
            let (districts,extra)=read_unit_geometries_with_method(&vertex_touch,&assignments,CoordinateBasis::ProjectedMetres,method).unwrap();
            assert_eq!(extra,0);assert_eq!(districts[&1].0.len(),2);assert_eq!(districts[&1].unsigned_area(),2.);
            let (districts,_)=read_unit_geometries_with_method(&vertex_edge_touch,&assignments,CoordinateBasis::ProjectedMetres,method).unwrap();
            assert_eq!(districts[&1].0.len(),2);assert_eq!(districts[&1].unsigned_area(),1.125);
        }
        for other in [square(1.,1.),square(0.,1.),square(0.,1.).translate(0.5,0.5),square(0.,0.2).translate(0.2,0.2)]{
            let error=read_unit_geometries(&text(vec![square(0.,1.),other]),&assignments,CoordinateBasis::ProjectedMetres).unwrap_err();
            assert_eq!(error,"Multipart unit components intersect.");
        }
        let shell=Polygon::new(square(0.,4.).exterior().clone(),vec![square(0.,2.).translate(1.,1.).exterior().clone()]);
        let island=text(vec![shell,square(0.,0.5).translate(1.5,1.5)]);
        assert!(read_unit_geometries(&island,&assignments,CoordinateBasis::ProjectedMetres).is_ok());
    }
    #[test]fn prepared_map_geoid_alias_preserves_geometry_and_rejects_ambiguous_identity(){
        let text=r#"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"GEOID":"44001"},"geometry":{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}}]}"#;
        let assignments=HashMap::from([("44001".to_string(),1)]);
        let read=|s:&str|read_unit_geometries(s,&assignments,CoordinateBasis::Wgs84Degrees);
        let expected=read(text).unwrap();
        assert_eq!(read(&text.replace("GEOID","geoid")).unwrap(),expected);
        for duplicate in [r#""geoid":"44001","GEOID":"44001""#,r#""geoid":"44003","GEOID":"44001""#,r#""geoid":"44001","g\u0065oid":"44001""#]{
            assert!(read(&text.replace(r#""GEOID":"44001""#,duplicate)).is_err());
        }
    }
    #[test]fn indexed_point_containment_matches_native_boundaries_holes_and_winding(){
        use geo::MapCoords;
        let hole=polygon![(x:1.,y:1.),(x:2.,y:1.),(x:2.,y:2.),(x:1.,y:2.),(x:1.,y:1.)];
        let concave=polygon![(x:0.,y:0.),(x:4.,y:0.),(x:4.,y:1.),(x:1.,y:1.),(x:1.,y:4.),(x:0.,y:4.),(x:0.,y:0.)];
        let triangle=polygon![(x:0.,y:0.),(x:3.,y:1.),(x:1.,y:4.),(x:0.,y:0.)];
        for original in [square(0.,4.),Polygon::new(square(0.,4.).exterior().clone(),vec![hole.exterior().clone()]),concave,triangle]{
            for shift in [0.,1_000_000.]{
            let transform=|c:Coord<f64>|if shift==0.{c}else{Coord{x:c.x*0.8-c.y*0.6+shift,y:c.x*0.6+c.y*0.8+shift}};
            for reverse in [false,true]{let mut p=original.map_coords(transform);if reverse{p.exterior_mut(|r|r.0.reverse());p.interiors_mut(|holes|for h in holes{h.0.reverse();});}
                let index=boundary_index(&p,&mut 0usize).unwrap();
                for x in -4..20{for y in -4..20{let point=geo_types::Point(transform(Coord{x:x as f64/4.,y:y as f64/4.}));
                    assert_eq!(contains_charged(&index,point,&mut 0usize).unwrap(),p.contains(&point));
                }}
                for ring in std::iter::once(p.exterior()).chain(p.interiors()){for c in &ring.0{
                    let point=geo_types::Point(*c);assert!(!contains_charged(&index,point,&mut 0usize).unwrap());
                }}
            }
            }
        }
    }
    #[test]fn balanced_dissolve_preserves_analytic_components_and_budget(){
        let geoms=vec![square(0.,1.).into(),square(1.,1.).into(),square(8.,1.).into(),square(2.,1.).into(),square(9.,1.).into()];let labels=vec![1,1,2,1,2];
        let balanced=group_dissolve_balanced(&geoms,&labels,&mut 0usize).unwrap();
        let sequential=group_dissolve(&geoms,&labels,2);
        assert_eq!(compactness_report(&balanced,"RI","2020",CoordinateBasis::ProjectedMetres,"fixture"),compactness_report(&sequential,"RI","2020",CoordinateBasis::ProjectedMetres,"fixture"));
        assert!(group_dissolve_balanced(&geoms,&labels,&mut 49_999_999usize).is_err());
    }
    #[test]fn spatial_dissolve_preserves_union_area_holes_and_membership(){
        use geo::MapCoords;
        let mut geoms=Vec::new();let mut labels=Vec::new();
        // Permuted grid forces the spatial ordering to differ from input ordering.
        for i in 0..60{let cell=i*17%60;let x=(cell%12) as f64;let y=(cell/12) as f64;
            geoms.push(Geometry::Polygon(square(x,1.).map_coords(|c|Coord{x:c.x,y:c.y+y})));labels.push(cell%3);
        }
        let with_hole=Polygon::new(square(30.,4.).exterior().clone(),vec![square(31.,1.).map_coords(|c|Coord{x:c.x,y:c.y+1.}).exterior().clone()]);
        geoms.push(with_hole.into());labels.push(0);
        let expected=group_dissolve_balanced(&geoms,&labels,&mut 0usize).unwrap();
        let actual=group_dissolve_spatial(&geoms,&labels,&mut 0usize).unwrap();
        for label in 0..3{
            assert_eq!(actual[&label].unsigned_area(),expected[&label].unsigned_area());
            assert_eq!(actual[&label].0.len(),expected[&label].0.len());
            for x in -1..72{for y in -1..14{let point=geo_types::Point::new(x as f64/2.,y as f64/2.);
                assert_eq!(actual[&label].contains(&point),expected[&label].contains(&point));
            }}
        }
        assert!(group_dissolve_spatial(&geoms,&labels,&mut 49_999_999usize).is_err());
    }
    #[test]fn spatial_dissolve_disjoint_shortcut_reduces_charged_work(){
        let geoms:Vec<_>=(0..2048).map(|i|Geometry::Polygon(square((i*119%2048) as f64*3.,1.))).collect();let labels=vec![1;geoms.len()];
        let mut spatial_work=0usize;let actual=group_dissolve_spatial(&geoms,&labels,&mut spatial_work).unwrap();
        let mut balanced_work=0usize;let expected=group_dissolve_balanced(&geoms,&labels,&mut balanced_work).unwrap();
        assert_eq!(actual[&1].unsigned_area(),2048.);assert_eq!(actual[&1].0.len(),2048);
        assert_eq!(actual[&1].unsigned_area(),expected[&1].unsigned_area());assert!(spatial_work<balanced_work/4);
    }
    #[test]fn batch_dissolve_matches_pairwise_union_membership_holes_and_components(){
        use geo::MapCoords;
        let outer=square(0.,4.);let hole=square(1.,1.).map_coords(|c|Coord{x:c.x,y:c.y+1.});
        let holed=Polygon::new(outer.exterior().clone(),vec![hole.exterior().clone()]);
        let fixtures=vec![
            vec![square(0.,1.),square(1.,1.),square(2.,1.)],
            vec![square(0.,2.),square(2.,1.),square(2.,1.).map_coords(|c|Coord{x:c.x,y:c.y+1.})],
            vec![square(0.,1.),square(1.,1.).map_coords(|c|Coord{x:c.x,y:c.y+1.})],
            vec![holed,square(1.25,0.5).map_coords(|c|Coord{x:c.x,y:c.y+1.25}),square(4.,4.)],
            (0..60).map(|i|{let cell=i*17%60;square((cell%12) as f64,1.).map_coords(|c|Coord{x:c.x,y:c.y+(cell/12) as f64})}).collect(),
        ];
        for polys in fixtures{for reverse in [false,true]{
            let geoms:Vec<_>=polys.iter().cloned().map(|mut p|{if reverse{p.exterior_mut(|r|r.0.reverse());p.interiors_mut(|holes|for r in holes{r.0.reverse();});}Geometry::Polygon(p)}).collect();
            let labels=vec![1;geoms.len()];let expected=group_dissolve_balanced(&geoms,&labels,&mut 0usize).unwrap();let actual=group_dissolve_batch(&geoms,&labels,&mut 0usize).unwrap();
            assert!((actual[&1].unsigned_area()-expected[&1].unsigned_area()).abs()<1e-12);
            assert_eq!(actual[&1].0.len(),expected[&1].0.len());
            for x in -1..54{for y in -1..28{let point=geo_types::Point::new(x as f64/4.,y as f64/4.);
                assert_eq!(actual[&1].contains(&point),expected[&1].contains(&point));
            }}
            assert!(group_dissolve_batch(&geoms,&labels,&mut 49_999_999usize).is_err());
        }}
    }
    #[test]fn balanced_dissolve_admits_two_thousand_separated_units(){
        let count=2048usize;let geoms:Vec<_>=(0..count).map(|i|square(i as f64*3.,1.).into()).collect();let labels=vec![1;count];let mut work=0usize;
        let result=group_dissolve_balanced(&geoms,&labels,&mut work).unwrap();
        assert_eq!(result[&1].0.len(),count);assert_eq!(result[&1].unsigned_area(),count as f64);assert!(work<50_000_000);
        assert!(25*count*(count-1)/2>50_000_000); // Previous pair-product charge for this separated fixture.
    }
    fn exhaustive_overlap(p:&Polygon<f64>,q:&Polygon<f64>)->bool{
        for(ai,a)in std::iter::once(p.exterior()).chain(p.interiors()).enumerate(){
            for(bi,b)in std::iter::once(q.exterior()).chain(q.interiors()).enumerate(){
                let sa=Polygon::new(a.clone(),vec![]).signed_area().signum()*if ai==0{1.}else{-1.};
                let sb=Polygon::new(b.clone(),vec![]).signed_area().signum()*if bi==0{1.}else{-1.};
                for x in a.lines(){for y in b.lines(){match geo::line_intersection::line_intersection(x,y){
                    Some(geo::line_intersection::LineIntersection::SinglePoint{is_proper:true,..})=>return true,
                    Some(geo::line_intersection::LineIntersection::Collinear{intersection})if intersection.start!=intersection.end=>{
                        if sa*sb*((x.end.x-x.start.x)*(y.end.x-y.start.x)+(x.end.y-x.start.y)*(y.end.y-y.start.y))>0.{return true;}
                    },_=>{},
                }}}
            }
        }
        q.exterior().points().any(|v|p.contains(&v))||p.exterior().points().any(|v|q.contains(&v))
            ||[geo::Centroid::centroid(p),geo::Centroid::centroid(q)].into_iter().flatten().any(|c|p.contains(&c)&&q.contains(&c))
    }
    #[test]fn polygon_index_matches_exhaustive_overlap_oracle(){
        for offset in -20..40{for width in [0.5,1.,2.]{for reverse in [false,true]{
            let p=square(0.,1.);let mut q=square(offset as f64/10.,width);
            if reverse{q.exterior_mut(|ring|ring.0.reverse());}
            let expected=exhaustive_overlap(&p,&q);
            assert_eq!(reject_overlapping_units(&[p.into(),q.into()],&mut 0usize).is_err(),expected);
        }}}
        let hole=Polygon::new(square(0.,4.).exterior().clone(),vec![square(1.,2.).exterior().clone()]);
        for x in [0.,1.,1.5,2.5,3.,4.]{let q=square(x,0.5);let expected=exhaustive_overlap(&hole,&q);
            assert_eq!(reject_overlapping_units(&[hole.clone().into(),q.into()],&mut 0usize).is_err(),expected);
        }
    }
    #[test]fn polygon_index_admits_ten_thousand_separated_units(){
        let geoms:Vec<_>=(0..10_000).map(|i|square(i as f64*3.,1.).into()).collect();let mut work=0usize;
        reject_overlapping_units(&geoms,&mut work).unwrap();assert!(work<1_000_000);
        assert!(reject_overlapping_units(&geoms,&mut 49_999_999usize).is_err());
    }
    #[test]fn indexed_ring_matches_exhaustive_intersection_oracle(){
        let mut seed=17u64;
        for size in 4..24{for _ in 0..20{
            let mut coords=Vec::new();for _ in 0..size{
                seed=seed.wrapping_mul(6364136223846793005).wrapping_add(1);let x=((seed>>32)%31)as f64;
                seed=seed.wrapping_mul(6364136223846793005).wrapping_add(1);let y=((seed>>32)%31)as f64;
                coords.push(Coord{x,y});
            }coords.push(coords[0]);let ring=LineString(coords);let lines:Vec<_>=ring.lines().collect();
            let mut valid=lines.iter().all(|l|l.start!=l.end);
            for i in 0..lines.len(){for j in 0..i{
                if i!=j+1&&!(j==0&&i==lines.len()-1)&&lines[i].intersects(&lines[j]){valid=false;}
            }}
            assert_eq!(validate_ring_segments(&ring,&mut 0usize).is_ok(),valid);
        }}
    }
    #[test]fn indexed_ring_admits_detailed_collinear_boundary_with_bounded_work(){
        let mut coords=Vec::new();let side=3000;
        for i in 0..side{coords.push(Coord{x:i as f64,y:0.});}
        for i in 0..side{coords.push(Coord{x:side as f64,y:i as f64});}
        for i in 0..side{coords.push(Coord{x:(side-i)as f64,y:side as f64});}
        for i in 0..side{coords.push(Coord{x:0.,y:(side-i)as f64});}
        coords.push(coords[0]);let ring=LineString(coords);let mut work=0usize;
        validate_ring_segments(&ring,&mut work).unwrap();assert!(work<250_000);
        assert!(validate_ring_segments(&ring,&mut 49_999_999usize).is_err());
    }
    #[test]fn overlap_guard_distinguishes_shared_edges_from_aligned_overlap(){
        assert!(reject_overlapping_units(&[square(0.,1.).into(),square(1.,1.).into()],&mut 0usize).is_ok());
        for offset in [0.,0.25,0.5,0.75]{
            assert!(reject_overlapping_units(&[square(0.,1.).into(),square(offset,1.).into()],&mut 0usize).is_err());
        }
        let concave=polygon![(x:0.,y:0.),(x:4.,y:0.),(x:4.,y:1.),(x:1.,y:1.),(x:1.,y:4.),(x:0.,y:4.),(x:0.,y:0.)];
        let mut reversed=concave.clone();reversed.exterior_mut(|r|r.0.reverse());
        assert!(reject_overlapping_units(&[concave.into(),reversed.into()],&mut 0usize).is_err());
    }
    #[test]fn sequential_estimate_never_exceeds_previous_cartesian_charge(){
        for n in [1usize,4,16,64,256]{
            let parts=(0..n).map(|i|square(i as f64*3.,1.)).collect::<Vec<_>>();let a=MultiPolygon(parts);let b=MultiPolygon(vec![square(0.5,1.)]);let mut work=0usize;
            charge_sequential_union(&a,&b,&mut work).unwrap();assert!(work<=multipolygon_positions(&a)*multipolygon_positions(&b));assert!(work>0);
        }
    }
    #[test]fn sequential_indexed_charge_preserves_dense_native_coordinate_output(){
        let dense=|x:f64|{let n=2048usize;let mut ring=Vec::new();for side in 0..4{for i in 0..n{let t=i as f64/n as f64;let (a,b)=match side{0=>(x+t,0.),1=>(x+1.,t),2=>(x+1.-t,1.),_=>(x,1.-t)};ring.push(Coord{x:a,y:b});}}ring.push(ring[0]);Polygon::new(LineString(ring),vec![])};
        let geoms=vec![Geometry::Polygon(dense(0.)),Geometry::Polygon(dense(1.)),Geometry::Polygon(dense(2.))];let labels=vec![1,1,1];
        assert!(8193usize*8193>50_000_000);let mut work=0usize;let result=group_dissolve_bounded(&geoms,&labels,&mut work).unwrap();assert_eq!(result,group_dissolve(&geoms,&labels,1));assert!(work<50_000_000);
    }
    #[test]fn sequential_previous_small_source_admission_is_preserved(){
        let geoms=(0..600).map(|i|Geometry::Polygon(square(i as f64*3.,1.))).collect::<Vec<_>>();let labels=vec![1;geoms.len()];let mut work=0usize;
        assert_eq!(group_dissolve_bounded(&geoms,&labels,&mut work).unwrap(),group_dissolve(&geoms,&labels,1));assert!(work<=25*600*599/2);
        let a=MultiPolygon(vec![square(0.,1.)]);let b=MultiPolygon(vec![square(1.,1.)]);assert!(charge_sequential_union(&a,&b,&mut 49_999_999usize).is_err());
    }
    #[test]fn bounded_dissolve_preserves_native_input_order_and_rejects_budget(){
        let geoms=vec![square(0.,1.).into(),square(1.,1.).into(),square(8.,1.).into(),square(2.,1.).into()];let labels=vec![1,1,2,1];
        let mut work=0usize;let result=group_dissolve_bounded(&geoms,&labels,&mut work).unwrap();
        assert_eq!(result,group_dissolve(&geoms,&labels,2));assert!(work>0);
        assert!(group_dissolve_bounded(&geoms,&labels,&mut 49_999_999usize).is_err());
    }
    #[test]fn shell_hole_allows_single_contact_without_coordinate_repair(){
        let outer=vec![vec![0.,0.],vec![4.,0.],vec![4.,4.],vec![0.,4.],vec![0.,0.]];
        let hole=vec![vec![0.,2.],vec![1.,1.],vec![1.,3.],vec![0.,2.]];
        let rings=json!([outer,hole]);let mut work=0usize;
        let p=read_polygon(rings.as_array().unwrap(),CoordinateBasis::ProjectedMetres,&mut 0,&mut work).unwrap();
        assert_eq!(p.interiors()[0].0[0],Coord{x:0.,y:2.});assert_eq!(p.unsigned_area(),15.);
        for invalid in [
            json!([[0.,1.],[1.,1.],[1.,3.],[0.,3.],[0.,1.]]), // shared edge
            json!([[0.,2.],[2.,1.],[4.,2.],[2.,3.],[0.,2.]]), // two contacts
            json!([[-1.,2.],[1.,1.],[1.,3.],[-1.,2.]]), // crossing/outside
        ]{let bad=json!([rings[0],invalid]);assert!(read_polygon(bad.as_array().unwrap(),CoordinateBasis::ProjectedMetres,&mut 0,&mut 0usize).is_err());}
        assert!(read_polygon(rings.as_array().unwrap(),CoordinateBasis::ProjectedMetres,&mut 0,&mut 49_999_999usize).is_err());
    }
    #[test]fn holes_allow_isolated_contact_and_reject_contact_cycle(){
        let shell=json!([[0.,0.],[4.,0.],[4.,4.],[0.,4.],[0.,0.]]);
        let rings=json!([shell,[[1.,1.],[2.,1.],[1.,2.],[1.,1.]],[[2.,1.],[3.,1.],[3.,2.],[2.,1.]]]);
        let p=read_polygon(rings.as_array().unwrap(),CoordinateBasis::ProjectedMetres,&mut 0,&mut 0usize).unwrap();assert_eq!(p.interiors().len(),2);assert_eq!(p.unsigned_area(),15.);
        let cycle=json!([shell,[[0.,2.],[2.,1.],[1.,3.],[0.,2.]],[[4.,2.],[3.,3.],[2.,1.],[4.,2.]]]);
        let error=read_polygon(cycle.as_array().unwrap(),CoordinateBasis::ProjectedMetres,&mut 0,&mut 0usize).unwrap_err();assert!(error.contains("contact cycle"),"{error}");
    }
    #[test]fn larger_source_work_budget_is_bounded_and_independent(){
        let mut default=SourceGeometryWork{used:49_999_999,limit:50_000_000};assert!(add_work(&mut default,2).is_err());
        let mut large=SourceGeometryWork{used:49_999_999,limit:200_000_000};add_work(&mut large,2).unwrap();assert_eq!(large.used,50_000_001);
        large.used=199_999_999;assert!(add_work(&mut large,2).is_err());
        let mut legacy=49_999_999usize;assert!(add_work(&mut legacy,2).is_err());
    }
    #[test]fn geometry_byte_profile_requires_explicit_root_marker(){
        assert_eq!(unit_geometry_byte_limit(r#"{"geometry_profile":"source-exact-large"}"#,16).unwrap(),MAX_UNIT_GEOMETRY_BYTES);
        assert_eq!(unit_geometry_byte_limit(r#"{"geometry_profile":"display-topology"}"#,16).unwrap(),MAX_UNIT_GEOMETRY_BYTES);
        assert_eq!(unit_geometry_byte_limit(r#"{"metadata":{"geometry_profile":"display-topology"}}"#,16).unwrap(),16);
        assert!(unit_geometry_byte_limit(r#"{"geometry_profile":"display-topology","geometry_profile":"display-topology"}"#,16).is_err());
        assert_eq!(unit_geometry_byte_limit(r#"{"metadata":{"geometry_profile":"source-exact-large"}}"#,16).unwrap(),16);
        assert!(unit_geometry_byte_limit(r#"{"geometry_profile":"source-exact-large","geometry_profile":"source-exact-large"}"#,16).is_err());
    }
    #[test] fn adjacent_units_are_dissolved_before_measurement(){
        let d=group_dissolve(&[square(0.,1.).into(),square(1.,1.).into()],&[1,1],1);
        let r=compactness_report(&d,"RI","2020",CoordinateBasis::ProjectedMetres,"fixture-metres");
        assert_eq!(r["districts"][0]["area_m2"],2.0);
        assert_eq!(r["districts"][0]["perimeter_m"],6.0);
    }
    #[test] fn degree_measurements_never_claim_metres(){
        let d=group_dissolve(&[square(0.,1.).into()],&[1],1);
        let r=compactness_report(&d,"RI","2020",CoordinateBasis::Wgs84Degrees,"EPSG:4326");
        assert!(r["districts"][0]["area_m2"].is_null());
        assert!(r["districts"][0]["perimeter_m"].is_null());
        assert_eq!(r["districts"][0]["area_coordinate_units_squared"],1.0);
    }
    #[test] fn islands_are_disclosed(){
        let d=group_dissolve(&[square(0.,2.).into(),square(10.,1.).into()],&[1,1],1);
        let r=compactness_report(&d,"RI","2020",CoordinateBasis::ProjectedMetres,"fixture");
        assert_eq!(r["districts"][0]["component_count"],2);
        assert_eq!(r["districts"][0]["measured_area_fraction"],0.8);
    }
    #[test] fn empty_geometry_is_reported(){
        let r=compactness_report(&HashMap::from([(1,MultiPolygon(vec![]))]),"RI","2020",CoordinateBasis::Wgs84Degrees,"EPSG:4326");
        assert_eq!(r["districts"][0]["status"],"unavailable");
    }
}
