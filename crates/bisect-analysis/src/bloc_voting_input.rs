//! Shared precinct TSV ingestion. Optional variant column materializes the full test family.
use crate::{BlocVotingTest, Precinct};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
pub fn validate_annotation_coverage(tests:&[BlocVotingTest],annotations:&crate::AnnotationSet,party:&str)->anyhow::Result<()>{
    anyhow::ensure!(!annotations.annotations.is_empty(),"[INPUT] candidate annotation CSV has no annotations");
    for t in tests {anyhow::ensure!(annotations.annotations.iter().any(|a|a.candidate_name==t.candidate&&a.party==party),"[INPUT] candidate '{}' has no annotation for party '{party}'",t.candidate);}
    Ok(())
}
pub fn read_precinct_family<R:Read>(source:R,min_precincts:usize)->anyhow::Result<Vec<BlocVotingTest>>{
    anyhow::ensure!(min_precincts>=4,"[INPUT] min_precincts must be at least four for this three-coefficient regression");
    let mut reader=csv::ReaderBuilder::new().delimiter(b'\t').from_reader(source);
    let headers=reader.headers()?.clone();let mut seen=BTreeSet::new();
    anyhow::ensure!(headers.iter().all(|h|seen.insert(h)),"[INPUT] duplicate precinct TSV headers");
    let required=["candidate_name","precinct_id","county_fips","total_votes","candidate_share","pct_minority","pct_dem_baseline"];
    let columns: BTreeMap<_,_>=required.iter().map(|name|headers.iter().position(|h|h==*name).map(|i|(*name,i)).ok_or_else(||anyhow::anyhow!("[INPUT] precinct TSV missing {name}"))).collect::<Result<_,_>>()?;
    let variant_column=headers.iter().position(|h|h=="variant");
    let mut groups:BTreeMap<(String,String),Vec<Precinct>>=BTreeMap::new();let mut identities=BTreeSet::new();
    for (i,row) in reader.records().enumerate(){let row=row?;let get=|k:&str|row.get(columns[k]).unwrap_or("").trim();
        let candidate=get("candidate_name");let id=get("precinct_id");
        let variant=variant_column.map(|i|row.get(i).unwrap_or("").trim()).unwrap_or("primary");
        anyhow::ensure!(!candidate.is_empty()&&!id.is_empty()&&!variant.is_empty()&&[candidate,id,variant].iter().all(|s|s.len()<=200&&!s.chars().any(char::is_control)),"[INPUT] row {} has empty/oversized/control-character candidate, precinct or variant",i+2);
        anyhow::ensure!(!candidate.contains("::")&&!variant.contains("::"),"[INPUT] candidate/variant must not contain reserved :: family-key separator");
        anyhow::ensure!(identities.insert((candidate.to_string(),variant.to_string(),id.to_string())),"[INPUT] duplicate precinct within candidate/variant");
        let number=|k:&str|->anyhow::Result<f64>{let value=get(k).parse::<f64>()?;anyhow::ensure!(value.is_finite(),"[INPUT] row {} has non-finite {k}",i+2);Ok(value)};
        let votes=number("total_votes")?;anyhow::ensure!(votes>0.0,"[INPUT] row {} has non-positive vote weight",i+2);
        let share=number("candidate_share")?;let minority=number("pct_minority")?;let baseline=number("pct_dem_baseline")?;
        anyhow::ensure!([share,minority,baseline].iter().all(|x|(0.0..=1.0).contains(x)),"[INPUT] row {} has shares outside [0,1]",i+2);
        let county=get("county_fips");anyhow::ensure!(county.is_empty()||(county.len()==5&&county.bytes().all(|b|b.is_ascii_digit())),"[INPUT] county_fips must be five digits or empty (singleton cluster)");
        groups.entry((candidate.to_string(),variant.to_string())).or_default().push(Precinct{id:id.to_string(),county_fips:county.to_string(),total_votes:votes,candidate_share:share,pct_minority:minority,pct_dem_baseline:baseline});
    }
    anyhow::ensure!(!groups.is_empty(),"[INPUT] precinct TSV has no regression rows");
    groups.into_iter().map(|((candidate,variant),precincts)|{
        anyhow::ensure!(precincts.len()>=min_precincts,"[INPUT] candidate '{candidate}' variant '{variant}' has {} precincts, below min_precincts {min_precincts}",precincts.len());
        Ok(BlocVotingTest{candidate,variant,precincts})
    }).collect()
}

#[cfg(test)] mod tests{
    use super::*;
    fn text()->String{let mut s="candidate_name\tprecinct_id\tcounty_fips\ttotal_votes\tcandidate_share\tpct_minority\tpct_dem_baseline\n".to_string();for i in 0..4{s+=&format!("A\tp{i}\t01001\t100\t0.4\t0.3\t0.2\n");}s}
    #[test] fn reads_native_primary_schema(){let x=read_precinct_family(text().as_bytes(),4).unwrap();assert_eq!(x.len(),1);assert_eq!(x[0].variant,"primary");}
    #[test] fn checks_every_candidate_threshold(){let s=text()+"B\tp0\t01001\t100\t0.4\t0.3\t0.2\n";assert!(read_precinct_family(s.as_bytes(),4).is_err());}
    #[test] fn rejects_duplicate_precinct_and_header(){let s=text()+"A\tp0\t01001\t100\t0.4\t0.3\t0.2\n";assert!(read_precinct_family(s.as_bytes(),4).is_err());assert!(read_precinct_family(text().replace("pct_dem_baseline","pct_minority").as_bytes(),4).is_err());}
    #[test] fn rejects_invalid_shares_and_weights(){for (a,b)in [("0.4","1.4"),("100","0"),("0.3","NaN"),("01001","county")] {assert!(read_precinct_family(text().replace(a,b).as_bytes(),4).is_err());}}
}
