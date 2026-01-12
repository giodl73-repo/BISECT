"""
2020 Census State Configuration

Congressional apportionment based on 2020 Census results.
Source: https://www.census.gov/data/tables/2020/dec/2020-apportionment-data.html

Total: 435 congressional districts across 50 states
"""

STATE_CONFIG_2020 = {
    'CA': {'name': 'California', 'districts': 52},
    'TX': {'name': 'Texas', 'districts': 38},
    'FL': {'name': 'Florida', 'districts': 28},
    'NY': {'name': 'New York', 'districts': 26},
    'PA': {'name': 'Pennsylvania', 'districts': 17},
    'IL': {'name': 'Illinois', 'districts': 17},
    'OH': {'name': 'Ohio', 'districts': 15},
    'GA': {'name': 'Georgia', 'districts': 14},
    'NC': {'name': 'North Carolina', 'districts': 14},
    'MI': {'name': 'Michigan', 'districts': 13},
    'NJ': {'name': 'New Jersey', 'districts': 12},
    'VA': {'name': 'Virginia', 'districts': 11},
    'WA': {'name': 'Washington', 'districts': 10},
    'AZ': {'name': 'Arizona', 'districts': 9},
    'MA': {'name': 'Massachusetts', 'districts': 9},
    'TN': {'name': 'Tennessee', 'districts': 9},
    'IN': {'name': 'Indiana', 'districts': 9},
    'MD': {'name': 'Maryland', 'districts': 8},
    'MO': {'name': 'Missouri', 'districts': 8},
    'WI': {'name': 'Wisconsin', 'districts': 8},
    'CO': {'name': 'Colorado', 'districts': 8},
    'MN': {'name': 'Minnesota', 'districts': 8},
    'SC': {'name': 'South Carolina', 'districts': 7},
    'AL': {'name': 'Alabama', 'districts': 7},
    'LA': {'name': 'Louisiana', 'districts': 6},
    'KY': {'name': 'Kentucky', 'districts': 6},
    'OR': {'name': 'Oregon', 'districts': 6},
    'OK': {'name': 'Oklahoma', 'districts': 5},
    'CT': {'name': 'Connecticut', 'districts': 5},
    'UT': {'name': 'Utah', 'districts': 4},
    'IA': {'name': 'Iowa', 'districts': 4},
    'NV': {'name': 'Nevada', 'districts': 4},
    'AR': {'name': 'Arkansas', 'districts': 4},
    'MS': {'name': 'Mississippi', 'districts': 4},
    'KS': {'name': 'Kansas', 'districts': 4},
    'NM': {'name': 'New Mexico', 'districts': 3},
    'NE': {'name': 'Nebraska', 'districts': 3},
    'ID': {'name': 'Idaho', 'districts': 2},
    'WV': {'name': 'West Virginia', 'districts': 2},
    'HI': {'name': 'Hawaii', 'districts': 2},
    'NH': {'name': 'New Hampshire', 'districts': 2},
    'ME': {'name': 'Maine', 'districts': 2},
    'RI': {'name': 'Rhode Island', 'districts': 2},
    'MT': {'name': 'Montana', 'districts': 2},
    'DE': {'name': 'Delaware', 'districts': 1},
    'SD': {'name': 'South Dakota', 'districts': 1},
    'ND': {'name': 'North Dakota', 'districts': 1},
    'AK': {'name': 'Alaska', 'districts': 1},
    'VT': {'name': 'Vermont', 'districts': 1},
    'WY': {'name': 'Wyoming', 'districts': 1},
}

# Verify total is 435
assert sum(config['districts'] for config in STATE_CONFIG_2020.values()) == 435, \
    "Total districts must equal 435"
