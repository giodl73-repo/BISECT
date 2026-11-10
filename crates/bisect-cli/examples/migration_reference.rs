fn main()->anyhow::Result<()>{
 let a:Vec<String>=std::env::args().collect();
 bisect_cli::migrate::run_migrate(std::path::Path::new(&a[1]),&a[2],&a[3],false)
}
