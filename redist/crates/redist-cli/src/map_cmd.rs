use std::path::PathBuf;
use crate::args::{MapArgs, MapScope, MapType};
use crate::runner::load_all_states;
use redist_map::{default_font_db, canvas_size_from_dpi};
use fontdb::Database as FontDb;

pub fn run_map(args: &MapArgs) -> anyhow::Result<()> {
    // Dispatch to national renderer if --scope national
    if args.scope == MapScope::National {
        let font_db = default_font_db();
        return run_national_map(args, &font_db);
    }

    let state_code = args.state.to_uppercase();
    if state_code.is_empty() {
        anyhow::bail!("--state is required for --scope state (default). Use --scope national to render all states.");
    }
    let year = args.year.to_string();
    let dpi: u32 = args.dpi.parse().unwrap_or(150);

    let all = load_all_states(&year)
        .map_err(|e| anyhow::anyhow!("Failed to load state config: {e}"))?;
    let (_, state_name, _) = all.iter()
        .find(|(code, _, _)| code == &state_code)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Unknown state: {state_code}"))?;

    // Path mirrors runner.rs: {version}/states/{state}/
    let output_root = PathBuf::from("outputs").join(&args.version);
    let state_dir = output_root.join("states").join(&state_name);
    let state_data_dir = state_dir.join("data");
    let maps_dir = state_dir.join("maps");
    std::fs::create_dir_all(&maps_dir)?;

    let types = resolve_map_types(&args.types);
    let font_db = default_font_db();

    let assignments_path = state_data_dir.join("final_assignments.json");
    if !assignments_path.exists() {
        anyhow::bail!(
            "No assignments at {}.\nRun: redist state --state {state_code} --version {} first.",
            assignments_path.display(), args.version
        );
    }

    for map_type in &types {
        match map_type {
            MapType::Districts => {
                let out = maps_dir.join("districts.png");
                if out.exists() && !args.force {
                    eprintln!("[skip] districts map (exists)");
                    continue;
                }
                let png = render_districts_map(&state_data_dir, &state_name, &state_code, &year,
                                               &args.version, dpi, &font_db)?;
                std::fs::write(&out, &png)?;
                eprintln!("[OK] districts map -> {} ({} bytes)", out.display(), png.len());
            }
            MapType::Rounds => {
                render_rounds_maps(
                    &state_dir, &maps_dir, &state_name, &state_code, &year,
                    &args.version, dpi, &font_db, args.force,
                )?;
            }
            MapType::Political => {
                let out = maps_dir.join("political.png");
                if out.exists() && !args.force { eprintln!("[skip] political map"); continue; }
                let png = render_choropleth_map(
                    &state_data_dir, &state_name, &state_code, &year,
                    &args.version, "political", dpi, &font_db,
                )?;
                std::fs::write(&out, &png)?;
                eprintln!("[OK] political map -> {}", out.display());
            }
            MapType::Demographic => {
                let out = maps_dir.join("demographic.png");
                if out.exists() && !args.force { eprintln!("[skip] demographic map"); continue; }
                let png = render_choropleth_map(
                    &state_data_dir, &state_name, &state_code, &year,
                    &args.version, "demographic", dpi, &font_db,
                )?;
                std::fs::write(&out, &png)?;
                eprintln!("[OK] demographic map -> {}", out.display());
            }
            MapType::Compactness => {
                let out = maps_dir.join("compactness.png");
                if out.exists() && !args.force { eprintln!("[skip] compactness map"); continue; }
                let png = render_choropleth_map(
                    &state_data_dir, &state_name, &state_code, &year,
                    &args.version, "compactness", dpi, &font_db,
                )?;
                std::fs::write(&out, &png)?;
                eprintln!("[OK] compactness map -> {}", out.display());
            }
            MapType::All => unreachable!(),
        }
    }
    Ok(())
}

fn resolve_map_types(types: &[MapType]) -> Vec<MapType> {
    if types.iter().any(|t| *t == MapType::All) {
        vec![MapType::Districts, MapType::Rounds, MapType::Political, MapType::Demographic]
    } else {
        types.to_vec()
    }
}

fn render_districts_map(
    state_dir: &PathBuf,
    state_name: &str,
    state_code: &str,
    year: &str,
    version: &str,
    dpi: u32,
    font_db: &FontDb,
) -> anyhow::Result<Vec<u8>> {
    use redist_map::{Projection, build_svg};
    use redist_map::colorscheme::{CategoricalScheme, graph_color};
    use redist_map::labeler::LabelSpec;
    use geo::BoundingRect;
    use geo_types::MultiPolygon;
    use crate::geometry::load_district_geometries;

    // state_dir here is actually state_data_dir (contains final_assignments.json)
    let assignments: std::collections::HashMap<String, usize> = serde_json::from_str(
        &std::fs::read_to_string(state_dir.join("final_assignments.json"))?
    )?;

    let districts = load_district_geometries(
        state_name, state_code, year, version, &assignments, std::path::Path::new("data")
    )?;

    let all_mp: MultiPolygon<f64> = MultiPolygon(
        districts.values().flat_map(|mp| mp.0.clone()).collect()
    );
    let bbox = all_mp.bounding_rect().ok_or_else(|| anyhow::anyhow!("empty geometry"))?;
    let lon_span = bbox.max().x - bbox.min().x;
    let lat_span = bbox.max().y - bbox.min().y;
    let aspect = if lat_span > 1e-9 { lon_span / lat_span } else { 1.5 };
    let (w, h) = canvas_size_from_dpi(dpi, 8.0, aspect);
    let proj = Projection::from_bbox(bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y, w, h, 0.05);

    // Adjacency-based graph coloring (use district adjacency from dissolved polygons)
    let num_districts = assignments.values().copied().max().unwrap_or(1);
    let adjacency: Vec<Vec<usize>> = (0..num_districts).map(|_| vec![]).collect();
    let scheme = CategoricalScheme::default();
    let colors = graph_color(&adjacency, &scheme);

    let mut district_list: Vec<(usize, MultiPolygon<f64>, (u8,u8,u8), LabelSpec)> = districts.into_iter()
        .map(|(id, mp)| {
            let color = colors.get(id.saturating_sub(1)).copied().unwrap_or((200, 200, 200));
            let label = LabelSpec {
                main: id.to_string(),
                annotation: None, stat: None, lineage_superscript: None,
            };
            (id, mp, color, label)
        })
        .collect();
    district_list.sort_by_key(|(id, _, _, _)| *id);

    let svg = build_svg(&district_list, &proj, w, h);
    redist_map::svg_to_png(&svg, font_db)
}

fn render_rounds_maps(
    state_dir: &PathBuf,
    maps_dir: &PathBuf,
    state_name: &str,
    state_code: &str,
    year: &str,
    version: &str,
    dpi: u32,
    font_db: &FontDb,
    force: bool,
) -> anyhow::Result<()> {
    use redist_map::{Projection, build_svg};
    use redist_map::colorscheme::CategoricalScheme;
    use redist_map::labeler::{LabelSpec, round_label};
    use geo::BoundingRect;
    use geo_types::MultiPolygon;
    use crate::geometry::load_district_geometries;

    let rounds_dir = maps_dir.join("rounds");
    std::fs::create_dir_all(&rounds_dir)?;

    // Check for intermediate assignments directory
    let intermediate_dir = state_dir.join("intermediate");
    if !intermediate_dir.exists() {
        eprintln!("[skip] rounds maps — no intermediate/ directory found (state has no bisection rounds recorded)");
        return Ok(());
    }

    // Gather round directories (depth_NN or any subdirectory)
    let mut round_dirs: Vec<_> = std::fs::read_dir(&intermediate_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    round_dirs.sort_by_key(|e| e.file_name());

    if round_dirs.is_empty() {
        eprintln!("[skip] rounds maps — no round directories in intermediate/");
        return Ok(());
    }

    let scheme = CategoricalScheme::default();

    for (i, dir) in round_dirs.iter().enumerate() {
        let out = rounds_dir.join(format!("round_{:02}.png", i));
        if out.exists() && !force { continue; }

        // Look for assignments.json inside the round directory
        let assignments_path = dir.path().join("assignments.json");
        if !assignments_path.exists() {
            eprintln!("[skip] round {i} — no assignments.json in {}", dir.path().display());
            continue;
        }

        let round_assignments: std::collections::HashMap<String, usize> = serde_json::from_str(
            &std::fs::read_to_string(&assignments_path)?
        )?;

        let districts = match load_district_geometries(
            state_name, state_code, year, version,
            &round_assignments, std::path::Path::new("data"),
        ) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[warn] round {i} geometry error: {e}");
                continue;
            }
        };

        let all_mp: MultiPolygon<f64> = MultiPolygon(
            districts.values().flat_map(|mp| mp.0.clone()).collect()
        );
        let bbox = match all_mp.bounding_rect() {
            Some(b) => b,
            None => { eprintln!("[skip] round {i} — empty geometry"); continue; }
        };
        let lon_span = bbox.max().x - bbox.min().x;
        let lat_span = bbox.max().y - bbox.min().y;
        let aspect = if lat_span > 1e-9 { lon_span / lat_span } else { 1.5 };
        let (w, h) = redist_map::canvas_size_from_dpi(dpi, 8.0, aspect);
        let proj = Projection::from_bbox(
            bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y, w, h, 0.05
        );

        let total_districts = districts.len();
        let mut district_list: Vec<(usize, MultiPolygon<f64>, (u8, u8, u8), LabelSpec)> =
            districts.into_iter()
                .map(|(id, mp)| {
                    let color = scheme.color(id.saturating_sub(1));
                    // annotation shows how many districts this region will become
                    let label = round_label(id, total_districts, total_districts);
                    (id, mp, color, label)
                })
                .collect();
        district_list.sort_by_key(|(id, _, _, _)| *id);

        let svg = build_svg(&district_list, &proj, w, h);
        let png = redist_map::svg_to_png(&svg, font_db)?;
        std::fs::write(&out, &png)?;
        eprintln!("[OK] round {i} -> {} ({} districts)", out.display(), total_districts);
    }
    Ok(())
}

fn render_choropleth_map(
    state_data_dir: &PathBuf,
    state_name: &str,
    state_code: &str,
    year: &str,
    version: &str,
    analysis_type: &str,
    dpi: u32,
    font_db: &FontDb,
) -> anyhow::Result<Vec<u8>> {
    use redist_map::{Projection, build_svg};
    use redist_map::colorscheme::{PoliticalScheme, DemographicScheme, CompactnessScheme};
    use redist_map::labeler::{LabelSpec, political_label, demographic_label, compactness_label};
    use geo::BoundingRect;
    use geo_types::MultiPolygon;
    use crate::geometry::load_district_geometries;

    // Check if analysis output exists
    let analysis_file = state_data_dir.parent()
        .unwrap_or(state_data_dir)
        .join("analysis")
        .join(format!("{analysis_type}.json"));
    if !analysis_file.exists() {
        anyhow::bail!(
            "Analysis file not found: {}. Run: redist analyze --state {} --year {year} --types {analysis_type}",
            analysis_file.display(), state_code
        );
    }

    // Load final assignments
    let assignments: std::collections::HashMap<String, usize> = serde_json::from_str(
        &std::fs::read_to_string(state_data_dir.join("final_assignments.json"))?
    )?;

    // Load analysis JSON
    let analysis_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&analysis_file)?
    )?;

    // Build district_stat map: district_id → stat value (dem_pct or pct_minority)
    let mut district_stats: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
    if let Some(districts) = analysis_json.get("districts").and_then(|d| d.as_array()) {
        for d in districts {
            let id = d.get("district").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let stat = match analysis_type {
                "political"   => d.get("dem_pct").and_then(|v| v.as_f64()).unwrap_or(0.5),
                "demographic" => d.get("pct_minority").and_then(|v| v.as_f64()).unwrap_or(0.0),
                "compactness" => d.get("polsby_popper").and_then(|v| v.as_f64()).unwrap_or(0.3),
                _ => 0.0,
            };
            district_stats.insert(id, stat);
        }
    }

    // Load dissolved district geometries
    let districts = load_district_geometries(
        state_name, state_code, year, version,
        &assignments, std::path::Path::new("data"),
    )?;

    // Build bbox from all district polygons
    let all_mp: MultiPolygon<f64> = MultiPolygon(
        districts.values().flat_map(|mp| mp.0.clone()).collect()
    );
    let bbox = all_mp.bounding_rect()
        .ok_or_else(|| anyhow::anyhow!("empty geometry for {state_name}"))?;
    let lon_span = bbox.max().x - bbox.min().x;
    let lat_span = bbox.max().y - bbox.min().y;
    let aspect = if lat_span > 1e-9 { lon_span / lat_span } else { 1.5 };
    let (w, h) = redist_map::canvas_size_from_dpi(dpi, 8.0, aspect);
    let proj = Projection::from_bbox(
        bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y, w, h, 0.05
    );

    // Build district list with choropleth colors and stat labels
    let gray = (200u8, 200u8, 200u8);
    let mut district_list: Vec<(usize, MultiPolygon<f64>, (u8, u8, u8), LabelSpec)> =
        districts.into_iter()
            .map(|(id, mp)| {
                let stat = district_stats.get(&id).copied();
                let (color, label): ((u8, u8, u8), LabelSpec) = match analysis_type {
                    "political" => {
                        let frac = stat.unwrap_or(0.5);
                        (PoliticalScheme.color(frac), political_label(id, frac))
                    }
                    "demographic" => {
                        let frac = stat.unwrap_or(0.0);
                        (DemographicScheme.color(frac), demographic_label(id, frac))
                    }
                    "compactness" => {
                        let pp = stat.unwrap_or(0.3);
                        (CompactnessScheme.color(pp), compactness_label(id, pp))
                    }
                    _ => (gray, LabelSpec {
                        main: id.to_string(),
                        annotation: None, stat: None, lineage_superscript: None,
                    }),
                };
                (id, mp, color, label)
            })
            .collect();
    district_list.sort_by_key(|(id, _, _, _)| *id);

    let svg = build_svg(&district_list, &proj, w, h);
    redist_map::svg_to_png(&svg, font_db)
}


// ── National map ──────────────────────────────────────────────────────────────

fn run_national_map(args: &MapArgs, font_db: &FontDb) -> anyhow::Result<()> {
    use redist_map::{InsetProjection, svg_to_png};
    use redist_map::colorscheme::{CategoricalScheme, PoliticalScheme, DemographicScheme,
                                   CompactnessScheme, graph_color};
    use redist_map::labeler::{LabelSpec, political_label, demographic_label, compactness_label,
                               adaptive_font_size, label_fits, halo_text_svg, largest_component};
    use crate::geometry::{load_district_geometries, state_name_to_code};
    use geo_types::MultiPolygon;
    use polylabel::polylabel;

    let dpi: u32 = args.dpi.parse().unwrap_or(150);
    let (w, h) = canvas_size_from_dpi(dpi, 16.0, 1.6);
    let proj = InsetProjection::us_national(w, h);

    let output_root = PathBuf::from("outputs").join(&args.version);
    let national_maps_dir = output_root.join("national").join("maps");
    std::fs::create_dir_all(&national_maps_dir)?;

    let types = resolve_map_types(&args.types);
    let year = args.year.to_string();

    let states_dir = output_root.join("states");
    if !states_dir.exists() {
        anyhow::bail!("No states directory at {}. Run: redist states first.", states_dir.display());
    }
    let mut state_entries: Vec<_> = std::fs::read_dir(&states_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    state_entries.sort_by_key(|e| e.file_name());

    for map_type in &types {
        let type_name = map_type.name();
        if type_name == "rounds" { eprintln!("[skip] national rounds map — not applicable"); continue; }
        let out = national_maps_dir.join(format!("{type_name}.png"));
        if out.exists() && !args.force {
            eprintln!("[skip] national {type_name}"); continue;
        }

        let mut all_districts: Vec<(usize, MultiPolygon<f64>, (u8,u8,u8), LabelSpec)> = vec![];
        let scheme = CategoricalScheme::default();
        let mut loaded = 0usize;
        let mut skipped: Vec<String> = vec![];
        let gray = (200u8, 200u8, 200u8);

        for entry in &state_entries {
            let state_name = entry.file_name().to_string_lossy().into_owned();
            let state_data_dir = entry.path().join("data");
            let asgn_path = state_data_dir.join("final_assignments.json");
            if !asgn_path.exists() { continue; }

            let state_code = match state_name_to_code(&state_name) {
                Some(c) => c,
                None => { skipped.push(format!("{state_name} (unknown code)")); continue; }
            };

            let raw: std::collections::HashMap<String, usize> = match
                std::fs::read_to_string(&asgn_path)
                    .and_then(|s| serde_json::from_str(&s)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
            {
                Ok(v) => v,
                Err(e) => { skipped.push(format!("{state_name} (read: {e})")); continue; }
            };

            let analysis: Option<serde_json::Value> = {
                let ap = entry.path().join("analysis").join(format!("{type_name}.json"));
                ap.exists().then(|| std::fs::read_to_string(&ap).ok()
                    .and_then(|s| serde_json::from_str(&s).ok())).flatten()
            };

            let geoms = match load_district_geometries(
                &state_name, state_code, &year, &args.version, &raw, std::path::Path::new("data")
            ) {
                Ok(g) => g,
                Err(e) => { skipped.push(format!("{state_name} ({e})")); continue; }
            };

            let stat_map: std::collections::HashMap<usize, f64> = analysis.as_ref()
                .and_then(|a| a.get("districts")).and_then(|d| d.as_array())
                .map(|ds| ds.iter().filter_map(|d| {
                    let id = d.get("district")?.as_u64()? as usize;
                    let s = match type_name {
                        "political"   => d.get("dem_pct")?.as_f64(),
                        "demographic" => d.get("pct_minority")?.as_f64(),
                        "compactness" => d.get("polsby_popper")?.as_f64(),
                        _ => None,
                    }?;
                    Some((id, s))
                }).collect())
                .unwrap_or_default();

            let num_d = raw.values().copied().max().unwrap_or(1);
            let adj: Vec<Vec<usize>> = (0..num_d).map(|_| vec![]).collect();
            let colors = graph_color(&adj, &scheme);

            for (district_id, mp) in geoms {
                let stat = stat_map.get(&district_id).copied();
                let (color, label) = match type_name {
                    "political"   => { let f = stat.unwrap_or(0.5); (PoliticalScheme.color(f), political_label(district_id, f)) }
                    "demographic" => { let f = stat.unwrap_or(0.0); (DemographicScheme.color(f), demographic_label(district_id, f)) }
                    "compactness" => { let f = stat.unwrap_or(0.3); (CompactnessScheme.color(f), compactness_label(district_id, f)) }
                    _ => {
                        let c = colors.get(district_id.saturating_sub(1)).copied().unwrap_or(gray);
                        (c, LabelSpec { main: district_id.to_string(), annotation: None, stat: None, lineage_superscript: None })
                    }
                };
                all_districts.push((district_id, mp, color, label));
            }
            loaded += 1;
        }

        if !skipped.is_empty() {
            eprintln!("WARNING: {} states omitted from national {type_name} map:", skipped.len());
            for s in &skipped { eprintln!("  - {s}"); }
        }
        eprintln!("[national {type_name}] {} districts across {loaded} states", all_districts.len());

        // Build SVG with InsetProjection
        let mut svg = format!(
            "<svg width=\"{w}\" height=\"{h}\" xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"{w}\" height=\"{h}\" fill=\"white\"/>"
        );
        for (_, mp, color, _) in &all_districts {
            let (r, g, b) = color;
            for poly in &mp.0 {
                let d: String = {
                    let mut path = String::new();
                    for (i, c) in poly.exterior().coords().enumerate() {
                        let (x, y) = proj.project(c.x, c.y);
                        if i == 0 { path.push_str(&format!("M {x:.1} {y:.1}")); }
                        else { path.push_str(&format!(" L {x:.1} {y:.1}")); }
                    }
                    path.push('Z'); path
                };
                svg.push_str(&format!(
                    "<path d=\"{d}\" fill=\"rgb({r},{g},{b})\" opacity=\"0.85\" stroke=\"#555\" stroke-width=\"0.3\"/>"
                ));
            }
        }
        // Labels (very small on national scale — skip tiny districts)
        let label_radius = proj.continental_height() * 0.008;
        for (_, mp, _, label) in &all_districts {
            if let Some(poly) = largest_component(mp) {
                if let Ok(center) = polylabel(poly, &1.0) {
                    let (cx, cy) = proj.project(center.x(), center.y());
                    let font_px = adaptive_font_size(label_radius).min(7.0);
                    if label_fits(&label.main, font_px, label_radius) && font_px >= 4.0 {
                        svg.push_str(&halo_text_svg(cx, cy, &label.main, "", font_px));
                    }
                }
            }
        }
        svg.push_str("</svg>");

        let png = svg_to_png(&svg, font_db)?;
        std::fs::write(&out, &png)?;
        eprintln!("[OK] national {type_name} -> {} ({} bytes)", out.display(), png.len());
    }
    Ok(())
}
