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
}
