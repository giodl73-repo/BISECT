"""Build the RI baseline publication from staged inputs using QGIS's Python."""
import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import platform
import shutil
import sqlite3
import sys
import zipfile
from osgeo import gdal, ogr
from qgis.core import *
from qgis.PyQt.QtCore import QVariant
from qgis.PyQt.QtGui import QColor, QFont
from workspace import sha, write_json, relative

gdal.UseExceptions(); ogr.UseExceptions()
COLORS = {1: '#368395', 2: '#d18a47'}


def inputs(run):
    selection = json.loads((run / 'selection.json').read_text())
    for item in selection['files']:
        path = run / relative(item['path'])
        if path.stat().st_size != item['bytes'] or sha(path) != item['sha256']:
            raise ValueError(f'Input changed: {item["path"]}')
    context = json.loads((run / 'inputs/context.rctx').read_text())
    manifest = json.loads((run / 'inputs/plan/baseline_manifest.json').read_text())
    assignments = json.loads((run / 'inputs/plan/baseline_assignments.json').read_text())['assignments']
    assert manifest['source_context_sha256'] == sha(run / 'inputs/context.rctx')
    assert context['units']['state'] == 'RI' and context['units']['year'] == 2020
    assert set(assignments) == set(context['units']['unit_ids'])
    assert len(context['populations']) == len(context['units']['unit_ids'])
    for item in manifest['artifacts']:
        assert sha(run / 'inputs/plan' / relative(item['path'])) == item['sha256']
    with zipfile.ZipFile(run / 'inputs/blocks.zip') as z:
        for ext in ('shp', 'dbf', 'shx'):
            member = next(x for x in z.namelist() if x.endswith('.' + ext))
            assert hashlib.sha256(z.read(member)).hexdigest() == context['source_hashes']['tiger_block_' + ext].removeprefix('sha256:')
    return selection, context, manifest, assignments


def make_data(run, output, context, assignments):
    source = gdal.OpenEx('/vsizip/' + (run / 'inputs/blocks.zip').as_posix(), gdal.OF_VECTOR)
    original = source.GetLayer(0)
    target = gdal.GetDriverByName('GPKG').Create(str(output), 0, 0, 0, gdal.GDT_Unknown)
    # TIGER shapefiles can contain Polygon and MultiPolygon records. GEOMETRY
    # permits both without coercing or changing their exact source WKB.
    units = target.CreateLayer('units', original.GetSpatialRef(), ogr.wkbUnknown, options=['FID=source_fid'])
    for field in original.schema: units.CreateField(field)
    units.StartTransaction()
    for source_feature in original:
        copied_feature = ogr.Feature(units.GetLayerDefn()); copied_feature.SetFrom(source_feature)
        copied_feature.SetFID(source_feature.GetFID()); units.CreateFeature(copied_feature)
    units.CommitTransaction(); copied_feature = source_feature = None
    units.CreateField(ogr.FieldDefn('district', ogr.OFTInteger))
    units.CreateField(ogr.FieldDefn('population', ogr.OFTInteger64))
    populations = dict(zip(context['units']['unit_ids'], context['populations']))
    seen = set(); totals = defaultdict(int); counts = defaultdict(int)
    units.StartTransaction()
    for feature in units:
        geoid = feature['GEOID20']; assert geoid not in seen and geoid in assignments
        assert len(geoid) == 15 and geoid.isdigit(); seen.add(geoid)
        feature['district'] = assignments[geoid]; feature['population'] = populations[geoid]
        units.SetFeature(feature)
        totals[assignments[geoid]] += populations[geoid]; counts[assignments[geoid]] += 1
    units.CommitTransaction(); assert seen == set(assignments)
    # Verify source FIDs, every source field and exact geometry after persistence.
    feature = units = target = None
    reopened = gdal.OpenEx(str(output), gdal.OF_VECTOR); copied = reopened.GetLayerByName('units')
    assert copied.GetSpatialRef().IsSame(original.GetSpatialRef())
    original.ResetReading()
    geometry_hash = hashlib.sha256()
    for feature in original:
        actual = copied.GetFeature(feature.GetFID()); assert actual is not None
        for field in original.schema: assert actual[field.name] == feature[field.name], field.name
        left = bytes(feature.GetGeometryRef().ExportToIsoWkb()); right = bytes(actual.GetGeometryRef().ExportToIsoWkb())
        assert left == right; geometry_hash.update(left)
    crs = copied.GetSpatialRef().ExportToWkt()
    actual = feature = copied = reopened = original = source = None
    return {'units': len(seen), 'state_population': sum(totals.values()), 'district_population': dict(totals),
            'district_blocks': dict(counts), 'source_geometry_sha256': geometry_hash.hexdigest(), 'source_crs_wkt': crs}


def derived_layer(project, gpkg, units, field, name, totals):
    groups = defaultdict(list)
    for f in units.getFeatures(): groups[f[field] if field == 'district' else f['GEOID20'][:5]].append(f.geometry())
    memory = QgsVectorLayer('MultiPolygon?crs=' + units.crs().authid(), name, 'memory')
    provider = memory.dataProvider()
    provider.addAttributes([QgsField('id', QVariant.String), QgsField('population', QVariant.LongLong)])
    memory.updateFields()
    for key, geometries in sorted(groups.items()):
        geometry = QgsGeometry.unaryUnion(geometries)
        if geometry.isNull() or geometry.isEmpty() or not geometry.isGeosValid(): raise ValueError(f'Invalid derived geometry {key}')
        geometry.convertToMultiType()
        f = QgsFeature(memory.fields()); f.setAttributes([str(key), totals.get(key, 0)]); f.setGeometry(geometry); provider.addFeatures([f])
    options = QgsVectorFileWriter.SaveVectorOptions(); options.driverName = 'GPKG'; options.layerName = name
    options.actionOnExistingFile = QgsVectorFileWriter.CreateOrOverwriteLayer
    result = QgsVectorFileWriter.writeAsVectorFormatV3(memory, str(gpkg), project.transformContext(), options)
    assert result[0] == QgsVectorFileWriter.NoError, result
    return QgsVectorLayer(str(gpkg) + '|layername=' + name, name.title(), 'ogr')


def text(layout, value, x, y, w, h, size=10, bold=False, color='#203541'):
    item = QgsLayoutItemLabel(layout); layout.addLayoutItem(item); item.setText(value)
    fmt = QgsTextFormat(); font = QFont('Arial', size); font.setBold(bold)
    fmt.setFont(font); fmt.setSize(size); fmt.setColor(QColor(color)); item.setTextFormat(fmt)
    item.attemptMove(QgsLayoutPoint(x, y)); item.attemptResize(QgsLayoutSize(w, h)); return item


def map_item(layout, project, layers, extent, x, y, w, h):
    item = QgsLayoutItemMap(layout); layout.addLayoutItem(item)
    item.attemptMove(QgsLayoutPoint(x, y)); item.attemptResize(QgsLayoutSize(w, h))
    item.setCrs(QgsCoordinateReferenceSystem('EPSG:26919')); item.setLayers(layers); item.setKeepLayerSet(True)
    transform = QgsCoordinateTransform(layers[-1].crs(), item.crs(), project)
    box = transform.transformBoundingBox(extent); box.scale(1.08); item.zoomToExtent(box)
    item.setBackgroundColor(QColor('#eef3f5')); item.setFrameEnabled(True); item.setFrameStrokeColor(QColor('#b4c2c8'))
    return item


def build(run, out):
    selection, context, manifest, assignments = inputs(run)
    out.mkdir(parents=True, exist_ok=False); (out / 'data').mkdir(); (out / 'provenance').mkdir()
    gpkg = out / 'data/publication.gpkg'; evidence = make_data(run, gpkg, context, assignments)
    app = QgsApplication([], False); app.initQgis(); project = QgsProject.instance()
    project.setFilePathStorage(Qgis.FilePathType.Relative); project.setCrs(QgsCoordinateReferenceSystem('EPSG:26919'))
    units = QgsVectorLayer(str(gpkg) + '|layername=units', 'Census blocks — preserved geometry and assignments', 'ogr')
    totals = evidence['district_population']; ideal = evidence['state_population'] / len(totals)
    districts = derived_layer(project, gpkg, units, 'district', 'districts', totals)
    counties = derived_layer(project, gpkg, units, 'county', 'counties', {})
    water = QgsVectorLayer(str(gpkg) + '|layername=units', 'Water-only Census blocks', 'ogr'); water.setSubsetString('"ALAND20" = 0')
    water.renderer().setSymbol(QgsFillSymbol.createSimple({'color': '#dce9ef', 'outline_style': 'no'}))
    cats = [QgsRendererCategory(str(d), QgsFillSymbol.createSimple({'color': color, 'outline_color': '#ffffff', 'outline_width': '.45'}), f'District {d}') for d, color in COLORS.items()]
    districts.setRenderer(QgsCategorizedSymbolRenderer('id', cats))
    counties.renderer().setSymbol(QgsFillSymbol.createSimple({'style': 'no', 'outline_color': '#445765', 'outline_width': '.22', 'outline_style': 'dash'}))
    for layer in [units, districts, water, counties]: assert layer.isValid(); project.addMapLayer(layer)
    project.layerTreeRoot().findLayer(units.id()).setItemVisibilityChecked(False)
    # Detail pages highlight the selected district while retaining geographic context.
    pages = [('overview', None)] + [(f'district-{d}', d) for d in sorted(totals)]
    for page, selected in pages:
        layout = QgsPrintLayout(project); layout.initializeDefaults(); layout.setName(page)
        layout.pageCollection().pages()[0].setPageSize(QgsLayoutSize(279.4, 215.9)); project.layoutManager().addLayout(layout)
        title = 'Rhode Island' if selected is None else f'Rhode Island · District {selected}'
        text(layout, title, 9, 5, 261, 13, 25, True)
        text(layout, 'BISECT  /  2020 Census  /  reference baseline candidate', 9, 20, 261, 8, 11)
        extent = districts.extent()
        display = districts
        if selected is not None:
            display = districts.clone(); display.setName(f'District {selected} detail styling'); project.addMapLayer(display)
            detailcats = [QgsRendererCategory(str(d), QgsFillSymbol.createSimple({'color': COLORS[d] if d == selected else '#d6dcdf', 'outline_color': '#ffffff', 'outline_width': '.45'}), f'District {d}') for d in COLORS]
            display.setRenderer(QgsCategorizedSymbolRenderer('id', detailcats))
            extent = next(f.geometry().boundingBox() for f in districts.getFeatures() if f['id'] == str(selected))
            project.layerTreeRoot().findLayer(display.id()).setItemVisibilityChecked(False)
        frame = map_item(layout, project, [counties, water, display], extent, 9, 33, 164, 164)
        text(layout, 'N ↑', 160, 36, 10, 8, 10, True)
        text(layout, 'PLAN SNAPSHOT', 181, 34, 89, 8, 12, True)
        text(layout, f'{evidence["units"]:,} Census blocks\n{evidence["state_population"]:,} residents\n2 proposed districts', 181, 45, 88, 23, 11)
        for n, d in enumerate(sorted(totals)):
            deviation = (totals[d] - ideal) / ideal * 100
            text(layout, f'District {d}', 181, 75+n*28, 88, 7, 12, True, COLORS[d])
            text(layout, f'{totals[d]:,} residents  ({deviation:+.3f}%)\n{evidence["district_blocks"][d]:,} blocks', 181, 83+n*28, 88, 15, 10)
        text(layout, 'Percentages: deviation from equal population.\nDashed lines: county boundaries.\nBlue-gray: water-only Census blocks.', 181, 134, 89, 20, 8)
        text(layout, 'RESEARCH CANDIDATE', 181, 161, 89, 7, 11, True)
        text(layout, 'Source package verification: pass.\nThis map does not establish legal validity,\nVRA compliance, partisan fairness,\nglobal optimality or official adoption.', 181, 170, 89, 26, 8)
        text(layout, 'U.S. Census Bureau: 2020 TIGER/Line + PL 94-171 · BISECT by Gio Della-Libera · Publication pilot', 9, 200, 261, 6, 8)
        text(layout, f'{page}  |  Source context SHA-256: {manifest["source_context_sha256"][:20]}…  |  Full provenance included', 9, 207, 261, 5, 7)
    project.viewSettings().setDefaultViewExtent(QgsReferencedRectangle(districts.extent(), districts.crs()))
    assert project.write(str(out / 'project.qgs'))
    public_selection = dict(selection); public_selection['files'] = [{k:v for k,v in item.items() if k != 'source'} for item in selection['files']]
    write_json(out / 'provenance/source-manifest.json', public_selection)
    evidence.update({'schema': 'bisect-publication-verification-v1', 'status': 'data-verified', 'plan_status': manifest['status'],
                     'source_verification_status': manifest['verification_status'], 'context_hash': context['context_hash'],
                     'context_file_sha256': sha(run / 'inputs/context.rctx'), 'expected_pages': [x[0] for x in pages],
                     'population_definition': '2020 Census PL 94-171 total resident population', 'derivative_sha256': sha(gpkg)})
    write_json(out / 'provenance/verification.json', evidence)
    write_json(out / 'environment.json', {'qgis': Qgis.QGIS_VERSION, 'gdal': gdal.VersionInfo('--version'),
               'python': sys.version, 'platform': platform.platform(), 'display_crs': 'EPSG:26919', 'network_required': False})
    print(json.dumps(evidence, indent=2), flush=True)
    del layer, display, districts, counties, water, units, frame, layout
    project.clear(); app.exitQgis()


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('--run', type=Path, required=True); p.add_argument('--output', type=Path, required=True)
    args = p.parse_args(); build(args.run.resolve(), args.output.resolve())
