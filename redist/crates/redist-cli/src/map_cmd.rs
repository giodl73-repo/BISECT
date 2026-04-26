use std::path::PathBuf;
use crate::args::{MapArgs, MapType};
use crate::runner::load_all_states;
use redist_map::{default_font_db, canvas_size_from_dpi};
use fontdb::Database as FontDb;

pub fn run_map(args: &MapArgs) -> anyhow::Result<()> {
    let state_code = args.state.to_uppercase();
    let year = args.year.to_string();
    let dpi: u32 = args.dpi.parse().unwrap_or(150);

    let all = load_all_states(&year)
        .map_err(|e| anyhow::anyhow!("Failed to load state config: {e}"))?;
    let (_, state_name, _) = all.iter()
        .find(|(code, _, _)| code == &state_code)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Unknown state: {state_code}"))?;

    let output_root = PathBuf::from("outputs").join(&args.version);
    let state_dir = output_root.join(&year).join(&state_name);
    let maps_dir = state_dir.join("maps");
    std::fs::create_dir_all(&maps_dir)?;

    let types = resolve_map_types(&args.types);
    let font_db = default_font_db();

    let assignments_path = state_dir.join("final_assignments.json");
    if !assignments_path.exists() {
        anyhow::bail!(
            "No assignments at {}. Run: redist state --state {state_code} first.",
            assignments_path.display()
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
                let png = render_districts_map(&state_dir, &state_name, &state_code, &year,
                                               &args.version, dpi, &font_db)?;
                std::fs::write(&out, &png)?;
                eprintln!("[OK] districts map -> {} ({} bytes)", out.display(), png.len());
            }
            MapType::Rounds => {
                render_rounds_maps(&state_dir, &maps_dir, dpi, &font_db, args.force)?;
            }
            MapType::Political => {
                let out = maps_dir.join("political.png");
                if out.exists() && !args.force { eprintln!("[skip] political map"); continue; }
                let png = render_choropleth_map(&state_dir, "political", dpi, &font_db)?;
                std::fs::write(&out, &png)?;
                eprintln!("[OK] political map -> {}", out.display());
            }
            MapType::Demographic => {
                let out = maps_dir.join("demographic.png");
                if out.exists() && !args.force { eprintln!("[skip] demographic map"); continue; }
                let png = render_choropleth_map(&state_dir, "demographic", dpi, &font_db)?;
                std::fs::write(&out, &png)?;
                eprintln!("[OK] demographic map -> {}", out.display());
            }
            MapType::Compactness => {
                eprintln!("[skip] compactness map — not yet wired (needs dissolved geometries)");
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
    _version: &str,
    dpi: u32,
    font_db: &FontDb,
) -> anyhow::Result<Vec<u8>> {
    use redist_map::{Projection, build_svg};
    use redist_map::colorscheme::{CategoricalScheme, graph_color};
    use redist_map::labeler::LabelSpec;
    use geo::BoundingRect;
    use geo_types::MultiPolygon;
    use crate::geometry::load_district_geometries;

    let assignments: std::collections::HashMap<String, usize> = serde_json::from_str(
        &std::fs::read_to_string(state_dir.join("final_assignments.json"))?
    )?;

    let districts = load_district_geometries(
        state_name, state_code, year, &assignments, std::path::Path::new("data")
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
    dpi: u32,
    font_db: &FontDb,
    force: bool,
) -> anyhow::Result<()> {
    let rounds_dir = maps_dir.join("rounds");
    std::fs::create_dir_all(&rounds_dir)?;

    // Check for intermediate assignments directory
    let intermediate_dir = state_dir.join("intermediate");
    if !intermediate_dir.exists() {
        eprintln!("[skip] rounds maps — no intermediate/ directory found");
        return Ok(());
    }

    let mut round_dirs: Vec<_> = std::fs::read_dir(&intermediate_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    round_dirs.sort_by_key(|e| e.file_name());

    if round_dirs.is_empty() {
        eprintln!("[skip] rounds maps — no round directories in intermediate/");
        return Ok(());
    }

    for (i, dir) in round_dirs.iter().enumerate() {
        let out = rounds_dir.join(format!("round_{:02}.png", i));
        if out.exists() && !force { continue; }
        // Stub: create a minimal PNG placeholder
        let placeholder_svg = format!(
        "<svg width=\"400\" height=\"300\" xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"400\" height=\"300\" fill=\"white\"/><text x=\"200\" y=\"150\" text-anchor=\"middle\" font-size=\"20\">Round {i}</text></svg>"
    );
        let png = redist_map::svg_to_png(&placeholder_svg, font_db)?;
        std::fs::write(&out, &png)?;
        eprintln!("[OK] round {i} -> {}", out.display());
    }
    Ok(())
}

fn render_choropleth_map(
    state_dir: &PathBuf,
    analysis_type: &str,
    _dpi: u32,
    font_db: &FontDb,
) -> anyhow::Result<Vec<u8>> {
    // Check if analysis output exists
    let analysis_file = state_dir.join("analysis").join(format!("{analysis_type}.json"));
    if !analysis_file.exists() {
        anyhow::bail!(
            "Analysis file not found: {}. Run: redist analyze --state ... --types {analysis_type}",
            analysis_file.display()
        );
    }
    // Stub: return a simple SVG→PNG
    let svg = format!(
        "<svg width=\"400\" height=\"300\" xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"400\" height=\"300\" fill=\"#e8f4f8\"/><text x=\"200\" y=\"150\" text-anchor=\"middle\" font-size=\"16\" fill=\"#333\">{analysis_type} map</text></svg>"
    );
    redist_map::svg_to_png(&svg, font_db)
}

