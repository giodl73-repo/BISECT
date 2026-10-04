"""Independent output checks using SQLite, OGR and PDF text extraction."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import sqlite3
import subprocess
from osgeo import gdal, ogr
from workspace import sha, write_json
gdal.UseExceptions(); ogr.UseExceptions()


def validate(package, run=None):
    evidence = json.loads((package / 'provenance/verification.json').read_text())
    gpkg = package / 'data/publication.gpkg'
    assert sha(gpkg) == evidence['derivative_sha256'], 'Published data changed'
    with sqlite3.connect(gpkg) as db:
        assert db.execute('pragma integrity_check').fetchall() == [('ok',)]
        rows = db.execute('select GEOID20,district,population from units').fetchall()
        assert len(rows) == evidence['units'] == len({x[0] for x in rows})
        assert all(isinstance(x[0], str) and re.fullmatch('[0-9]{15}', x[0]) for x in rows)
        totals = Counter()
        for geoid, district, population in rows: totals[str(district)] += population
        assert dict(totals) == evidence['district_population']
        assert sum(totals.values()) == evidence['state_population']
        assert dict(db.execute('select id,population from districts')) == dict(totals)
        declared = db.execute("select geometry_type_name from gpkg_geometry_columns where table_name='units'").fetchone()[0]
        assert declared == 'GEOMETRY'
    if run:
        context = json.loads((run / 'inputs/context.rctx').read_text())
        assignments = json.loads((run / 'inputs/plan/baseline_assignments.json').read_text())['assignments']
        expected = {(u, assignments[u], p) for u,p in zip(context['units']['unit_ids'], context['populations'])}
        assert set(rows) == expected
        original = gdal.OpenEx('/vsizip/' + (run / 'inputs/blocks.zip').as_posix(), gdal.OF_VECTOR)
        original_layer = original.GetLayer(0)
        current = gdal.OpenEx(str(gpkg), gdal.OF_VECTOR); current_layer = current.GetLayerByName('units')
        assert original_layer.GetSpatialRef().IsSame(current_layer.GetSpatialRef())
        groups = {d: ogr.Geometry(ogr.wkbMultiPolygon) for d in set(assignments.values())}
        for f in original_layer:
            converted = current_layer.GetFeature(f.GetFID())
            assert bytes(f.GetGeometryRef().ExportToIsoWkb()) == bytes(converted.GetGeometryRef().ExportToIsoWkb())
            assert all(f[x.name] == converted[x.name] for x in original_layer.schema)
            geometry = f.GetGeometryRef()
            if ogr.GT_Flatten(geometry.GetGeometryType()) == ogr.wkbPolygon:
                groups[assignments[f['GEOID20']]].AddGeometry(geometry)
            else:
                for part in geometry: groups[assignments[f['GEOID20']]].AddGeometry(part)
        for f in current.GetLayerByName('districts'):
            expected_geometry = groups[int(f['id'])].UnionCascaded()
            assert expected_geometry.SymDifference(f.GetGeometryRef()).IsEmpty(), 'District dissolve coverage differs'
    expected_pages = evidence['expected_pages']; exports = package / 'exports'
    assert sorted(x.stem for x in exports.glob('*.pdf')) == sorted(expected_pages)
    assert sorted(x.stem for x in exports.glob('*.png')) == sorted(expected_pages)
    for name in expected_pages:
        pdf = exports / (name + '.pdf')
        info = subprocess.check_output(['pdfinfo', str(pdf)]).decode('utf-8')
        assert re.search(r'^Pages:\s+1\s*$', info, re.M), name
        text = ' '.join(subprocess.check_output(['pdftotext', '-enc', 'UTF-8', '-raw', str(pdf), '-']).decode('utf-8').split())
        for value in ['Rhode Island', 'reference baseline candidate', f'{evidence["state_population"]:,}',
                      *[f'{p:,}' for p in totals.values()], 'legal validity', 'Census Bureau']:
            assert value in text, (name, value)
    result = {'status': 'passed', 'pages': expected_pages, 'units': len(rows), 'state_population': sum(totals.values()),
              'district_population': dict(totals), 'exact_source_comparison': bool(run), 'gpkg_sha256': sha(gpkg)}
    write_json(package / 'provenance/output-checks.json', result); print(json.dumps(result), flush=True)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('--package', type=Path, required=True); p.add_argument('--run', type=Path)
    args = p.parse_args(); validate(args.package.resolve(), args.run.resolve() if args.run else None)
