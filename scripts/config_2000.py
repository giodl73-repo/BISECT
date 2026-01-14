"""
2000 Census State Configuration

Congressional apportionment based on 2000 Census results.
Source: https://www.census.gov/data/tables/time-series/dec/apportionment-data-text.html

Total: 435 congressional districts across 50 states
Used for 107th-111th Congresses (2003-2013)
"""

STATE_CONFIG_2000 = {
    'CA': {'name': 'California', 'districts': 53},      # Same in 2010, lost 1 in 2020
    'TX': {'name': 'Texas', 'districts': 32},           # Gained 4 to 36 in 2010
    'NY': {'name': 'New York', 'districts': 29},        # Lost 2 to 27 in 2010
    'FL': {'name': 'Florida', 'districts': 25},         # Gained 2 to 27 in 2010
    'PA': {'name': 'Pennsylvania', 'districts': 19},    # Lost 1 to 18 in 2010
    'IL': {'name': 'Illinois', 'districts': 19},        # Lost 1 to 18 in 2010
    'OH': {'name': 'Ohio', 'districts': 18},            # Lost 2 to 16 in 2010
    'MI': {'name': 'Michigan', 'districts': 15},        # Lost 1 to 14 in 2010
    'GA': {'name': 'Georgia', 'districts': 13},         # Gained 1 to 14 in 2010
    'NJ': {'name': 'New Jersey', 'districts': 13},      # Lost 1 to 12 in 2010
    'NC': {'name': 'North Carolina', 'districts': 13},  # Same in 2010
    'VA': {'name': 'Virginia', 'districts': 11},        # Same in 2010
    'MA': {'name': 'Massachusetts', 'districts': 10},   # Lost 1 to 9 in 2010
    'IN': {'name': 'Indiana', 'districts': 9},          # Same in 2010
    'WA': {'name': 'Washington', 'districts': 9},       # Gained 1 to 10 in 2010
    'TN': {'name': 'Tennessee', 'districts': 9},        # Same in 2010
    'MO': {'name': 'Missouri', 'districts': 9},         # Lost 1 to 8 in 2010
    'WI': {'name': 'Wisconsin', 'districts': 8},        # Same in 2010
    'MD': {'name': 'Maryland', 'districts': 8},         # Same in 2010
    'AZ': {'name': 'Arizona', 'districts': 8},          # Gained 1 to 9 in 2010
    'MN': {'name': 'Minnesota', 'districts': 8},        # Same in 2010
    'LA': {'name': 'Louisiana', 'districts': 7},        # Lost 1 to 6 in 2010
    'CO': {'name': 'Colorado', 'districts': 7},         # Same in 2010
    'AL': {'name': 'Alabama', 'districts': 7},          # Same in 2010
    'KY': {'name': 'Kentucky', 'districts': 6},         # Same in 2010
    'SC': {'name': 'South Carolina', 'districts': 6},   # Gained 1 to 7 in 2010
    'OK': {'name': 'Oklahoma', 'districts': 5},         # Same in 2010
    'OR': {'name': 'Oregon', 'districts': 5},           # Same in 2010
    'CT': {'name': 'Connecticut', 'districts': 5},      # Same in 2010
    'IA': {'name': 'Iowa', 'districts': 5},             # Lost 1 to 4 in 2010
    'MS': {'name': 'Mississippi', 'districts': 4},      # Same in 2010
    'KS': {'name': 'Kansas', 'districts': 4},           # Same in 2010
    'AR': {'name': 'Arkansas', 'districts': 4},         # Same in 2010
    'UT': {'name': 'Utah', 'districts': 3},             # Gained 1 to 4 in 2010
    'NV': {'name': 'Nevada', 'districts': 3},           # Gained 1 to 4 in 2010
    'NM': {'name': 'New Mexico', 'districts': 3},       # Same in 2010
    'WV': {'name': 'West Virginia', 'districts': 3},    # Same in 2010
    'NE': {'name': 'Nebraska', 'districts': 3},         # Same in 2010
    'ID': {'name': 'Idaho', 'districts': 2},            # Same in 2010
    'HI': {'name': 'Hawaii', 'districts': 2},           # Same in 2010
    'NH': {'name': 'New Hampshire', 'districts': 2},    # Same in 2010
    'ME': {'name': 'Maine', 'districts': 2},            # Same in 2010
    'RI': {'name': 'Rhode Island', 'districts': 2},     # Same in 2010
    'MT': {'name': 'Montana', 'districts': 1},          # Same in 2010, gained 1 in 2020
    'DE': {'name': 'Delaware', 'districts': 1},         # Same in 2010
    'SD': {'name': 'South Dakota', 'districts': 1},     # Same in 2010
    'ND': {'name': 'North Dakota', 'districts': 1},     # Same in 2010
    'AK': {'name': 'Alaska', 'districts': 1},           # Same in 2010
    'VT': {'name': 'Vermont', 'districts': 1},          # Same in 2010
    'WY': {'name': 'Wyoming', 'districts': 1},          # Same in 2010
}

# Verify total is 435
assert sum(config['districts'] for config in STATE_CONFIG_2000.values()) == 435, \
    "Total districts must equal 435"

# Changes from 2000 to 2010:
# Gained seats: TX (+4), FL (+2), GA (+1), SC (+1), UT (+1), NV (+1), WA (+1)
# Lost seats: NY (-2), OH (-2), MA (-1), NJ (-1), PA (-1), IL (-1), MI (-1), IA (-1), LA (-1), MO (-1)
