//! Explicit unit-centroid/population sources for the shared native compactness kernel.
use crate::district_geometry::CoordinateBasis;
use geo::Centroid;
use geo_types::MultiPolygon;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::collections::{HashMap,HashSet};

/// Centroids must already use the geometry's coordinate basis and recorded CRS.
/// All assigned units contribute; the reference is the full dissolved area centroid.
pub fn add_population_compactness(report:&mut Value,districts:&HashMap<usize,MultiPolygon<f64>>,
    assignments:&HashMap<String,usize>,source:&str,basis:CoordinateBasis)->Result<(),String>{
    if source.len()>3*1024*1024{return Err("Population centroid CSV exceeds 3 MiB.".into());}
    let mut reader=csv::Reader::from_reader(source.as_bytes());
    let headers=reader.headers().map_err(|e|e.to_string())?.clone();
    let mut header_names=HashSet::new();
    for header in &headers{if !header_names.insert(header){return Err("Duplicate population centroid CSV header.".into());}}
    let cols=["GEOID","x","y","population"].map(|name|headers.iter().position(|v|v==name).ok_or_else(||format!("Population centroid CSV requires {name}."))).into_iter().collect::<Result<Vec<_>,_>>()?;
    let mut seen=HashSet::new();let mut matched=0usize;let mut excluded=0usize;
    let mut groups:HashMap<usize,Vec<((f64,f64),u64)>>=HashMap::new();
    for (index,row) in reader.records().enumerate(){
        if index>=100_000{return Err("Population centroid CSV exceeds 100,000 rows.".into());}
        let row=row.map_err(|e|e.to_string())?;
        let id=row.get(cols[0]).ok_or("Missing population unit ID.")?;
        if id.is_empty()||id.len()>200||id.chars().any(char::is_control)||!seen.insert(id.to_string()){return Err("Empty, oversized or duplicate population unit ID.".into());}
        let x:f64=row[cols[1]].parse().map_err(|_|"Invalid centroid x.")?;
        let y:f64=row[cols[2]].parse().map_err(|_|"Invalid centroid y.")?;
        if !x.is_finite()||!y.is_finite()||x.abs()>1e9||y.abs()>1e9{return Err("Centroids must be finite coordinates within ±1e9.".into());}
        if basis!=CoordinateBasis::ProjectedMetres&&(x.abs()>180.||y.abs()>90.){return Err("Geographic population centroids exceed longitude/latitude bounds.".into());}
        let digits=&row[cols[3]];
        if digits.is_empty()||!digits.bytes().all(|b|b.is_ascii_digit()){return Err("Population requires unsigned decimal integer counts.".into());}
        let population:u64=digits.parse().map_err(|_|"Population exceeds u64.")?;
        if let Some(&district)=assignments.get(id){matched+=1;groups.entry(district).or_default().push(((x,y),population));}else{excluded+=1;}
    }
    if matched!=assignments.len(){return Err(format!("Population centroid CSV covers {matched} of {} plan units.",assignments.len()));}
    report["population_compactness_source"]=json!({"source_sha256":format!("{:x}",Sha256::digest(source.as_bytes())),"matched_units":matched,"excluded_units":excluded,
        "method":"population-weighted-mean-squared-distance-to-full-dissolved-area-centroid-v1",
        "coordinate_units_squared":if basis==CoordinateBasis::ProjectedMetres{"m2"}else{"degrees2"},
        "scope":"All assigned unit rows contribute, including islands. Reference is the full dissolved geometry area centroid, not the population centroid or largest component centroid. Supplied centroid locations and population counts are declarations; no projection, containment or authenticity is verified. Counts are summed as checked u64, then native f64 weighting rounds large counts. Zero total population retains native score zero and is labelled separately. Lower scores mean less population spread in these coordinates; the score is not dimensionless."});
    for row in report["districts"].as_array_mut().ok_or("Missing compactness rows.")?{
        let district=row["district"].as_u64().ok_or("Invalid district label.")? as usize;
        let data=groups.get(&district).ok_or("Missing district population rows.")?;
        let total=data.iter().try_fold(0u64,|sum,(_,pop)|sum.checked_add(*pop).ok_or("District population overflow."))?;
        let Some(center)=districts.get(&district).and_then(|g|g.centroid())else{
            row["population_compactness"]=json!({"status":"unavailable","reason":"Full dissolved geometry has no area centroid.","total_population":total.to_string(),"unit_count":data.len()});continue;
        };
        let points:Vec<_>=data.iter().map(|(point,_)|*point).collect();
        let populations:Vec<_>=data.iter().map(|(_,pop)|*pop).collect();
        let score=crate::compactness::try_population_weighted_compactness(&points,&populations,(center.x(),center.y())).map_err(|e|e.to_string())?;
        row["population_compactness"]=json!({"status":if total==0{"zero-population-native-zero"}else{"measured"},"mean_squared_distance":score,"district_area_centroid":[center.x(),center.y()],"total_population":total.to_string(),"unit_count":data.len()});
    }
    Ok(())
}

#[cfg(test)]mod tests{
    use super::*;
    use geo_types::polygon;
    fn fixture()->(HashMap<usize,MultiPolygon<f64>>,HashMap<String,usize>,Value){
        let d=HashMap::from([(1,MultiPolygon(vec![polygon![(x:0.,y:0.),(x:4.,y:0.),(x:4.,y:2.),(x:0.,y:2.),(x:0.,y:0.)]]))]);
        let a=HashMap::from([("u1".into(),1),("u2".into(),1)]);
        (d,a,json!({"districts":[{"district":1}]}))
    }
    #[test]fn analytic_weighted_distance_and_exact_counts(){let(d,a,mut r)=fixture();add_population_compactness(&mut r,&d,&a,"GEOID,x,y,population\nu1,1,1,3\nu2,4,1,1\n",CoordinateBasis::ProjectedMetres).unwrap();assert_eq!(r["districts"][0]["population_compactness"]["mean_squared_distance"],1.75);assert_eq!(r["districts"][0]["population_compactness"]["total_population"],"4");}
    #[test]fn source_coverage_and_duplicate_headers_rejected(){let(d,a,mut r)=fixture();for s in ["GEOID,x,y,population\nu1,1,1,3\n","GEOID,x,y,population\nu1,1,1,3\nu1,2,1,4\n","GEOID,x,x,population\nu1,1,1,3\n"]{assert!(add_population_compactness(&mut r,&d,&a,s,CoordinateBasis::ProjectedMetres).is_err());}}
    #[test]fn overflow_rejected_and_zero_population_labelled(){let(d,a,mut r)=fixture();assert!(add_population_compactness(&mut r,&d,&a,"GEOID,x,y,population\nu1,1,1,18446744073709551615\nu2,4,1,1\n",CoordinateBasis::ProjectedMetres).is_err());add_population_compactness(&mut r,&d,&a,"GEOID,x,y,population\nu1,1,1,0\nu2,4,1,0\n",CoordinateBasis::Wgs84Degrees).unwrap();assert_eq!(r["districts"][0]["population_compactness"]["status"],"zero-population-native-zero");assert_eq!(r["population_compactness_source"]["coordinate_units_squared"],"degrees2");}
}
