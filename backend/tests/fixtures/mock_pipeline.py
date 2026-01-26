#!/usr/bin/env python3
"""
Mock pipeline script for testing.

Simulates the real pipeline by:
- Printing STATUS messages
- Accepting command-line arguments
- Exiting with configurable exit codes
"""
import sys
import time
import argparse

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--years', required=True)
    parser.add_argument('--workers', type=int, default=4)
    parser.add_argument('--version', required=True)
    parser.add_argument('--states', default=None)
    parser.add_argument('--fail', action='store_true', help='Exit with error code')
    parser.add_argument('--delay', type=float, default=0.1, help='Delay in seconds')

    args = parser.parse_args()

    # Print some STATUS messages
    years = args.years.split(',')

    for year in years:
        # Year started
        print(f"STATUS:YEAR:{year}:STARTED:0/50", flush=True)
        time.sleep(args.delay)

        # Worker update
        print(f"STATUS:WORKER:{year}:1:STATE:5/50:california:STAGE:1/7:redistricting", flush=True)
        time.sleep(args.delay)

        # Year complete
        print(f"STATUS:YEAR:{year}:COMPLETE:50/50", flush=True)
        time.sleep(args.delay)

    # Nation processing
    print(f"STATUS:NATION:{years[0]}:POSTPROCESS:5/9", flush=True)
    time.sleep(args.delay)

    if args.fail:
        print("ERROR: Mock pipeline failure", file=sys.stderr, flush=True)
        sys.exit(1)

    print("Pipeline complete!", flush=True)
    sys.exit(0)

if __name__ == '__main__':
    main()
