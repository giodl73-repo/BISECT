"""Reopen a portable project, verify local dependencies and export every layout."""
import argparse
import json
from pathlib import Path
from qgis.core import QgsApplication, QgsProject, QgsLayoutExporter, Qgis
from workspace import sha, write_json


def export(package, output):
    package = package.resolve(); output.mkdir(parents=True, exist_ok=False)
    expected = json.loads((package / 'provenance/verification.json').read_text())['expected_pages']
    app = QgsApplication([], False); app.initQgis(); project = QgsProject.instance()
    assert project.read(str(package / 'project.qgs'))
    for layer in project.mapLayers().values():
        assert layer.isValid(), layer.name()
        source = Path(layer.source().split('|')[0]).resolve()
        assert source.is_relative_to(package) and source.is_file(), source
    layouts = project.layoutManager().layouts(); assert sorted(x.name() for x in layouts) == sorted(expected)
    settings = QgsLayoutExporter.PdfExportSettings(); settings.dpi = 150; settings.textRenderFormat = Qgis.TextRenderFormat.AlwaysText
    pages = []
    for name in expected:
        layout = next(x for x in layouts if x.name() == name); exporter = QgsLayoutExporter(layout)
        assert layout.pageCollection().pageCount() == 1
        pdf = output / (name + '.pdf'); png = output / (name + '.png')
        assert exporter.exportToPdf(str(pdf), settings) == QgsLayoutExporter.Success
        rendered = exporter.renderPageToImage(0, dpi=150); assert rendered.save(str(png))
        pages.append({'name': name, 'pdf_sha256': sha(pdf), 'png_sha256': sha(png)})
        print(name, flush=True)
    write_json(output / 'exports.json', {'qgis': Qgis.QGIS_VERSION, 'pages': pages})
    del exporter, layout, layouts, layer
    project.clear(); app.exitQgis()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument('--package', type=Path, required=True); parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args(); export(args.package, args.output)
