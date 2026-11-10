fn main()->Result<(),Box<dyn std::error::Error>>{
 let manifest=fletch_core::read_cache_manifest_json(std::env::args().nth(1).ok_or("manifest path")?)?;
 let mut plan=fletch_core::plan_cache_prune(&manifest)?;
 // Portable output spelling only; candidate selection and counts are upstream.
 plan.object_root=plan.object_root.replace('\\',"/");
 for candidate in &mut plan.candidates{candidate.absolute_path=candidate.absolute_path.replace('\\',"/");}
 print!("{}",serde_json::to_string_pretty(&plan)?);Ok(())
}
