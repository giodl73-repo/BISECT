fn main(){
 let args:Vec<_>=std::env::args().skip(1).collect();
 if args.len()!=2{eprintln!("Expected output root and label, or --all.");std::process::exit(2);}
 bisect_cli::doctor::run_doctor(&bisect_cli::args::DoctorArgs{state:"RI".into(),year:"2020".into(),chamber:"congressional".into(),resolution:"tract".into(),output_base:args[0].clone(),label:if args[1]=="--all"{None}else{Some(args[1].clone())},version:"v1".into(),all:args[1]=="--all",verify_manifest:None,check_tutorial_data:false,tutorial:"vermont-2020".into()});
}
