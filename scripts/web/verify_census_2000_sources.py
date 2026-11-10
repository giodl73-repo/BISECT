"""Verify the 50 state 2000 TIGER/PL94 entries against Census directories.

GETs only directory listings unless --download-state is explicitly supplied.
Retains listing bytes, URL/hash evidence and optional original ZIPs in target.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import time
from urllib.error import HTTPError
from urllib.parse import urljoin
from urllib.request import Request, urlopen

parser = argparse.ArgumentParser()
parser.add_argument('--download-state', choices=['VT', 'WY', 'DE', 'RI'])
parser.add_argument('--offline', action='store_true')
args = parser.parse_args()
root = Path('target/census-2000-sources')
root.mkdir(parents=True, exist_ok=True)
manifest = json.loads(Path('data/manifest.json').read_text(encoding='utf-8'))
BASE = 'https://www2.census.gov/'
TRACT = BASE + 'geo/tiger/TIGER2010/TRACT/2000/'
PL = BASE + 'census_2000/datasets/redistricting_file--pl_94-171/'


class Links(HTMLParser):
    def __init__(self, text):
        super().__init__()
        self.links = set()
        self.feed(text)

    def handle_starttag(self, tag, attrs):
        if tag == 'a':
            self.links.update(v for k, v in attrs if k == 'href' and v)


def obtain(url, path, limit):
    if path.exists():
        raw = path.read_bytes()
    else:
        if args.offline:
            raise RuntimeError('Missing retained source: ' + str(path))
        for attempt in range(3):
            try:
                with urlopen(Request(url, headers={'User-Agent': 'bisect-source-verification/1'}), timeout=30) as response:
                    if response.geturl() != url:
                        raise RuntimeError('Unexpected source redirect: ' + response.geturl())
                    raw = response.read(limit + 1)
                break
            except HTTPError as error:
                if error.code != 429 or attempt == 2:
                    raise
                print('Census rate limit; retrying this listing after a pause.', flush=True)
                delay = error.headers.get('Retry-After', '')
                time.sleep(min(60, int(delay)) if delay.isdigit() else 20 * (attempt + 1))
        assert 0 < len(raw) <= limit
        temp = path.with_suffix(path.suffix + '.tmp')
        temp.write_bytes(raw)
        temp.replace(path)
    assert 0 < len(raw) <= limit
    return raw


tract = obtain(TRACT, root / 'tract-directory.html', 4 * 1024 * 1024)
tract_links = Links(tract.decode('utf-8')).links


def verify(entry):
    code, state = entry
    filename = f'tl_2010_{int(state["fips"]):02}_tract00.zip'
    assert filename in tract_links, (code, filename)
    directory = PL + state['name'].replace(' ', '_') + '/'
    listing = obtain(directory, root / (code + '-pl-directory.html'), 2 * 1024 * 1024)
    geography = code.lower() + 'geo.upl.zip'
    assert geography in Links(listing.decode('utf-8')).links, (code, geography)
    return code, {'tiger': urljoin(TRACT, filename), 'pl94171': urljoin(directory, geography), 'listing_url': directory, 'listing_sha256': hashlib.sha256(listing).hexdigest()}


with ThreadPoolExecutor(max_workers=1) as pool:
    verified = dict(pool.map(verify, sorted(manifest['states'].items())))
assert len(verified) == 50
evidence = {'states': verified, 'tract_directory_url': TRACT, 'tract_directory_sha256': hashlib.sha256(tract).hexdigest(), 'state_count': 50, 'scope': 'Directory link verification; original-data parsing and execution require separate tests.'}
if args.download_state:
    state = args.download_state
    directory = root / state
    directory.mkdir(exist_ok=True)
    evidence['original_archives'] = []
    for kind in ['tiger', 'pl94171']:
        url = verified[state][kind]
        raw = obtain(url, directory / (kind + '.zip'), 64 * 1024 * 1024)
        evidence['original_archives'].append({'state': state, 'year': '2000', 'kind': kind, 'url': url, 'byte_length': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
(root / 'verified.json').write_text(json.dumps(evidence, indent=2) + '\n', encoding='utf-8')
print('PASS verified Census 2000 directory links for all 50 state tract/geography pairs' + ('; retained original ' + args.download_state + ' ZIPs' if args.download_state else ''), flush=True)
