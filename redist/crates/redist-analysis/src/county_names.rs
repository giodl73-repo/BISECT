/// Static county FIPS → display name lookup.
///
/// Covers high-priority states: WA, VA (independent cities), LA (parishes),
/// TX, NY, CA, FL top counties, and DC.
pub fn county_name(fips: &str) -> Option<&'static str> {
    match fips {
        // ---------------------------------------------------------------
        // Washington (53) — all 39 counties
        // ---------------------------------------------------------------
        "53001" => Some("Adams County"),
        "53003" => Some("Asotin County"),
        "53005" => Some("Benton County"),
        "53007" => Some("Chelan County"),
        "53009" => Some("Clallam County"),
        "53011" => Some("Clark County"),
        "53013" => Some("Columbia County"),
        "53015" => Some("Cowlitz County"),
        "53017" => Some("Douglas County"),
        "53019" => Some("Ferry County"),
        "53021" => Some("Franklin County"),
        "53023" => Some("Garfield County"),
        "53025" => Some("Grant County"),
        "53027" => Some("Grays Harbor County"),
        "53029" => Some("Island County"),
        "53031" => Some("Jefferson County"),
        "53033" => Some("King County"),
        "53035" => Some("Kitsap County"),
        "53037" => Some("Kittitas County"),
        "53039" => Some("Klickitat County"),
        "53041" => Some("Lewis County"),
        "53043" => Some("Lincoln County"),
        "53045" => Some("Mason County"),
        "53047" => Some("Okanogan County"),
        "53049" => Some("Pacific County"),
        "53051" => Some("Pend Oreille County"),
        "53053" => Some("Pierce County"),
        "53055" => Some("San Juan County"),
        "53057" => Some("Skagit County"),
        "53059" => Some("Skamania County"),
        "53061" => Some("Snohomish County"),
        "53063" => Some("Spokane County"),
        "53065" => Some("Stevens County"),
        "53067" => Some("Thurston County"),
        "53069" => Some("Wahkiakum County"),
        "53071" => Some("Walla Walla County"),
        "53073" => Some("Whatcom County"),
        "53075" => Some("Whitman County"),
        "53077" => Some("Yakima County"),

        // ---------------------------------------------------------------
        // Virginia independent cities (51510–51840)
        // ---------------------------------------------------------------
        "51510" => Some("Alexandria city"),
        "51515" => Some("Bedford city"),
        "51520" => Some("Bristol city"),
        "51530" => Some("Buena Vista city"),
        "51540" => Some("Charlottesville city"),
        "51550" => Some("Chesapeake city"),
        "51570" => Some("Colonial Heights city"),
        "51580" => Some("Covington city"),
        "51590" => Some("Danville city"),
        "51595" => Some("Emporia city"),
        "51600" => Some("Fairfax city"),
        "51610" => Some("Falls Church city"),
        "51620" => Some("Franklin city"),
        "51630" => Some("Fredericksburg city"),
        "51640" => Some("Galax city"),
        "51650" => Some("Hampton city"),
        "51660" => Some("Harrisonburg city"),
        "51670" => Some("Hopewell city"),
        "51678" => Some("Lexington city"),
        "51680" => Some("Lynchburg city"),
        "51683" => Some("Manassas city"),
        "51685" => Some("Manassas Park city"),
        "51690" => Some("Martinsville city"),
        "51700" => Some("Newport News city"),
        "51710" => Some("Norfolk city"),
        "51720" => Some("Norton city"),
        "51730" => Some("Petersburg city"),
        "51735" => Some("Poquoson city"),
        "51740" => Some("Portsmouth city"),
        "51750" => Some("Radford city"),
        "51760" => Some("Richmond city"),
        "51770" => Some("Roanoke city"),
        "51775" => Some("Salem city"),
        "51790" => Some("Staunton city"),
        "51800" => Some("Suffolk city"),
        "51810" => Some("Virginia Beach city"),
        "51820" => Some("Waynesboro city"),
        "51830" => Some("Williamsburg city"),
        "51840" => Some("Winchester city"),

        // ---------------------------------------------------------------
        // Louisiana parishes (22) — all 64 parishes
        // ---------------------------------------------------------------
        "22001" => Some("Acadia Parish"),
        "22003" => Some("Allen Parish"),
        "22005" => Some("Ascension Parish"),
        "22007" => Some("Assumption Parish"),
        "22009" => Some("Avoyelles Parish"),
        "22011" => Some("Beauregard Parish"),
        "22013" => Some("Bienville Parish"),
        "22015" => Some("Bossier Parish"),
        "22017" => Some("Caddo Parish"),
        "22019" => Some("Calcasieu Parish"),
        "22021" => Some("Caldwell Parish"),
        "22023" => Some("Cameron Parish"),
        "22025" => Some("Catahoula Parish"),
        "22027" => Some("Claiborne Parish"),
        "22029" => Some("Concordia Parish"),
        "22031" => Some("De Soto Parish"),
        "22033" => Some("East Baton Rouge Parish"),
        "22035" => Some("East Carroll Parish"),
        "22037" => Some("East Feliciana Parish"),
        "22039" => Some("Evangeline Parish"),
        "22041" => Some("Franklin Parish"),
        "22043" => Some("Grant Parish"),
        "22045" => Some("Iberia Parish"),
        "22047" => Some("Iberville Parish"),
        "22049" => Some("Jackson Parish"),
        "22051" => Some("Jefferson Parish"),
        "22053" => Some("Jefferson Davis Parish"),
        "22055" => Some("Lafayette Parish"),
        "22057" => Some("Lafourche Parish"),
        "22059" => Some("LaSalle Parish"),
        "22061" => Some("Lincoln Parish"),
        "22063" => Some("Livingston Parish"),
        "22065" => Some("Madison Parish"),
        "22067" => Some("Morehouse Parish"),
        "22069" => Some("Natchitoches Parish"),
        "22071" => Some("Orleans Parish"),
        "22073" => Some("Ouachita Parish"),
        "22075" => Some("Plaquemines Parish"),
        "22077" => Some("Pointe Coupee Parish"),
        "22079" => Some("Rapides Parish"),
        "22081" => Some("Red River Parish"),
        "22083" => Some("Richland Parish"),
        "22085" => Some("Sabine Parish"),
        "22087" => Some("Saint Bernard Parish"),
        "22089" => Some("Saint Charles Parish"),
        "22091" => Some("Saint Helena Parish"),
        "22093" => Some("Saint James Parish"),
        "22095" => Some("Saint John the Baptist Parish"),
        "22097" => Some("Saint Landry Parish"),
        "22099" => Some("Saint Martin Parish"),
        "22101" => Some("Saint Mary Parish"),
        "22103" => Some("Saint Tammany Parish"),
        "22105" => Some("Tangipahoa Parish"),
        "22107" => Some("Tensas Parish"),
        "22109" => Some("Terrebonne Parish"),
        "22111" => Some("Union Parish"),
        "22113" => Some("Vermilion Parish"),
        "22115" => Some("Vernon Parish"),
        "22117" => Some("Washington Parish"),
        "22119" => Some("Webster Parish"),
        "22121" => Some("West Baton Rouge Parish"),
        "22123" => Some("West Carroll Parish"),
        "22125" => Some("West Feliciana Parish"),
        "22127" => Some("Winn Parish"),

        // ---------------------------------------------------------------
        // Texas (48) — top counties
        // ---------------------------------------------------------------
        "48001" => Some("Anderson County"),
        "48003" => Some("Andrews County"),
        "48113" => Some("Dallas County"),
        "48121" => Some("Denton County"),
        "48141" => Some("El Paso County"),
        "48201" => Some("Harris County"),
        "48215" => Some("Hidalgo County"),
        "48245" => Some("Jefferson County"),
        "48251" => Some("Johnson County"),
        "48257" => Some("Kaufman County"),
        "48291" => Some("Liberty County"),
        "48309" => Some("McLennan County"),
        "48329" => Some("Midland County"),
        "48339" => Some("Montgomery County"),
        "48355" => Some("Nueces County"),
        "48375" => Some("Potter County"),
        "48391" => Some("Randall County"),
        "48439" => Some("Tarrant County"),
        "48453" => Some("Travis County"),
        "48479" => Some("Webb County"),
        "48491" => Some("Williamson County"),

        // ---------------------------------------------------------------
        // New York (36) — top counties
        // ---------------------------------------------------------------
        "36005" => Some("Bronx County"),
        "36047" => Some("Kings County"),
        "36049" => Some("Lewis County"),
        "36055" => Some("Monroe County"),
        "36059" => Some("Nassau County"),
        "36061" => Some("New York County"),
        "36067" => Some("Onondaga County"),
        "36081" => Some("Queens County"),
        "36085" => Some("Richmond County"),
        "36103" => Some("Suffolk County"),

        // ---------------------------------------------------------------
        // California (06) — top counties
        // ---------------------------------------------------------------
        "06001" => Some("Alameda County"),
        "06013" => Some("Contra Costa County"),
        "06019" => Some("Fresno County"),
        "06037" => Some("Los Angeles County"),
        "06059" => Some("Orange County"),
        "06065" => Some("Riverside County"),
        "06067" => Some("Sacramento County"),
        "06071" => Some("San Bernardino County"),
        "06073" => Some("San Diego County"),
        "06075" => Some("San Francisco County"),
        "06077" => Some("San Joaquin County"),
        "06085" => Some("Santa Clara County"),
        "06111" => Some("Ventura County"),

        // ---------------------------------------------------------------
        // Florida (12) — top counties
        // ---------------------------------------------------------------
        "12011" => Some("Broward County"),
        "12021" => Some("Collier County"),
        "12031" => Some("Duval County"),
        "12057" => Some("Hillsborough County"),
        "12069" => Some("Lake County"),
        "12083" => Some("Marion County"),
        "12086" => Some("Miami-Dade County"),
        "12095" => Some("Orange County"),
        "12099" => Some("Palm Beach County"),
        "12103" => Some("Pinellas County"),
        "12105" => Some("Polk County"),
        "12117" => Some("Seminole County"),

        // ---------------------------------------------------------------
        // District of Columbia
        // ---------------------------------------------------------------
        "11001" => Some("District of Columbia"),

        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_king_county_wa() {
        assert_eq!(county_name("53033"), Some("King County"));
    }

    #[test]
    fn test_richmond_city_va() {
        assert_eq!(county_name("51760"), Some("Richmond city"));
    }

    #[test]
    fn test_orleans_parish_la() {
        assert_eq!(county_name("22071"), Some("Orleans Parish"));
    }

    #[test]
    fn test_unknown_fips() {
        assert_eq!(county_name("99999"), None);
    }
}
