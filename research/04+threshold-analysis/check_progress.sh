#!/bin/bash
# Quick progress checker for 50-state threshold analysis

echo "============================================"
echo "50-STATE VRA THRESHOLD ANALYSIS - PROGRESS"
echo "============================================"
echo ""

# Check if analysis is running
if pgrep -f "run_50_state_threshold_analysis.py" > /dev/null; then
    echo "Status: ✅ RUNNING"
else
    echo "Status: ⚠️  NOT RUNNING (may have completed or crashed)"
fi

echo ""
echo "Latest output (last 40 lines):"
echo "--------------------------------------------"
tail -40 analysis_run.log 2>/dev/null || echo "Log file not found"

echo ""
echo "============================================"
echo "Monitor live: tail -f analysis_run.log"
echo "============================================"
