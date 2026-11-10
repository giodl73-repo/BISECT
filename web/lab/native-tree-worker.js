import {prepareNativeCertifiedModel} from './native-model-prepare.js';
import {parseNativeTreeBundle} from './native-tree-bundle.js';
import {adoptNativeExternalTree} from './native-external-tree.js';
import {exportNativeTreePlan} from './native-tree-plan.js';
import {instantiateEngine} from './wasm-engine.js';
import {generateNativeProofTree} from './native-tree-generate.js';
import {sha256} from './proof-solver-core.js';
self.onmessage = async ({data}) => {
  try {
    if (data.mode === 'open') { self.postMessage({bundle: parseNativeTreeBundle(new Uint8Array(data.bytes))}); return; }
    if (data.mode === 'prepare-model') {
      const [metadata,artifact]=await Promise.all([fetch(new URL('./proof-checker/proof-checker-build.json',import.meta.url)),fetch(new URL('./proof-checker/bisect_proof_checker.wasm',import.meta.url))]);if(!metadata.ok||!artifact.ok)throw new Error('Native model preparation assets are unavailable.');
      self.postMessage({prepared:await prepareNativeCertifiedModel({contextJson:data.contextJson,districts:data.districts,weightPolicy:data.weightPolicy,receipt:await metadata.json(),wasmBinary:new Uint8Array(await artifact.arrayBuffer())})});return;
    }
    if (data.mode === 'generate') {
      const paths=['./bisect_wasm.wasm','./roundingsat/build-receipt.json','./roundingsat/roundingsat.wasm','./roundingsat/roundingsat.mjs','./proof-checker/proof-checker-build.json','./proof-checker/bisect_proof_checker.wasm'];
      const responses=await Promise.all(paths.map(path=>fetch(new URL(path,import.meta.url))));if(!responses.every(response=>response.ok))throw new Error('Automatic tree assets are unavailable.');
      const [engineBytes,solverReceipt,solverWasm,glue,checkerReceipt,checkerWasm]=await Promise.all(responses.map((response,index)=>[1,4].includes(index)?response.json():response.arrayBuffer().then(bytes=>new Uint8Array(bytes))));
      if(await sha256(glue)!==solverReceipt.files?.['roundingsat.mjs']?.sha256)throw new Error('Solver JavaScript identity mismatch.');
      const engine=await instantiateEngine(engineBytes),{default:solverFactory}=await import(new URL('./roundingsat/roundingsat.mjs',import.meta.url).href);
      self.postMessage({generated:await generateNativeProofTree({rootInstanceJson:data.rootInstanceJson,seed:data.seed,refinement:data.refinement,landAdjacency:data.landAdjacency,seconds:data.seconds,engine,solverFactory,solverReceipt,solverWasm,checkerReceipt,checkerWasm,onProgress:progress=>self.postMessage({progress})})});return;
    }
    if (data.mode === 'map') {
      const response = await fetch(new URL('./bisect_wasm.wasm', import.meta.url)); if (!response.ok) throw new Error('Native map engine asset is unavailable.');
      const engine = await instantiateEngine(new Uint8Array(await response.arrayBuffer()));
      self.postMessage({map: engine.execute({operation:'render-native-district-map',input:{document_json:data.documentJson,unit_geometry_json:data.geometryJson,dpi:data.dpi,map_type:'districts',round_key_basis:'unit-id',dissolve_method:'sequential'}})}); return;
    }
    if (!['verify', 'export-plan'].includes(data.mode)) throw new Error('Unknown tree worker action.');
    const [metadata, artifact] = await Promise.all([fetch(new URL('./proof-checker/proof-checker-build.json', import.meta.url)), fetch(new URL('./proof-checker/bisect_proof_checker.wasm', import.meta.url))]);
    if (!metadata.ok || !artifact.ok) throw new Error('Native tree checker assets are unavailable.');
    const receipt = await metadata.json(), wasmBinary = new Uint8Array(await artifact.arrayBuffer()), bundle = data.bundle;
    if (data.mode === 'export-plan') { self.postMessage({plan: await exportNativeTreePlan({treeRequest: {...bundle, expected_tree: data.tree}, contextJson: data.contextJson, profileJson: data.profileJson, label: data.label, createdAt: data.createdAt, wasmBinary, receipt})}); return; }
    self.postMessage({tree: await adoptNativeExternalTree({rootInstanceJson: bundle.root_instance_json, nodes: bundle.nodes, expectedTree: bundle.expected_tree ?? null, wasmBinary, receipt})});
  } catch (error) { self.postMessage({error: String(error?.message ?? error)}); }
};
