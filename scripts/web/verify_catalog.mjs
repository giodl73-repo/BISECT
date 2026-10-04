import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';

const root = path.resolve(process.argv[2] || 'dist/catalog');
const manifest = JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
if (manifest.schema_version !== 1) throw new Error('Unsupported catalog schema.');
let bytes = 0;
for (const [name,hash] of Object.entries(manifest.assets)) {
  if (!/^assets\/[a-f0-9]{64}\.json$/.test(name) || name !== `assets/${hash}.json`) throw new Error('Unsafe asset reference.');
  const data = await fs.readFile(path.join(root,name)); bytes += data.length;
  if (crypto.createHash('sha256').update(data).digest('hex') !== hash) throw new Error(`Corrupted asset: ${name}`);
}
for (const name of ['index.html','lab.css','lab.js','static.js','catalog.json']) bytes += (await fs.stat(path.join(root,name))).size;
if (bytes > 900_000_000) throw new Error('Catalog exceeds publication size budget.');
for (const run of manifest.runs) for (const state of run.states) if (state.metrics) {
  for (const name of [state.geometry_ref,state.assignments_ref]) if (!Object.hasOwn(manifest.assets,name)) throw new Error('Undeclared asset reference.');
  const geometry = JSON.parse(await fs.readFile(path.join(root,state.geometry_ref),'utf8'));
  const assignments = JSON.parse(await fs.readFile(path.join(root,state.assignments_ref),'utf8'));
  if (geometry.state !== state.code || geometry.year !== run.config.year) throw new Error('Geometry vintage mismatch.');
  const ids = geometry.features.map(feature=>feature.properties.geoid), known = new Set(ids);
  if (ids.some(id=>typeof id!=='string'||!/^\d{11}$/.test(id)) || known.size!==ids.length || Object.keys(assignments).some(id=>!known.has(id)) || Object.keys(assignments).length!==state.metrics.units) throw new Error('Invalid GEOID join.');
  if (Object.values(assignments).some(district=>!Number.isInteger(district)||district<1||district>state.k)) throw new Error('Invalid district assignment.');
}
console.log(JSON.stringify({schema:1,bytes,experiments:manifest.runs.length,coverage:manifest.coverage}));
