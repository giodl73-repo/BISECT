pub(crate) fn import_scope(state: &str, year: &str, label: &str) -> Result<&'static str, String> {
    const STATES: &str = "AL01 AK02 AZ04 AR05 CA06 CO08 CT09 DE10 DC11 FL12 GA13 HI15 ID16 IL17 IN18 IA19 KS20 KY21 LA22 ME23 MD24 MA25 MI26 MN27 MS28 MO29 MT30 NE31 NV32 NH33 NJ34 NM35 NY36 NC37 ND38 OH39 OK40 OR41 PA42 RI44 SC45 SD46 TN47 TX48 UT49 VT50 VA51 WA53 WV54 WI55 WY56";
    let fips = STATES
        .split_whitespace()
        .find(|s| &s[..2] == state)
        .map(|s| &s[2..])
        .ok_or("Select a supported tract-input state.")?;
    if !["2000", "2010", "2020"].contains(&year) || label.trim().is_empty() || label.len() > 200 {
        return Err("Supply a supported Census year and source label of 1–200 bytes.".into());
    }
    Ok(fips)
}

pub(crate) fn tract_id(raw: &str, fips: &str) -> Result<String, String> {
    if raw.is_empty() || raw.len() > 11 || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Supply numeric tract GEOIDs of at most eleven digits.".into());
    }
    let id = format!("{raw:0>11}");
    if !id.starts_with(fips) {
        return Err("Tract input contains a GEOID outside the selected state.".into());
    }
    Ok(id)
}
