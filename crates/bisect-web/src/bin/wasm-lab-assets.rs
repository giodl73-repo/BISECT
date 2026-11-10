//! Export prepared engine inputs and shared display geometry for a static WASM lab.
use anyhow::{ensure, Context, Result};
use bisect_web::{catalog::digest,data::{self,Paths}};
use clap::Parser;
use serde_json::{json,Value};
use std::{collections::{BTreeMap,BTreeSet},fs,path::{Path,PathBuf}};
#[derive(Parser)]
struct Args {
    #[arg(long,default_value=".")]root:PathBuf,
    #[arg(long,default_value="runs/lab")]store:PathBuf,
    #[arg(long,default_value="dist/laboratory")]saved_catalog:PathBuf,
    #[arg(long)]output:PathBuf,
    #[arg(long,value_delimiter=',',default_value="RI,IA,NC")]states:Vec<String>,
    #[arg(long,default_value="2020")]year:String,
}
fn asset(out:&Path,value:&Value,manifest:&mut Value)->Result<String>{
    let bytes=serde_json::to_vec(value)?;let hash=digest(&bytes);
    let path=format!("assets/{hash}.json");fs::write(out.join(&path),bytes)?;
    manifest["assets"][&path]=json!(hash);Ok(path)
}
fn main()->Result<()> {
    let args=Args::parse();let root=args.root.canonicalize()?;
    ensure!(["2000","2010","2020"].contains(&args.year.as_str()),"Unsupported Census year.");
    ensure!(!args.output.exists()||fs::read_dir(&args.output)?.next().is_none(),"Output must be new or empty.");
    let paths=Paths::new(root.clone(),root.join(args.store),root.join("target/release/bisect.exe"))?;
    ensure!(!args.states.is_empty()&&args.states.iter().all(|c|paths.codes().contains(c)),"Invalid state selection.");
    fs::create_dir_all(args.output.join("assets"))?;
    let mut manifest:Value=serde_json::from_slice(&fs::read(args.saved_catalog.join("catalog.json"))?)?;
    for (path,hash) in manifest["assets"].as_object().context("Missing saved assets.")? {
        ensure!(hash.as_str().is_some_and(|h|h.len()==64&&h.bytes().all(|b|b.is_ascii_hexdigit())),"Invalid saved hash.");
        ensure!(path==&format!("assets/{}.json",hash.as_str().context("Invalid hash.")?),"Invalid saved asset path.");
        let bytes=fs::read(args.saved_catalog.join(path))?;ensure!(digest(&bytes)==hash.as_str().unwrap(),"Saved asset hash mismatch.");fs::write(args.output.join(path),bytes)?;
    }
    manifest["graphs"]=json!({});
    for code in &args.states {
        let path=data::prepare(&paths,code,&args.year)?;
        let bytes=fs::read(&path)?;let graph=bisect_data::deserialize_adjacency(&bytes).map_err(|e|anyhow::anyhow!("{e}"))?;
        let stem=path.to_string_lossy().trim_end_matches(".adj.bin").to_owned();
        let ids:BTreeMap<String,String>=serde_json::from_slice(&fs::read(format!("{stem}_geoids.json"))?)?;
        let geoids:Vec<_>=(0..graph.n_vertices).map(|i|ids.get(&i.to_string()).cloned().context("Missing GEOID index.")).collect::<Result<_>>()?;
        ensure!(ids.len()==geoids.len(),"GEOID/graph dimension mismatch.");
        let centers:Value=serde_json::from_slice(&fs::read(format!("{stem}_centroids.json"))?)?;
        ensure!(graph.vertex_areas.len()==graph.n_vertices&&graph.vertex_ext_perimeters.len()==graph.n_vertices,"Physical geometry attributes unavailable; prepare a complete graph first.");
        let mut edges:Vec<_>=graph.edge_weights.iter().map(|(&(u,v),&w)|(u,v,w)).collect();edges.sort_by_key(|e|(e.0,e.1));
        let prepared=json!({"schema_version":1,"state":code,"year":args.year,"geoids":geoids,"adjacency":graph.adjacency,"population":graph.vertex_weights,"edges":edges,"areas":graph.vertex_areas,"exterior_perimeters":graph.vertex_ext_perimeters,"centroids":centers["centroids"]});
        let reference=asset(&args.output,&prepared,&mut manifest)?;
        manifest["graphs"][format!("{code}:{}",args.year)]=json!({"graph_ref":reference,"native_graph_sha256":digest(&bytes),"profile":"prepared-tract-graph","certified_nrs_instance":false});
        let key=format!("{code}:{}",args.year);
        let geometry_ref=if let Some(existing)=manifest["geometries"][&key].as_str() {
            existing.to_owned()
        } else {
            let geometry=data::geometry(&paths,code,&args.year)?;
            asset(&args.output,&geometry,&mut manifest)?
        };
        let geometry:Value=serde_json::from_slice(&fs::read(args.output.join(&geometry_ref))?)?;
        ensure!(geometry["state"]==*code&&geometry["year"]==args.year,"Display geometry identity mismatch.");
        let join=json!(geoids.iter().map(|id|(id.clone(),1usize)).collect::<BTreeMap<_,_>>());
        bisect_web::catalog::validate_join(&geometry,&join,1)?;
        manifest["geometries"][key]=json!(geometry_ref);
    }
    let geometry_keys:BTreeSet<_>=manifest["geometries"].as_object().context("Missing geometry index.")?.keys().cloned().collect();
    for state in manifest["catalog"]["states"].as_array_mut().context("Missing states.")? {
        let code=state["code"].as_str().unwrap().to_owned();
        for year in state["years"].as_array_mut().unwrap(){
            let available=args.states.contains(&code)&&year["year"]==args.year;
            year["available"]=json!(available);year["cached"]=json!(available);
            let vintage=year["year"].as_str().context("Invalid catalog year.")?;
            year["geometry"]=json!(geometry_keys.contains(&format!("{code}:{vintage}")));
        }
    }
    let structures=["ilp","proportional-bisect","proportional-section","simulated-annealing","capacity-clustering","regionalization","flow-construction","spectral","prime-factor","standard-bisect","nway","bfs-growth","centroidal-voronoi","moving-knife","compact-polsby","ratio-optimal","ratio-optimal-area","ratio-optimal-vra"];
    let standard=["multiscale","multiscale-adaptive","vra-recom","smc-percentile","convergence","single","multi","percentile","bisection-ensemble","short-burst","short-burst-forest","short-burst-merge-split","flip","forest-recom","merge-split","parallel-tempering"];
    manifest["catalog"]["weights"]=json!(["geographic","unweighted","county","partisan","vra-aligned","economic-character","housing-character","zone-membership"]);
    manifest["catalog"]["structures"]=json!(structures);
    manifest["catalog"]["searches"]=json!(standard);
    manifest["catalog"]["search_compatibility"]=json!(structures.iter().map(|s|(*s,if *s=="standard-bisect"{standard.to_vec()}else if ["proportional-section","compact-polsby","ratio-optimal","ratio-optimal-area","ratio-optimal-vra"].contains(s){vec!["single","multi"]}else{vec!["single"]})).collect::<BTreeMap<_,_>>());
    manifest["catalog"]["engine_available"]=json!(true);manifest["catalog"]["execution"]=json!("browser-wasm");
    let wasm=fs::read(root.join("target/wasm32-unknown-unknown/release/bisect_wasm.wasm"))?;
    manifest["wasm_sha256"]=json!(digest(&wasm));fs::write(args.output.join("bisect_wasm.wasm"),wasm)?;
    for file in ["label-build-archive.js","label-build-archive-worker.js","label-build-files.js","label-rename-files.js","plan-doctor-files.js", "tutorial-files.js", "manifest-files.js", "doctor-files.js","policy-files.js","balance-build.js","ilp-dispatch-options.js","ilp-inspection-files.js","ilp-report-handoff.js","config-run.js","config-files.js","config-template.js","config-catalog.js","config-catalog-worker.js","config-catalog-preparation.js","national-files.js","commission-files.js","multiscale-input.js","multiscale-ui.js","multiscale-input-worker.js","lab.js","lab.css","static.js","wasm-catalog.js","wasm-engine.js","wasm-worker.js","laboratory-project.js","laboratory-round-capture.js","laboratory-ilp-reports.js","laboratory-electoral.js","laboratory-coi.js","laboratory-project-worker.js","assignment-verification.js","assignment-verification-worker.js","project.js","project-handoff.js","bloc-files.js","compactness-files.js","statistics-files.js","election-analysis-settings.js","stability-files.js","label-inventory-files.js", "label-chain-files.js", "migration-files.js", "candidate-files.js", "civic-files.js", "suite-assembly-files.js","suite-files.js","research-files.js","partition-diagnostic-files.js","reproducibility-files.js","plan-map.js","report-output.js","package-files.js","configured-runtime-artifacts.js","configured-run-recipe.js","configured-source-import.js","configured-source-import-worker.js","configured-source-bundle.js","native-demographic-adoption.js","demographic-input.js","partisan-input.js","partisan-tsv-worker.js","character-input.js","character-csv-worker.js","election-input.js","election-csv-worker.js","demographic-csv-worker.js","json-worker.js"]{fs::copy(root.join("web/lab").join(file),args.output.join(file))?;}
    // Keep the browser module/worker closure current as capabilities grow.
    for entry in fs::read_dir(root.join("web/lab"))? {
        let entry=entry?;
        if entry.file_type()?.is_file() { fs::copy(entry.path(),args.output.join(entry.file_name()))?; }
    }
    let html=fs::read_to_string(root.join("web/lab/index.html"))?.replace("<body>","<body data-backend=\"wasm\">")
        .replace("<head>","<head><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self'; connect-src 'self'; style-src 'self' 'unsafe-inline'; object-src 'none'; base-uri 'none'; form-action 'none'\">")
        .replace("Configurations and completed state results survive a server restart.","Published results remain available. Live runs remain in this tab; download their run records and maps before closing it.")
        .replace("Initial graph preparation finishes before cancellation takes effect.","Cancellation terminates the active browser Worker; completed states are retained.");fs::write(args.output.join("index.html"),html)?;
    fs::write(args.output.join("catalog.json"),serde_json::to_vec_pretty(&manifest)?)?;
    println!("Static WASM laboratory: {}",args.output.display());Ok(())
}
