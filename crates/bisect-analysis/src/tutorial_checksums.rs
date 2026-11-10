//! Tutorial checksum schema and comparison, shared with native Doctor.
use serde::Deserialize;
use serde_json::{json,Value};
#[derive(Debug,Deserialize)]pub struct TutorialChecksums{
 pub schema_version:String,#[serde(default)]pub tutorial:String,
 #[serde(default)]pub pinned_inputs:Vec<PinnedInput>,#[serde(default)]pub expected_outputs:Vec<ExpectedOutput>,
 #[serde(default)]pub build_commit:String,#[serde(default)]pub pinned_at:String,
}
#[derive(Debug,Deserialize)]pub struct PinnedInput{pub name:String,#[serde(default)]pub source_url:String,pub local_path:String,pub sha256:String}
#[derive(Debug,Deserialize)]pub struct ExpectedOutput{pub path:String,pub sha256:String,#[serde(default)]pub tract_count:Option<u64>}
pub fn hash_matches(got:&str,expected:&str)->bool{got.eq_ignore_ascii_case(expected)}
pub fn assess(c:&TutorialChecksums,tutorial:&str,mut digest:impl FnMut(&str)->Option<Result<String,String>>)->Value{
 let mut rows=Vec::new();let(mut pass,mut fail,mut missing)=(0,0,0);
 for (kind,name,path,expected,url,tracts) in c.pinned_inputs.iter().map(|i|("input",i.name.as_str(),i.local_path.as_str(),i.sha256.as_str(),Some(i.source_url.as_str()),None)).chain(c.expected_outputs.iter().map(|o|("output",o.path.as_str(),o.path.as_str(),o.sha256.as_str(),None,o.tract_count))){
  let (status,got,error)=match digest(path){None=>{missing+=1;("missing",None,None)},Some(Err(e))=>{fail+=1;("read-error",None,Some(e))},Some(Ok(got))=>{let ok=hash_matches(&got,expected);if ok{pass+=1;}else{fail+=1;}(if ok{"pass"}else{"fail"},Some(got),None)}};
  rows.push(json!({"kind":kind,"name":name,"path":path,"expected_sha256":expected,"actual_sha256":got,"status":status,"read_error":error,"source_url":url,"expected_tract_count":tracts.map(|n|n.to_string())}));
 }
 json!({"native_exit_code":if fail>0{1}else{0},"pass":pass,"fail":fail,"missing":missing,"rows":rows,"tutorial_warning":if !c.tutorial.is_empty()&&c.tutorial!=tutorial{Some(format!("Checksum document declares tutorial='{}' but requested tutorial='{tutorial}'",c.tutorial))}else{None},"pinned_build_commit":c.build_commit,"pinned_at":c.pinned_at})
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn comparison_retains_native_missing_and_uppercase_behavior(){let c:TutorialChecksums=serde_json::from_value(json!({"schema_version":"tutorial-checksums v1","pinned_inputs":[{"name":"a","local_path":"a","sha256":"ABC"},{"name":"b","local_path":"b","sha256":"x"}]})).unwrap();let r=assess(&c,"demo",|p|if p=="a"{Some(Ok("abc".into()))}else{None});assert_eq!(r["pass"],1);assert_eq!(r["missing"],1);assert_eq!(r["native_exit_code"],0);}
 #[test]fn read_error_and_drift_fail(){let c:TutorialChecksums=serde_json::from_value(json!({"schema_version":"tutorial-checksums v1","expected_outputs":[{"path":"a","sha256":"x","tract_count":9007199254740993u64}]})).unwrap();let r=assess(&c,"demo",|_|Some(Err("unreadable".into())));assert_eq!(r["native_exit_code"],1);assert_eq!(r["rows"][0]["expected_tract_count"],"9007199254740993");}
}
