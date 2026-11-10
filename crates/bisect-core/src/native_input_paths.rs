//! Native State path selection; the host supplies existence observations.
#[derive(Debug,Clone,PartialEq)]
pub struct Selection{pub path:Option<String>,pub effective_resolution:String,pub fallback_to_tract:bool}

/// Availability and report discovery intentionally use different native names.
/// These are basenames, not authorized paths; the host supplies its own data root.
pub fn cvap_availability_basename(state_name: &str, year: &str) -> String {
    format!("{state_name}_cvap_{year}.csv")
}

/// The first existing candidate wins, including directories and malformed files.
/// Do not filter by requested population source or skip a failed first reader.
pub fn report_demographic_basenames(state_name: &str, state_code: &str, year: &str) -> [String; 4] {
    let name = state_name.to_lowercase().replace(' ', "_");
    let code = state_code.to_lowercase();
    [format!("{name}_vap_{year}.csv"), format!("{code}_vap_{year}.csv"),
     format!("{name}_cvap_{year}.csv"), format!("{code}_cvap_{year}.csv")]
}

#[cfg(test)]
mod demographic_tests {
    use super::*;

    #[test]
    fn raw_availability_and_ordered_normalized_report_names_stay_distinct() {
        assert_eq!(cvap_availability_basename("Research Area", "2020"), "Research Area_cvap_2020.csv");
        assert_eq!(report_demographic_basenames("Research Area", "RI", "2020"), [
            "research_area_vap_2020.csv", "ri_vap_2020.csv", "research_area_cvap_2020.csv", "ri_cvap_2020.csv"]);
        // Duplicate native candidates are intentional when name equals code.
        assert_eq!(report_demographic_basenames("RI", "RI", "2020")[..2], ["ri_vap_2020.csv", "ri_vap_2020.csv"]);
    }
}
pub fn select_adjacency_path(state:&str,year:&str,resolution:&str,mut exists:impl FnMut(&str)->bool)->Selection{
 let bg=matches!(resolution,"block_group"|"block-group");
 let filename=if bg{format!("{state}_bg_adjacency_{year}.pkl")}else{format!("{state}_adjacency_{year}.pkl")};
 for version in ["V3","V4"]{let path=format!("{version}/data/{year}/adjacency/{filename}");if exists(&path){return Selection{path:Some(path),effective_resolution:resolution.into(),fallback_to_tract:false};}}
 if bg{for version in ["V3","V4"]{let path=format!("{version}/data/{year}/adjacency/{state}_adjacency_{year}.pkl");if exists(&path){return Selection{path:Some(path),effective_resolution:"tract".into(),fallback_to_tract:true};}}}
 Selection{path:None,effective_resolution:if bg{"tract".into()}else{resolution.into()},fallback_to_tract:bg}
}

/// Native auxiliary discovery remains relative to the run working directory.
/// These names are records, not filesystem authority in portable consumers.
pub fn character_relative_path(state_name:&str,year:&str,kind:&str)->Option<String>{
 match kind{
  "economic"=>Some(format!("data/{year}/lodes/{state_name}_wac_tract.csv")),
  "housing"=>Some(format!("data/{year}/acs_housing/{state_name}_housing_{year}.csv")),
  _=>None,
 }
}

pub fn demographic_relative_path(state_name:&str,year:&str)->String{format!("data/{year}/demographics/{state_name}_demographics_{year}.csv")}
