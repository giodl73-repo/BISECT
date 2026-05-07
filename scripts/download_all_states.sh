#!/bin/bash
# Download Census block data for all 50 states

STATES="AL AK AZ AR CA CO CT DE FL GA HI ID IL IN IA KS KY LA ME MD MA MI MN MS MO MT NE NV NH NJ NM NY NC ND OH OK OR PA RI SC SD TN TX UT VT VA WA WV WI WY"

for state in $STATES; do
    echo "Starting download for $state..."
    python scripts/download_data.py --state $state --year 2020 > data/raw/${state,,}_download.log 2>&1 &
    sleep 2  # Stagger starts slightly
done

echo "All 50 state downloads started in background"
echo "Check data/raw/*_download.log for progress"
