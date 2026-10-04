use anyhow::{bail, ensure, Context, Result};
use bisect_data::{deserialize_adjacency, serialize_adjacency, AdjacencyGraph};
use geo::{Centroid, MapCoords, Simplify};
use geo_types::{Geometry, MultiPolygon, Polygon};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct Paths {
    pub root: PathBuf,
    pub store: PathBuf,
    pub engine: PathBuf,
    pub data: PathBuf,
    pub source_outputs: PathBuf,
    pub manifest: bisect_cli::fetch::Manifest,
}

impl Paths {
    pub fn new(root: PathBuf, store: PathBuf, engine: PathBuf) -> Result<Self> {
        let manifest_path = std::env::var_os("BISECT_MANIFEST")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("data/manifest.json"));
        let manifest: bisect_cli::fetch::Manifest =
            serde_json::from_slice(&std::fs::read(manifest_path)?)?;
        let resolve = |p: &str| {
            let p = PathBuf::from(p);
            if p.is_absolute() {
                p
            } else {
                root.join(p)
            }
        };
        let paths = Self {
            data: resolve(&manifest.local_data_dir),
            source_outputs: resolve(&manifest.local_outputs_dir),
            root,
            store,
            engine,
            manifest,
        };
        std::fs::create_dir_all(paths.store.join("jobs"))?;
        std::fs::create_dir_all(paths.store.join("cache"))?;
        Ok(paths)
    }
    pub fn codes(&self) -> Vec<String> {
        let mut codes: Vec<_> = self
            .manifest
            .states
            .keys()
            .filter(|c| c.as_str() != "DC")
            .cloned()
            .collect();
        codes.sort();
        codes
    }
    pub fn shape(&self, code: &str, year: &str) -> Option<PathBuf> {
        let fips = &self.manifest.states.get(code)?.fips;
        let yy = &year[2..];
        [
            format!("tl_{year}_{fips}_tract"),
            format!("tl_{year}_{fips}_tract{yy}"),
            format!("tl_2010_{fips}_tract{yy}"),
        ]
        .into_iter()
        .map(|stem| {
            self.data
                .join(year)
                .join("tiger/tracts")
                .join(&stem)
                .join(format!("{stem}.shp"))
        })
        .find(|p| p.is_file())
    }
    fn populations(&self, code: &str, year: &str) -> (PathBuf, PathBuf) {
        let name = self.manifest.states[code]
            .name
            .to_lowercase()
            .replace(' ', "_");
        let prefix = code.to_lowercase();
        let base = self.data.join(year).join("redistricting").join(name);
        (
            base.join(format!("{prefix}geo{year}.pl")),
            base.join(format!("{prefix}00001{year}.pl")),
        )
    }
    pub fn source_graph(&self, code: &str, year: &str) -> Option<PathBuf> {
        let file = format!("{}_adjacency_{year}.adj.bin", code.to_lowercase());
        ["V3", "V4"]
            .into_iter()
            .map(|version| {
                self.source_outputs
                    .join(version)
                    .join("data")
                    .join(year)
                    .join("adjacency")
                    .join(&file)
            })
            .find(|p| p.is_file())
    }
    pub fn cache(&self, code: &str, year: &str) -> PathBuf {
        self.store
            .join("cache")
            .join(format!("{}_adjacency_{year}.adj.bin", code.to_lowercase()))
    }
    pub fn available(&self, code: &str, year: &str) -> bool {
        let (geo, pop) = self.populations(code, year);
        self.shape(code, year).is_some()
            && (self.cache(code, year).is_file()
                || self.source_graph(code, year).is_some()
                || (geo.is_file() && (year == "2000" || pop.is_file())))
    }
    pub fn catalog(&self) -> Value {
        let policy = bisect_cli::policy::LocationRegistry::load();
        let states: Vec<_> = self.codes().into_iter().map(|code| {
            let state = &self.manifest.states[&code];
            let years: Vec<_> = ["2000", "2010", "2020"].into_iter().map(|year| {
                let districts: BTreeMap<_, _> = ["congressional", "house", "senate"].into_iter().map(|chamber| (chamber, policy.chamber_districts(&code, chamber, year).or_else(|| if chamber == "congressional" { state.districts.get(year).copied() } else { None }))).collect();
                json!({"year":year,"available":self.available(&code,year),"geometry":self.shape(&code,year).is_some(),"cached":self.cache(&code,year).is_file() || self.source_graph(&code,year).is_some(),"districts":districts})
            }).collect();
            json!({"code":code,"name":state.name,"fips":state.fips,"years":years})
        }).collect();
        json!({"states":states,"engine_available":self.engine.is_file(),"structures":crate::model::STRUCTURES,"weights":crate::model::WEIGHTS,"searches":crate::model::SEARCHES,"search_compatibility":crate::model::STRUCTURES.iter().map(|s| ((*s).to_owned(),crate::model::supported_searches(s))).collect::<std::collections::BTreeMap<_,_>>(),"resolution":"tract","population_source":"total","execution":"heuristic","data_root":self.data,"store":self.store})
    }
    pub fn district_count(&self, code: &str, year: &str, chamber: &str) -> Result<usize> {
        bisect_cli::policy::LocationRegistry::load()
            .chamber_districts(code, chamber, year)
            .or_else(|| {
                if chamber == "congressional" {
                    self.manifest.states[code].districts.get(year).copied()
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("No {chamber} district count for {code} {year}."))
    }
}

pub fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let temp = path.with_extension("tmp");
    std::fs::write(&temp, serde_json::to_vec_pretty(value)?)?;
    std::fs::rename(temp, path)?;
    Ok(())
}

/// Preparation is an explicit experimental graph profile, separate from certified NRS inputs.
pub fn prepare(paths: &Paths, code: &str, year: &str) -> Result<PathBuf> {
    if let Some(source) = paths.source_graph(code, year) {
        return Ok(source);
    }
    let target = paths.cache(code, year);
    if target.exists() {
        return Ok(target);
    }
    let shape = paths
        .shape(code, year)
        .context("Missing TIGER tract geometry.")?;
    let records = bisect_data::read_tiger_tracts(&shape)?;
    let (geo, pop) = paths.populations(code, year);
    let blocks = bisect_data::read_pl94_block_populations_for_year(&geo, &pop, year.parse()?)?;
    let mut populations: HashMap<String, i64> = HashMap::new();
    for block in blocks {
        *populations.entry(block.geoid[..11].to_owned()).or_default() += block.population;
    }
    let input_population: i64 = populations.values().sum();
    let projected: Vec<_> = records
        .iter()
        .map(|record| {
            let geometry = bisect_map::wkb_to_geometry(&record.geometry_wkb)?;
            let Geometry::MultiPolygon(mp) = (match geometry {
                Geometry::Polygon(p) => Geometry::MultiPolygon(MultiPolygon(vec![p])),
                other => other,
            }) else {
                bail!("Unsupported TIGER geometry.");
            };
            Ok(mp.map_coords(|coord| {
                let (x, y) = bisect_data::projection::nad83_to_epsg5070(coord.x, coord.y);
                geo_types::Coord { x, y }
            }))
        })
        .collect::<Result<_>>()?;
    let wkbs: Vec<_> = projected
        .iter()
        .map(bisect_data::tiger::geo_to_wkb_multipolygon)
        .collect();
    let mut graph = bisect_data::build_adjacency_graph(&wkbs, 10.0)?;
    graph.vertex_weights = records
        .iter()
        .map(|r| populations.get(&r.geoid).copied().unwrap_or(0))
        .collect();
    ensure!(
        graph.vertex_weights.iter().sum::<i64>() == input_population && input_population > 0,
        "TIGER/population join did not retain the full state population."
    );
    graph.set_areas(records.iter().map(|r| r.aland as f64).collect());
    let centers: Vec<_> = projected
        .iter()
        .map(|g| {
            g.centroid()
                .map(|c| (c.x(), c.y()))
                .context("Empty tract geometry.")
        })
        .collect::<Result<_>>()?;
    let geoids: Vec<_> = records.iter().map(|r| r.geoid.clone()).collect();
    let mut bridges = bisect_data::connect_island_components(&graph.adjacency, &centers, &geoids);
    bridges.sort();
    let mut lengths: Vec<_> = graph.edge_weights.values().copied().collect();
    lengths.sort_by(f64::total_cmp);
    let bridge_weight = lengths.get(lengths.len() / 2).copied().unwrap_or(1.0);
    for &(u, v) in &bridges {
        graph.adjacency[u].push(v);
        graph.adjacency[v].push(u);
        graph
            .edge_weights
            .insert((u.min(v), u.max(v)), bridge_weight);
    }
    for neighbors in &mut graph.adjacency {
        neighbors.sort();
        neighbors.dedup();
    }
    graph.n_edges = graph.edge_weights.len();
    let stem = target
        .to_string_lossy()
        .trim_end_matches(".adj.bin")
        .to_owned();
    let geoid_map: BTreeMap<_, _> = geoids
        .iter()
        .enumerate()
        .map(|(i, g)| (i.to_string(), g))
        .collect();
    write_json(&PathBuf::from(format!("{stem}_geoids.json")), &geoid_map)?;
    let centroids: Vec<_> = records
        .iter()
        .map(|r| {
            bisect_map::wkb_to_geometry(&r.geometry_wkb)
                .ok()
                .and_then(|g| g.centroid())
                .map(|p| [p.x(), p.y()])
        })
        .collect();
    ensure!(
        centroids.iter().all(Option::is_some),
        "Missing geographic centroids."
    );
    write_json(
        &PathBuf::from(format!("{stem}_centroids.json")),
        &json!({"centroids":centroids}),
    )?;
    write_json(
        &target.with_extension("profile.json"),
        &json!({"profile":"lab-tract-v1","year":year,"state":code,"units":graph.n_vertices,"population":input_population,"minimum_shared_boundary_m":10,"projection":"EPSG:5070","island_bridge_weight":"median retained edge weight","bridge_count":bridges.len(),"geometry":shape,"geography":geo,"population_file":pop,"certified_nrs_instance":false}),
    )?;
    // Write the binary last so a failed preparation cannot appear complete.
    std::fs::write(&target, serialize_adjacency(&graph))?;
    Ok(target)
}

fn polygon_coordinates(polygon: &Polygon<f64>) -> Value {
    let rings: Vec<_> = std::iter::once(polygon.exterior())
        .chain(polygon.interiors())
        .map(|ring| ring.0.iter().map(|c| [c.x, c.y]).collect::<Vec<_>>())
        .collect();
    json!(rings)
}

pub fn geometry(paths: &Paths, code: &str, year: &str) -> Result<Value> {
    let cache = paths
        .store
        .join("cache")
        .join(format!("{code}_{year}_map.json"));
    if cache.exists() {
        return Ok(serde_json::from_slice(&std::fs::read(cache)?)?);
    }
    let records = bisect_data::read_tiger_tracts(
        paths
            .shape(code, year)
            .context("Geometry unavailable for this state and year.")?,
    )?;
    let features:Vec<_>=records.iter().map(|record| {
        let geom=bisect_map::wkb_to_geometry(&record.geometry_wkb)?;
        let polygons=match geom { Geometry::Polygon(p)=>vec![p],Geometry::MultiPolygon(mp)=>mp.0,_=>bail!("Unsupported geometry.") };
        let coordinates:Vec<_>=polygons.iter().map(|p|polygon_coordinates(&p.simplify(&0.0002))).collect();
        Ok(json!({"type":"Feature","properties":{"geoid":record.geoid,"state":code,"county":&record.geoid[..5]},"geometry":{"type":"MultiPolygon","coordinates":coordinates}}))
    }).collect::<Result<_>>()?;
    let result = json!({"type":"FeatureCollection","state":code,"year":year,"display_simplification_degrees":0.0002,"features":features});
    write_json(&cache, &result)?;
    Ok(result)
}

pub fn assignments_by_geoid(
    graph_path: &Path,
    assignments: &HashMap<String, usize>,
) -> Result<BTreeMap<String, usize>> {
    let stem = graph_path
        .to_string_lossy()
        .trim_end_matches(".adj.bin")
        .to_owned();
    let geoids: HashMap<String, String> =
        serde_json::from_slice(&std::fs::read(format!("{stem}_geoids.json"))?)?;
    let mapped: BTreeMap<_, _> = assignments
        .iter()
        .map(|(index, district)| {
            geoids
                .get(index)
                .map(|g| (g.clone(), *district))
                .context("Assignment is missing a GEOID join.")
        })
        .collect::<Result<_>>()?;
    ensure!(
        mapped.len() == assignments.len(),
        "GEOID mapping is not one to one."
    );
    Ok(mapped)
}

pub fn evaluate(
    graph: &AdjacencyGraph,
    assignments: &HashMap<String, usize>,
    geoids: &BTreeMap<String, usize>,
    k: usize,
    weights: &str,
    alpha: f64,
) -> Result<Value> {
    ensure!(
        assignments.len() == graph.n_vertices,
        "Incomplete assignment coverage."
    );
    let assignment: Vec<_> = (0..graph.n_vertices)
        .map(|i| {
            assignments
                .get(&i.to_string())
                .copied()
                .context("Missing graph unit assignment.")
        })
        .collect::<Result<_>>()?;
    ensure!(
        assignment.iter().all(|&d| (1..=k).contains(&d)),
        "Out of range district assignment."
    );
    let mut populations = vec![0_i64; k];
    let mut counts = vec![0_usize; k];
    for (i, &d) in assignment.iter().enumerate() {
        populations[d - 1] += graph.vertex_weights[i];
        counts[d - 1] += 1;
    }
    ensure!(
        counts.iter().all(|&n| n > 0),
        "Empty district in engine output."
    );
    let total: i64 = populations.iter().sum();
    ensure!(total > 0, "No population.");
    let ideal = total as f64 / k as f64;
    let mut component_counts = vec![0; k];
    let mut visited = vec![false; graph.n_vertices];
    for i in 0..graph.n_vertices {
        if visited[i] {
            continue;
        }
        let district = assignment[i];
        component_counts[district - 1] += 1;
        let mut stack = vec![i];
        visited[i] = true;
        while let Some(v) = stack.pop() {
            for &neighbor in &graph.adjacency[v] {
                if !visited[neighbor] && assignment[neighbor] == district {
                    visited[neighbor] = true;
                    stack.push(neighbor);
                }
            }
        }
    }
    let mut counties: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    for (geoid, &district) in geoids {
        counties
            .entry(geoid[..5].into())
            .or_default()
            .insert(district);
    }
    let mut cut = 0.0;
    let mut cut_edges = 0;
    let mut weighted_cut = 0.0;
    for (&(u, v), &length) in &graph.edge_weights {
        if assignment[u] != assignment[v] {
            cut += length;
            cut_edges += 1;
        }
    }
    if weights == "unweighted" {
        weighted_cut = cut_edges as f64;
    } else if weights == "geographic" {
        weighted_cut = cut;
    }
    // County cost is filled by collect_result using the actual index-to-GEOID join.
    let districts:Vec<_>=populations.iter().enumerate().map(|(i,&population)|json!({"district":i+1,"population":population,"units":counts[i],"deviation_percent":100.0*(population as f64/ideal-1.0),"components":component_counts[i]})).collect();
    Ok(
        json!({"population":total,"units":graph.n_vertices,"district_count":k,"graph_boundary_m":cut,"cut_edges":cut_edges,"weighted_boundary":if weights=="county" {Value::Null} else {json!(weighted_cut)},"alpha_county":alpha,"split_counties":counties.values().filter(|set|set.len()>1).count(),"max_deviation_percent":districts.iter().filter_map(|d|d["deviation_percent"].as_f64()).map(f64::abs).fold(0.0,f64::max),"contiguous":component_counts.iter().all(|&c|c==1),"districts":districts,"optimality":"unproved","boundary_includes_synthetic_bridges":true}),
    )
}

pub fn collect_result(
    paths: &Paths,
    job: &crate::model::Job,
    code: &str,
    k: usize,
    graph_path: &Path,
) -> Result<Value> {
    let label = format!("{}_{}", job.id, code.to_lowercase());
    let data = paths
        .store
        .join("jobs")
        .join(&job.id)
        .join("engine")
        .join(&job.config.year)
        .join("plans")
        .join(label)
        .join("data");
    let assignments: HashMap<String, usize> =
        serde_json::from_slice(&std::fs::read(data.join("final_assignments.json"))?)?;
    let geoids = assignments_by_geoid(graph_path, &assignments)?;
    let graph = deserialize_adjacency(&std::fs::read(graph_path)?)?;
    let mut metrics = evaluate(
        &graph,
        &assignments,
        &geoids,
        k,
        &job.config.weights,
        job.config.alpha_county,
    )?;
    if job.config.weights == "county" {
        let stem = graph_path
            .to_string_lossy()
            .trim_end_matches(".adj.bin")
            .to_owned();
        let index_geoids: HashMap<String, String> =
            serde_json::from_slice(&std::fs::read(format!("{stem}_geoids.json"))?)?;
        let cost: f64 = graph
            .edge_weights
            .iter()
            .filter(|((u, v), _)| assignments[&u.to_string()] != assignments[&v.to_string()])
            .map(|(&(u, v), &w)| {
                w * (1.0
                    + if index_geoids[&u.to_string()][..5] == index_geoids[&v.to_string()][..5] {
                        job.config.alpha_county
                    } else {
                        0.0
                    })
            })
            .sum();
        metrics["weighted_boundary"] = json!(cost);
    }
    metrics["balance_passed"] = json!(
        metrics["max_deviation_percent"]
            .as_f64()
            .unwrap_or(f64::INFINITY)
            <= job.config.balance_tolerance + 1e-8
    );
    metrics["graph_path"] = json!(graph_path);
    use sha2::{Digest, Sha256};
    metrics["graph_sha256"] = json!(format!("{:x}", Sha256::digest(std::fs::read(graph_path)?)));
    let stem = graph_path
        .to_string_lossy()
        .trim_end_matches(".adj.bin")
        .to_owned();
    metrics["geoid_join_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(std::fs::read(format!("{stem}_geoids.json"))?)
    ));
    metrics["engine_provenance"] =
        serde_json::from_slice(&std::fs::read(data.join("provenance.json"))?)?;
    let mut map = geometry(paths, code, &job.config.year)?;
    for feature in map["features"]
        .as_array_mut()
        .context("Missing geometry features.")?
    {
        let geoid = feature["properties"]["geoid"]
            .as_str()
            .context("Missing geometry GEOID.")?;
        let district = geoids.get(geoid).copied();
        feature["properties"]["district"] = json!(district);
    }
    let mapped_units = map["features"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| !f["properties"]["district"].is_null())
        .count();
    ensure!(
        mapped_units == geoids.len(),
        "Map join does not cover every assigned unit."
    );
    let base = paths.store.join("jobs").join(&job.id);
    write_json(&base.join(format!("{code}_map.json")), &map)?;
    write_json(&base.join(format!("{code}_assignments.json")), &geoids)?;
    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metrics_detect_disconnects_and_population_imbalance() {
        let graph = AdjacencyGraph {
            adjacency: vec![vec![1], vec![0, 2], vec![1, 3], vec![2]],
            vertex_weights: vec![3, 1, 2, 2],
            edge_weights: HashMap::from([((0, 1), 2.0), ((1, 2), 3.0), ((2, 3), 4.0)]),
            n_vertices: 4,
            n_edges: 3,
            vertex_areas: vec![],
            vertex_ext_perimeters: vec![],
        };
        let assignments = HashMap::from([
            ("0".into(), 1),
            ("1".into(), 2),
            ("2".into(), 1),
            ("3".into(), 2),
        ]);
        let geoids = BTreeMap::from([
            ("44001000100".into(), 1),
            ("44001000200".into(), 2),
            ("44003000100".into(), 1),
            ("44003000200".into(), 2),
        ]);
        let result = evaluate(&graph, &assignments, &geoids, 2, "geographic", 0.0).unwrap();
        assert_eq!(result["contiguous"], false);
        assert_eq!(result["graph_boundary_m"], 9.0);
        assert_eq!(result["split_counties"], 2);
        assert_eq!(result["max_deviation_percent"], 25.0);
    }
}
