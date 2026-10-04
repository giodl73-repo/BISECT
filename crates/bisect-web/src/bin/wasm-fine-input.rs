//! Prepare explicit multiscale browser inputs from local Census source files.
use anyhow::{ensure,Context,Result};
use clap::Parser;
use geo::{Centroid,MapCoords,Simplify};
use geo_types::{Geometry,MultiPolygon};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::io::Read;
use std::{collections::{BTreeMap,BTreeSet},fs,path::PathBuf};

#[derive(Parser)]
struct Args {
    #[arg(long)]state:String,
    #[arg(long)]year:u16,
    #[arg(long)]tract_graph:PathBuf,
    #[arg(long)]shape:PathBuf,
    #[arg(long)]geography:PathBuf,
    #[arg(long)]population:PathBuf,
    #[arg(long)]output:PathBuf,
    /// Explicitly use the native median-edge cost for island bridges.
    #[arg(long)]native_island_bridges:bool,
}
fn main()->Result<()> {
    let a=Args::parse();
    ensure!(!a.output.exists(),"Output already exists; choose a new file.");
    let report_path=a.output.with_extension("sources.json");
    ensure!(!report_path.exists(),"Source report already exists; choose a new output file.");
    let manifest=bisect_cli::fetch::load_manifest().map_err(anyhow::Error::msg)?;
    let fips=manifest.states.get(&a.state).context("Unknown state.")?.fips.clone();
    let tract:Value=serde_json::from_slice(&fs::read(&a.tract_graph)?)?;
    ensure!(tract["schema_version"]==1 && tract["state"]==a.state && tract["year"]==a.year.to_string(),"Tract graph schema or scope mismatch.");
    let projection=fs::read_to_string(a.shape.with_extension("prj"))?;
    ensure!(projection.len()<=65536 && projection.starts_with("GEOGCS[") && !projection.contains("PROJCS[") && projection.contains("North_American_1983") && projection.contains("UNIT[\"Degree\""),"Fine shapefile must declare geographic NAD83 coordinates; projected or unknown CRS is rejected.");
    let records=bisect_data::tiger::read_tiger_block_groups(&a.shape)?;
    ensure!(records.len()<=100000 && records.iter().all(|r|r.geoid.starts_with(&fips)),"Invalid fine state scope or unit limit.");
    let mut population=BTreeMap::<String,i64>::new();
    for b in bisect_data::read_pl94_block_populations_for_year(&a.geography,&a.population,a.year)? {
        ensure!(b.geoid.starts_with(&fips),"Foreign-state PL block.");
        let sum=population.entry(b.geoid[..12].into()).or_default();
        *sum=sum.checked_add(b.population).context("Fine population overflow.")?;
    }
    ensure!(population.len()==records.len() && records.iter().all(|r|population.contains_key(&r.geoid)),"TIGER and PL block-group universes differ; no populations are filled.");
    let ids:Vec<_>=records.iter().map(|r|r.geoid.clone()).collect();
    let pop:Vec<_>=ids.iter().map(|id|population[id]).collect();
    let mut parents=BTreeMap::<String,i64>::new();
    for(id,p)in ids.iter().zip(&pop){*parents.entry(id[..11].into()).or_default()+=p;}
    let tract_ids=tract["geoids"].as_array().context("Missing tract IDs.")?;
    let tract_pop=tract["population"].as_array().context("Missing tract population.")?;
    ensure!(tract_ids.len()==tract_pop.len() && parents.len()==tract_ids.len() && tract_ids.iter().enumerate().all(|(i,id)|id.as_str().is_some_and(|id|parents.get(id).copied()==tract_pop[i].as_i64())),"Fine/tract parent population or coverage mismatch.");
    let mut projected=Vec::new();let mut features=Vec::new();let mut points=0usize;
    for r in &records {
        let geometry=bisect_map::wkb_to_geometry(&r.geometry_wkb)?;
        let polygons=match geometry{Geometry::Polygon(p)=>vec![p],Geometry::MultiPolygon(mp)=>mp.0,_=>anyhow::bail!("Nonpolygon fine geometry.")};
        let coordinates:Vec<_>=polygons.iter().map(|p|{let p=p.simplify(&0.0002);std::iter::once(p.exterior()).chain(p.interiors()).map(|ring|ring.0.iter().map(|c|[c.x,c.y]).collect::<Vec<_>>()).collect::<Vec<_>>()}).collect();
        for polygon in &coordinates {for ring in polygon {
            ensure!(ring.len()>=4 && ring.first()==ring.last() && ring.iter().all(|p|p[0].is_finite()&&p[1].is_finite()&&p[0].abs()<=180.&&p[1].abs()<=90.),"Invalid simplified geographic ring in {}.",r.geoid);
            points=points.checked_add(ring.len()).context("Point count overflow.")?;
            ensure!(points<=2000000,"Fine map exceeds the browser two-million-point limit.");
        }}
        features.push(json!({"type":"Feature","properties":{"geoid":r.geoid},"geometry":{"type":"MultiPolygon","coordinates":coordinates}}));
        projected.push(MultiPolygon(polygons).map_coords(|c|{let(x,y)=bisect_data::projection::nad83_to_epsg5070(c.x,c.y);geo_types::Coord{x,y}}));
    }
    let wkbs:Vec<_>=projected.iter().map(bisect_data::tiger::geo_to_wkb_multipolygon).collect();
    let mut graph=bisect_data::build_adjacency_graph(&wkbs,10.)?;
    let centers:Vec<_>=projected.iter().map(|p|p.centroid().map(|p|(p.x(),p.y())).context("Empty fine geometry.")).collect::<Result<_>>()?;
    let mut lengths:Vec<_>=graph.edge_weights.values().copied().collect();lengths.sort_by(f64::total_cmp);
    let bridge_weight=lengths.get(lengths.len()/2).copied().context("No measured boundaries for bridge cost.")?;
    let mut bridges=Vec::new();
    // A connected whole state can still contain disconnected parent regions.
    // Prepare each parent first, then connect the state; never add parent edges
    // merely to force agreement with the selected tract graph.
    for parent in parents.keys(){
        let group:Vec<_>=ids.iter().enumerate().filter_map(|(i,id)|id.starts_with(parent).then_some(i)).collect();
        let index:BTreeMap<_,_>=group.iter().enumerate().map(|(local,&global)|(global,local)).collect();
        let adjacency:Vec<Vec<usize>>=group.iter().map(|&i|graph.adjacency[i].iter().filter_map(|j|index.get(j).copied()).collect()).collect();
        let local_centers:Vec<_>=group.iter().map(|&i|centers[i]).collect();
        let local_ids:Vec<_>=group.iter().map(|&i|ids[i].clone()).collect();
        for(u,v)in bisect_data::connect_island_components(&adjacency,&local_centers,&local_ids){bridges.push((group[u],group[v]));}
    }
    ensure!(bridges.is_empty() || a.native_island_bridges,"Fine parent regions require {} bridges; use --native-island-bridges to explicitly select derived median-edge costs.",bridges.len());
    for &(u,v) in &bridges {graph.adjacency[u].push(v);graph.adjacency[v].push(u);graph.edge_weights.insert((u.min(v),u.max(v)),bridge_weight);}
    let state_bridges=bisect_data::connect_island_components(&graph.adjacency,&centers,&ids);
    ensure!(state_bridges.is_empty() || a.native_island_bridges,"Fine state graph requires connectivity bridges; use --native-island-bridges to explicitly select derived median-edge costs.");
    for &(u,v) in &state_bridges {graph.adjacency[u].push(v);graph.adjacency[v].push(u);graph.edge_weights.insert((u.min(v),u.max(v)),bridge_weight);}
    bridges.extend(state_bridges);
    for ns in &mut graph.adjacency{ns.sort_unstable();}
    for parent in parents.keys(){
        let group:Vec<_>=ids.iter().enumerate().filter_map(|(i,id)|(id.starts_with(parent)).then_some(i)).collect();
        let mut seen=BTreeSet::from([group[0]]);let mut todo=vec![group[0]];
        while let Some(i)=todo.pop(){for &j in &graph.adjacency[i]{if ids[j].starts_with(parent) && seen.insert(j){todo.push(j);}}}
        ensure!(seen.len()==group.len(),"Block groups within tract {parent} are disconnected; input was not altered.");
    }
    let mut expected=BTreeSet::new();
    for(i,ns)in tract["adjacency"].as_array().context("Missing tract adjacency.")?.iter().enumerate(){
        let p=tract_ids.get(i).and_then(Value::as_str).context("Invalid tract index.")?;
        for j in ns.as_array().context("Invalid tract neighbors.")? {
            let index=j.as_u64().context("Invalid tract neighbor index.")? as usize;
            let q=tract_ids.get(index).and_then(Value::as_str).context("Tract neighbor out of range.")?;
            expected.insert((p.to_owned(),q.to_owned()));
        }
    }
    let mut actual=BTreeSet::new();
    for(i,ns)in graph.adjacency.iter().enumerate(){for &j in ns {
        let p=&ids[i][..11];let q=&ids[j][..11];
        if p!=q {actual.insert((p.to_owned(),q.to_owned()));}
    }}
    ensure!(actual==expected,"Fine adjacency quotient differs from selected tract graph: {} missing and {} extra directed parent edges. Prepare an explicitly compatible tract/fine profile; inputs were not altered.",expected.difference(&actual).count(),actual.difference(&expected).count());
    let mut edges:Vec<_>=graph.edge_weights.iter().map(|(&(u,v),&w)|(u,v,w)).collect();edges.sort_by_key(|e|(e.0,e.1));
    ensure!(graph.adjacency.iter().map(Vec::len).sum::<usize>()<=2000000 && edges.iter().all(|e|e.2.is_finite()&&e.2>0.),"Invalid or oversized fine graph boundaries.");
    let bundle=json!({"schema_version":"bisect-multiscale-input-v1","source_label":format!("Census TIGER/PL {} {}; EPSG:5070 boundaries; {} native median-cost island bridges; map simplified 0.0002 degrees",a.year,a.state,bridges.len()),"graph":{"schema_version":"bisect-block-group-graph-v1","state":a.state,"year":a.year.to_string(),"geoids":ids,"adjacency":graph.adjacency,"population":pop},"boundary_edges":edges,"geometry":{"type":"FeatureCollection","features":features}});
    let bytes=serde_json::to_vec(&bundle)?;
    ensure!(bytes.len()<=25*1024*1024,"Bundle exceeds browser 25 MiB import limit.");
    let mut sources=BTreeMap::new();
    for (key,path) in [("tract_graph",a.tract_graph.clone()),("geography",a.geography.clone()),("population",a.population.clone()),("shp",a.shape.clone()),("dbf",a.shape.with_extension("dbf")),("shx",a.shape.with_extension("shx")),("prj",a.shape.with_extension("prj"))] {
        let mut file=fs::File::open(&path)?;let mut hash=Sha256::new();let mut buffer=[0u8;65536];
        loop {let n=file.read(&mut buffer)?;if n==0{break;}hash.update(&buffer[..n]);}
        sources.insert(key,json!({"file":path.file_name().context("Missing source filename.")?.to_string_lossy(),"sha256":format!("{:x}",hash.finalize())}));
    }
    let report=json!({"schema_version":"bisect-fine-preparation-v1","state":a.state,"year":a.year,"units":records.len(),"bundle_sha256":bisect_web::catalog::digest(&bytes),"sources":sources,"projection":"EPSG:5070","minimum_shared_boundary_m":10,"display_simplification_degrees":0.0002,"bridge_policy":if a.native_island_bridges{"native-median-edge-cost"}else{"reject-disconnected"},"bridges":bridges.iter().map(|&(u,v)|json!({"from":ids[u],"to":ids[v],"cost":bridge_weight})).collect::<Vec<_>>(),"source_boundary":"Source bytes are locally hashed; source provenance is not authenticated. Bridge costs are derived graph costs, not measured shared boundaries. Population comes from PL blocks, grouped by twelve-digit GEOID; no missing populations are filled."});
    fs::write(report_path,serde_json::to_vec_pretty(&report)?)?;
    fs::write(&a.output,bytes)?;
    println!("Fine browser input: {} ({} units)",a.output.display(),records.len());
    Ok(())
}
