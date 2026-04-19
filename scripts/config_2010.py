"""
2010 Census State Configuration

Congressional apportionment based on 2010 Census results.
Source: https://www.census.gov/data/tables/time-series/dec/apportionment-data-text.html

Total: 435 congressional districts across 50 states
"""

STATE_CONFIG_2010 = {
    'CA': {'name': 'California', 'districts': 53},      # Lost 1 seat in 2020
    'TX': {'name': 'Texas', 'districts': 36},           # Gained 2 seats by 2020
    'FL': {'name': 'Florida', 'districts': 27},         # Gained 1 seat by 2020
    'NY': {'name': 'New York', 'districts': 27},        # Lost 1 seat by 2020
    'PA': {'name': 'Pennsylvania', 'districts': 18},    # Lost 1 seat by 2020
    'IL': {'name': 'Illinois', 'districts': 18},        # Lost 1 seat by 2020
    'OH': {'name': 'Ohio', 'districts': 16},            # Lost 1 seat by 2020
    'GA': {'name': 'Georgia', 'districts': 14},         # Same in 2020
    'NC': {'name': 'North Carolina', 'districts': 13},  # Gained 1 seat by 2020
    'MI': {'name': 'Michigan', 'districts': 14},        # Lost 1 seat by 2020
    'NJ': {'name': 'New Jersey', 'districts': 12},      # Same in 2020
    'VA': {'name': 'Virginia', 'districts': 11},        # Same in 2020
    'WA': {'name': 'Washington', 'districts': 10},      # Same in 2020
    'AZ': {'name': 'Arizona', 'districts': 9},          # Same in 2020
    'MA': {'name': 'Massachusetts', 'districts': 9},    # Same in 2020
    'TN': {'name': 'Tennessee', 'districts': 9},        # Same in 2020
    'IN': {'name': 'Indiana', 'districts': 9},          # Same in 2020
    'MD': {'name': 'Maryland', 'districts': 8},         # Same in 2020
    'MO': {'name': 'Missouri', 'districts': 8},         # Same in 2020
    'WI': {'name': 'Wisconsin', 'districts': 8},        # Same in 2020
    'MN': {'name': 'Minnesota', 'districts': 8},        # Same in 2020
    'CO': {'name': 'Colorado', 'districts': 7},         # Gained 1 seat by 2020
    'SC': {'name': 'South Carolina', 'districts': 7},   # Same in 2020
    'AL': {'name': 'Alabama', 'districts': 7},          # Same in 2020
    'LA': {'name': 'Louisiana', 'districts': 6},        # Same in 2020
    'KY': {'name': 'Kentucky', 'districts': 6},         # Same in 2020
    'OR': {'name': 'Oregon', 'districts': 5},           # Gained 1 seat by 2020
    'OK': {'name': 'Oklahoma', 'districts': 5},         # Same in 2020
    'CT': {'name': 'Connecticut', 'districts': 5},      # Same in 2020
    'UT': {'name': 'Utah', 'districts': 4},             # Same in 2020
    'IA': {'name': 'Iowa', 'districts': 4},             # Same in 2020
    'NV': {'name': 'Nevada', 'districts': 4},           # Same in 2020
    'AR': {'name': 'Arkansas', 'districts': 4},         # Same in 2020
    'MS': {'name': 'Mississippi', 'districts': 4},      # Same in 2020
    'KS': {'name': 'Kansas', 'districts': 4},           # Same in 2020
    'NM': {'name': 'New Mexico', 'districts': 3},       # Same in 2020
    'NE': {'name': 'Nebraska', 'districts': 3},         # Same in 2020
    'WV': {'name': 'West Virginia', 'districts': 3},    # Lost 1 seat by 2020
    'ID': {'name': 'Idaho', 'districts': 2},            # Same in 2020
    'HI': {'name': 'Hawaii', 'districts': 2},           # Same in 2020
    'NH': {'name': 'New Hampshire', 'districts': 2},    # Same in 2020
    'ME': {'name': 'Maine', 'districts': 2},            # Same in 2020
    'RI': {'name': 'Rhode Island', 'districts': 2},     # Same in 2020
    'MT': {'name': 'Montana', 'districts': 1},          # Gained 1 seat by 2020
    'DE': {'name': 'Delaware', 'districts': 1},         # Same in 2020
    'SD': {'name': 'South Dakota', 'districts': 1},     # Same in 2020
    'ND': {'name': 'North Dakota', 'districts': 1},     # Same in 2020
    'AK': {'name': 'Alaska', 'districts': 1},           # Same in 2020
    'VT': {'name': 'Vermont', 'districts': 1},          # Same in 2020
    'WY': {'name': 'Wyoming', 'districts': 1},          # Same in 2020
}

# Verify total is 435
assert sum(config['districts'] for config in STATE_CONFIG_2010.values()) == 435, \
    "Total districts must equal 435"

# Key differences from 2020:
# Gained seats by 2020: TX (+2), FL (+1), NC (+1), CO (+1), MT (+1), OR (+1)
# Lost seats by 2020: CA (-1), NY (-1), IL (-1), OH (-1), PA (-1), MI (-1), WV (-1)
