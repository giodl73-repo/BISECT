fn main()->Result<(),String>{
    let args:Vec<String>=std::env::args().collect();
    bisect_cli::label_cmd::run_mv(&args[1],&args[2],args.get(3).is_some_and(|s|s=="force"))
}
