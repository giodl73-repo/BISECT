fn main()->Result<(),String>{let args:Vec<String>=std::env::args().collect();bisect_cli::label_cmd::run_verify(&args[1],args.get(2).map(String::as_str))}
