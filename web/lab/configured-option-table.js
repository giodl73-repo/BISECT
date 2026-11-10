// Request values after native YAML dispatch; applicability remains method-specific.
export function configuredOptionTable(result){
 const options=result?.engine_request?.options;
 if(!options||typeof options!=='object'||Array.isArray(options))return [];
 const note=options.structure==='compact-polsby'
  ? 'Native YAML uses a fixed 1% cut slack and has no epsilon option. Change slack in the laboratory. CompactBisect uses seeds 1 through the displayed per-split budget, regardless of the declared seed.'
  : options.structure==='flow-construction'
  ? 'Native YAML uses BFS repair. Select repair mode in the laboratory. Construction uses population and adjacency; boundary weights, random seeds and search settings do not change the generated plan. Boundary costs are reported measurements.'
  : typeof options.search==='string'&&options.search.startsWith('short-burst')
  ? 'Burst length × burst count sets the proposal budget. Zero work returns the initial plan; an omitted native burst seed initializes with seed 0. Boundary weights affect initialization. Endpoints are selected by unweighted cut percentile; this heuristic is not a calibrated ensemble.'
  : 'Values after YAML resolution and native dispatch. Each method uses its applicable options.';
 return [{title:'Options sent to the engine',note,headings:['Option','Value'],rows:Object.entries(options).slice(0,64).map(([name,value])=>[name,value===null?'Not supplied':typeof value==='boolean'?(value?'true':'false'):typeof value==='object'?JSON.stringify(value).slice(0,500):String(value)])}];
}
