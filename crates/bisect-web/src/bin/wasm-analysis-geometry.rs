//! Prepare unsimplified TIGER boundary candidates independently of display assets.
use anyhow::{ensure, Context, Result};
use bisect_web::catalog::digest;
use clap::Parser;
use geo::MapCoords;
use geo_types::{Coord, Geometry, Polygon};
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::{Path, PathBuf}};

const NAD83_WKT:&str="GEOGCS[\"GCS_North_American_1983\",DATUM[\"D_North_American_1983\",SPHEROID[\"GRS_1980\",6378137,298.257222101]],PRIMEM[\"Greenwich\",0],UNIT[\"Degree\",0.017453292519943295]]";
#[derive(Parser)]
struct Args {
    #[arg(long)] input_shapefile:PathBuf,
    #[arg(long)] output:PathBuf,
}
fn polygon_coordinates(p:&Polygon<f64>,positions:&mut usize)->Value{
    json!(std::iter::once(p.exterior()).chain(p.interiors()).map(|ring|{
        *positions+=ring.0.len();ring.0.iter().map(|c|[c.x,c.y]).collect::<Vec<_>>()
    }).collect::<Vec<_>>())
}
fn validate_crs(wkt:&str)->Result<()>{
    ensure!(wkt.trim()==NAD83_WKT,"Only the supported TIGER NAD83 geographic WKT is accepted; no CRS is guessed.");Ok(())
}
fn prepare(shape:&Path)->Result<(Value,Value)>{
    ensure!(shape.extension().and_then(|s|s.to_str())==Some("shp"),"Supply a .shp input.");
    let wkt=fs::read_to_string(shape.with_extension("prj")).context("Missing source CRS .prj.")?;validate_crs(&wkt)?;
    let mut sources=serde_json::Map::new();
    for extension in ["shp","shx","dbf","prj"]{
        let path=shape.with_extension(extension);let bytes=fs::read(&path).with_context(||format!("Missing source sidecar {extension}."))?;
        sources.insert(extension.into(),json!({"name":path.file_name().and_then(|s|s.to_str()).context("Invalid source name.")?,"bytes":bytes.len(),"sha256":digest(&bytes)}));
    }
    let records=bisect_data::read_tiger_tracts(shape)?;
    ensure!(!records.is_empty(),"No tract geometry records.");
    let mut ids=BTreeSet::new();let mut state=None;let mut positions=0;
    let mut features=Vec::with_capacity(records.len());
    for record in records{
        ensure!(record.geoid.len()==11&&record.geoid.bytes().all(|b|b.is_ascii_digit()),"Tract GEOID must contain eleven ASCII digits.");
        let fips=&record.geoid[..2];
        ensure!(!["02","15","60","66","69","72","78"].contains(&fips),"EPSG:5070 preparation is restricted to contiguous-US tract inputs.");
        if let Some(expected)=&state{ensure!(expected==fips,"Mixed state tract input.");}else{state=Some(fips.to_owned());}
        ensure!(ids.insert(record.geoid.clone()),"Duplicate tract GEOID.");
        let polygons=match bisect_map::wkb_to_geometry(&record.geometry_wkb)?{
            Geometry::Polygon(p)=>vec![p],Geometry::MultiPolygon(mp)=>mp.0,_=>anyhow::bail!("Unsupported source geometry."),
        };
        let mut coordinates=Vec::with_capacity(polygons.len());
        for polygon in polygons{
            ensure!(std::iter::once(polygon.exterior()).chain(polygon.interiors()).flat_map(|r|&r.0).all(|c|c.x.is_finite()&&c.y.is_finite()&&(-130.0..=-60.0).contains(&c.x)&&(20.0..=55.0).contains(&c.y)),"Source coordinates lie outside the supported contiguous-US preparation extent.");
            let projected=polygon.map_coords(|c|{let(x,y)=bisect_data::projection::nad83_to_epsg5070(c.x,c.y);Coord{x,y}});
            coordinates.push(polygon_coordinates(&projected,&mut positions));
        }
        features.push(json!({"type":"Feature","properties":{"GEOID":record.geoid},"geometry":{"type":"MultiPolygon","coordinates":coordinates}}));
    }
    let geometry=json!({"type":"FeatureCollection","coordinate_basis":"projected-metres","crs_label":"EPSG:5070","geometry_policy":"unsimplified-source-vertices","features":features});
    let bytes=serde_json::to_vec(&geometry)?;
    let provenance=json!({"schema_version":1,"state_fips":state,"units":ids.len(),"positions":positions,"geometry_sha256":digest(&bytes),"geometry_bytes":bytes.len(),"source_crs":"EPSG:4269 (supported WKT matched)","target_crs":"EPSG:5070","projection_method":"bisect-data native NAD83 Albers forward","simplification":false,"sources":sources,"geometry_browser_byte_cap":bisect_analysis::district_geometry::MAX_UNIT_GEOMETRY_BYTES,"within_geometry_only_browser_byte_cap":bytes.len()<=bisect_analysis::district_geometry::MAX_UNIT_GEOMETRY_BYTES,"within_browser_position_cap":positions<=bisect_analysis::district_geometry::MAX_UNIT_GEOMETRY_POSITIONS,"status":"unsimplified-boundary-candidate","scope":"Source hashes establish file identity, not source authenticity. Projection preserves ring vertices and uses the native forward equations. No topology repair, source certification, dissolve or compactness validation is performed. EPSG:5070 is restricted here to contiguous-US inputs. Browser byte capacity also includes the plan and optional population CSV; charged topology/union work can reject otherwise small sources."});
    Ok((geometry,provenance))
}
fn main()->Result<()>{
    let args=Args::parse();ensure!(!args.output.exists(),"Output directory must not exist.");
    let (geometry,provenance)=prepare(&args.input_shapefile)?;
    fs::create_dir_all(&args.output)?;
    fs::write(args.output.join("unit-geometry.json"),serde_json::to_vec(&geometry)?)?;
    fs::write(args.output.join("provenance.json"),serde_json::to_vec_pretty(&provenance)?)?;
    println!("{}",serde_json::to_string_pretty(&provenance)?);Ok(())
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn source_crs_is_explicit(){
        assert!(validate_crs(NAD83_WKT).is_ok());assert!(validate_crs(&format!("{NAD83_WKT}\n")).is_ok());
        assert!(validate_crs("EPSG:4269").is_err());assert!(validate_crs(&NAD83_WKT.replace("North_American_1983","WGS_1984")).is_err());
    }
    #[test]fn real_tiger_vertices_and_native_projection_are_preserved(){
        let source=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/tiger/tl_2020_44_tract/tl_2020_44_tract.shp");
        let records=bisect_data::read_tiger_tracts(&source).unwrap();let (geometry,provenance)=prepare(&source).unwrap();
        assert_eq!(geometry["features"].as_array().unwrap().len(),records.len());
        let mut positions=0;
        for (record,feature) in records.iter().zip(geometry["features"].as_array().unwrap()){
            assert_eq!(feature["properties"]["GEOID"],record.geoid);
            let polys=match bisect_map::wkb_to_geometry(&record.geometry_wkb).unwrap(){Geometry::Polygon(p)=>vec![p],Geometry::MultiPolygon(mp)=>mp.0,_=>panic!("polygon expected")};
            assert_eq!(polys.len(),feature["geometry"]["coordinates"].as_array().unwrap().len());
            for (p,coordinates) in polys.iter().zip(feature["geometry"]["coordinates"].as_array().unwrap()){
                assert_eq!(1+p.interiors().len(),coordinates.as_array().unwrap().len());
                for (ring,points) in std::iter::once(p.exterior()).chain(p.interiors()).zip(coordinates.as_array().unwrap()){
                    assert_eq!(ring.0.len(),points.as_array().unwrap().len());positions+=ring.0.len();
                    for (c,point) in ring.0.iter().zip(points.as_array().unwrap()){
                        let(x,y)=bisect_data::projection::nad83_to_epsg5070(c.x,c.y);assert_eq!(point,&json!([x,y]));
                    }
                }
            }
        }
        assert_eq!(provenance["positions"],positions);assert_eq!(provenance["geometry_sha256"],digest(&serde_json::to_vec(&geometry).unwrap()));
        assert_eq!(provenance["simplification"],false);
    }
}
