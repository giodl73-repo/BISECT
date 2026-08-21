"""Canonical congressional apportionment configuration by census year."""

STATE_NAMES = {
    "AK": "Alaska",
    "AL": "Alabama",
    "AR": "Arkansas",
    "AZ": "Arizona",
    "CA": "California",
    "CO": "Colorado",
    "CT": "Connecticut",
    "DE": "Delaware",
    "FL": "Florida",
    "GA": "Georgia",
    "HI": "Hawaii",
    "IA": "Iowa",
    "ID": "Idaho",
    "IL": "Illinois",
    "IN": "Indiana",
    "KS": "Kansas",
    "KY": "Kentucky",
    "LA": "Louisiana",
    "MA": "Massachusetts",
    "MD": "Maryland",
    "ME": "Maine",
    "MI": "Michigan",
    "MN": "Minnesota",
    "MO": "Missouri",
    "MS": "Mississippi",
    "MT": "Montana",
    "NC": "North Carolina",
    "ND": "North Dakota",
    "NE": "Nebraska",
    "NH": "New Hampshire",
    "NJ": "New Jersey",
    "NM": "New Mexico",
    "NV": "Nevada",
    "NY": "New York",
    "OH": "Ohio",
    "OK": "Oklahoma",
    "OR": "Oregon",
    "PA": "Pennsylvania",
    "RI": "Rhode Island",
    "SC": "South Carolina",
    "SD": "South Dakota",
    "TN": "Tennessee",
    "TX": "Texas",
    "UT": "Utah",
    "VA": "Virginia",
    "VT": "Vermont",
    "WA": "Washington",
    "WI": "Wisconsin",
    "WV": "West Virginia",
    "WY": "Wyoming",
}

_DISTRICTS_BY_YEAR = {
    "2000": (
        ("CA", 53), ("TX", 32), ("NY", 29), ("FL", 25), ("PA", 19),
        ("IL", 19), ("OH", 18), ("MI", 15), ("GA", 13), ("NJ", 13),
        ("NC", 13), ("VA", 11), ("MA", 10), ("IN", 9), ("WA", 9),
        ("TN", 9), ("MO", 9), ("WI", 8), ("MD", 8), ("AZ", 8),
        ("MN", 8), ("LA", 7), ("CO", 7), ("AL", 7), ("KY", 6),
        ("SC", 6), ("OK", 5), ("OR", 5), ("CT", 5), ("IA", 5),
        ("MS", 4), ("KS", 4), ("AR", 4), ("UT", 3), ("NV", 3),
        ("NM", 3), ("WV", 3), ("NE", 3), ("ID", 2), ("HI", 2),
        ("NH", 2), ("ME", 2), ("RI", 2), ("MT", 1), ("DE", 1),
        ("SD", 1), ("ND", 1), ("AK", 1), ("VT", 1), ("WY", 1),
    ),
    "2010": (
        ("CA", 53), ("TX", 36), ("FL", 27), ("NY", 27), ("PA", 18),
        ("IL", 18), ("OH", 16), ("GA", 14), ("NC", 13), ("MI", 14),
        ("NJ", 12), ("VA", 11), ("WA", 10), ("AZ", 9), ("MA", 9),
        ("TN", 9), ("IN", 9), ("MD", 8), ("MO", 8), ("WI", 8),
        ("MN", 8), ("CO", 7), ("SC", 7), ("AL", 7), ("LA", 6),
        ("KY", 6), ("OR", 5), ("OK", 5), ("CT", 5), ("UT", 4),
        ("IA", 4), ("NV", 4), ("AR", 4), ("MS", 4), ("KS", 4),
        ("NM", 3), ("NE", 3), ("WV", 3), ("ID", 2), ("HI", 2),
        ("NH", 2), ("ME", 2), ("RI", 2), ("MT", 1), ("DE", 1),
        ("SD", 1), ("ND", 1), ("AK", 1), ("VT", 1), ("WY", 1),
    ),
    "2020": (
        ("CA", 52), ("TX", 38), ("FL", 28), ("NY", 26), ("PA", 17),
        ("IL", 17), ("OH", 15), ("GA", 14), ("NC", 14), ("MI", 13),
        ("NJ", 12), ("VA", 11), ("WA", 10), ("AZ", 9), ("MA", 9),
        ("TN", 9), ("IN", 9), ("MD", 8), ("MO", 8), ("WI", 8),
        ("CO", 8), ("MN", 8), ("SC", 7), ("AL", 7), ("LA", 6),
        ("KY", 6), ("OR", 6), ("OK", 5), ("CT", 5), ("UT", 4),
        ("IA", 4), ("NV", 4), ("AR", 4), ("MS", 4), ("KS", 4),
        ("NM", 3), ("NE", 3), ("ID", 2), ("WV", 2), ("HI", 2),
        ("NH", 2), ("ME", 2), ("RI", 2), ("MT", 2), ("DE", 1),
        ("SD", 1), ("ND", 1), ("AK", 1), ("VT", 1), ("WY", 1),
    ),
}


def _build_state_config(year):
    config = {
        code: {"name": STATE_NAMES[code], "districts": districts}
        for code, districts in _DISTRICTS_BY_YEAR[year]
    }
    assert len(config) == 50
    assert sum(item["districts"] for item in config.values()) == 435
    return config


STATE_CONFIG_2000 = _build_state_config("2000")
STATE_CONFIG_2010 = _build_state_config("2010")
STATE_CONFIG_2020 = _build_state_config("2020")

STATE_SEATS_2000 = {code: item["districts"] for code, item in STATE_CONFIG_2000.items()}
STATE_SEATS_2010 = {code: item["districts"] for code, item in STATE_CONFIG_2010.items()}
STATE_SEATS_2020 = {code: item["districts"] for code, item in STATE_CONFIG_2020.items()}

_CONFIGS = {
    "2000": STATE_CONFIG_2000,
    "2010": STATE_CONFIG_2010,
    "2020": STATE_CONFIG_2020,
}

_SEATS = {
    "2000": STATE_SEATS_2000,
    "2010": STATE_SEATS_2010,
    "2020": STATE_SEATS_2020,
}


def _normalize_year(year):
    normalized = str(year)
    if normalized not in _CONFIGS:
        raise ValueError(
            f"Unsupported year: {year}. Must be one of: {', '.join(_CONFIGS)}"
        )
    return normalized


def get_state_config(year):
    """Return state names and congressional district counts for a census year."""
    return _CONFIGS[_normalize_year(year)]


def get_state_seats(year):
    """Return congressional district counts keyed by state code."""
    return _SEATS[_normalize_year(year)]
