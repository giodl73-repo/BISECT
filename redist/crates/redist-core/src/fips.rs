/// Map 2-letter postal code → Census FIPS code (zero-padded to 2 digits).
/// Returns None for unknown codes. The caller decides the fallback.
pub fn state_code_to_fips(code: &str) -> Option<&'static str> {
    match code {
        "AL" => Some("01"), "AK" => Some("02"), "AZ" => Some("04"),
        "AR" => Some("05"), "CA" => Some("06"), "CO" => Some("08"),
        "CT" => Some("09"), "DE" => Some("10"), "DC" => Some("11"),
        "FL" => Some("12"), "GA" => Some("13"), "HI" => Some("15"),
        "ID" => Some("16"), "IL" => Some("17"), "IN" => Some("18"),
        "IA" => Some("19"), "KS" => Some("20"), "KY" => Some("21"),
        "LA" => Some("22"), "ME" => Some("23"), "MD" => Some("24"),
        "MA" => Some("25"), "MI" => Some("26"), "MN" => Some("27"),
        "MS" => Some("28"), "MO" => Some("29"), "MT" => Some("30"),
        "NE" => Some("31"), "NV" => Some("32"), "NH" => Some("33"),
        "NJ" => Some("34"), "NM" => Some("35"), "NY" => Some("36"),
        "NC" => Some("37"), "ND" => Some("38"), "OH" => Some("39"),
        "OK" => Some("40"), "OR" => Some("41"), "PA" => Some("42"),
        "RI" => Some("44"), "SC" => Some("45"), "SD" => Some("46"),
        "TN" => Some("47"), "TX" => Some("48"), "UT" => Some("49"),
        "VT" => Some("50"), "VA" => Some("51"), "WA" => Some("53"),
        "WV" => Some("54"), "WI" => Some("55"), "WY" => Some("56"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_fips_vt() { assert_eq!(state_code_to_fips("VT"), Some("50")); }
    #[test] fn test_fips_dc() { assert_eq!(state_code_to_fips("DC"), Some("11")); }
    #[test] fn test_fips_unknown() { assert_eq!(state_code_to_fips("ZZ"), None); }

    // Hard states
    #[test] fn test_fips_va() { assert_eq!(state_code_to_fips("VA"), Some("51")); }
    #[test] fn test_fips_nv() { assert_eq!(state_code_to_fips("NV"), Some("32")); }
    #[test] fn test_fips_tx() { assert_eq!(state_code_to_fips("TX"), Some("48")); }
    #[test] fn test_fips_la() { assert_eq!(state_code_to_fips("LA"), Some("22")); }
    #[test] fn test_fips_wa() { assert_eq!(state_code_to_fips("WA"), Some("53")); }

    // Non-consecutive FIPS codes (tricky ones)
    #[test] fn test_fips_ri_is_44_not_43() { assert_eq!(state_code_to_fips("RI"), Some("44")); }
    #[test] fn test_fips_wa_is_53_not_52()  { assert_eq!(state_code_to_fips("WA"), Some("53")); }

    #[test]
    fn test_all_50_states_plus_dc_have_fips() {
        let codes = [
            "AL","AK","AZ","AR","CA","CO","CT","DE","DC","FL","GA",
            "HI","ID","IL","IN","IA","KS","KY","LA","ME","MD","MA",
            "MI","MN","MS","MO","MT","NE","NV","NH","NJ","NM","NY",
            "NC","ND","OH","OK","OR","PA","RI","SC","SD","TN","TX",
            "UT","VT","VA","WA","WV","WI","WY",
        ];
        for code in &codes {
            assert!(state_code_to_fips(code).is_some(), "missing FIPS for {code}");
            let fips = state_code_to_fips(code).unwrap();
            assert!(fips.len() == 2, "{code} FIPS must be 2 chars, got '{fips}'");
            assert!(fips.chars().all(|c| c.is_ascii_digit()), "{code} FIPS must be numeric, got '{fips}'");
        }
        // 51 entries: 50 states + DC
        assert_eq!(codes.len(), 51);
    }
}
