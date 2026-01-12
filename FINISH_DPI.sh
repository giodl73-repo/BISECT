#!/bin/bash
# Script to finish DPI parameter implementation

echo "Adding --dpi to add_cities_to_districts.py..."
sed -i "/parser.add_argument('--debug'/i\    parser.add_argument('--dpi', type=int, default=150, help='DPI for output maps')" scripts/pipeline/add_cities_to_districts.py

echo "Adding --dpi to visualize_all_rounds.py..."
sed -i "/parser.add_argument('--debug'/i\    parser.add_argument('--dpi', type=int, default=150, help='DPI for output maps')" scripts/pipeline/visualize_all_rounds.py

echo "Adding --dpi to create_individual_district_maps.py..."
sed -i "/parser.add_argument('--debug'/i\    parser.add_argument('--dpi', type=int, default=150, help='DPI for output maps')" scripts/pipeline/create_individual_district_maps.py

echo "Done! Now scripts accept --dpi parameter (defaults to 150)"
