// Frozen references never name executable files or filesystem input paths.
import fs from 'node:fs/promises';import assert from 'node:assert/strict';import {createHash}from'node:crypto';
export async function loadFineUnitMapCorpus(){
 const root=new URL('./fine-unit-map/',import.meta.url),manifest=JSON.parse(await fs.readFile(new URL('manifest.json',root),'utf8')),raw=await fs.readFile(new URL('corpus.json',root));
 assert.equal(manifest.schema_version,'bisect-fine-unit-map-regression-v1');assert.equal(createHash('sha256').update(raw).digest('hex'),manifest.corpus_sha256);
 const corpus=JSON.parse(raw);assert.equal(corpus.schema_version,manifest.schema_version);assert.equal(corpus.cases.length,32);assert.equal(new Set(corpus.cases.map(c=>c.name)).size,32);return corpus;
}
