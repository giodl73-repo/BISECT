//! Native State/States configuration filter. This is not a legal certificate.
pub fn validate(court:bool,partition_mode:&str,proportional_weights:bool,search:Option<&str>,partisan_shares:bool)->Result<(),String>{
 if !court{return Ok(());}
 let bad=|what:&str,val:&str|Err(format!("profile=court rejects {what} '{val}': it is partisan or experimental and is not court-defensible. Use a neutral configuration or --profile open."));
 if partisan_shares{return bad("--partisan-shares","<set>");}
 if matches!(partition_mode,"partisan-weighted"|"proportional"|"proportional-section"){return bad("--partition-mode",partition_mode);}
 if proportional_weights{return bad("--weights-override","proportional");}
 if search.is_some_and(|s|!matches!(s,"single"|"convergence")){return bad("--search","non-deterministic search");}
 Ok(())
}
