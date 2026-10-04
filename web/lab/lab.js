import {readElectionFile} from './election-input.js';
import {readPartisanFile} from './partisan-input.js';
import {readCharacterFile,validateCharacterInput,usesCharacterWeights} from './character-input.js';
import {loadStaticCatalog,parseSeed} from './static.js';
import {readDemographicFile,districtCountShares} from './demographic-input.js';
import {MAX_PROJECT_BYTES} from './project.js';
let demographicInputs=Object.create(null),demographicSequence=0,demographicLoading=false;
let electionInputs=Object.create(null),electionSequence=0,electionLoading=false;
let partisanInputs=Object.create(null),partisanSequence=0,partisanLoading=false;
let characterInputs=Object.create(null),characterSequence=0,characterLoading=false;

const $ = id => document.getElementById(id);

const browserWasm = document.body.dataset.backend === 'wasm';
const usesRunSeed = structure => !['proportional-section','spectral','flow-construction','capacity-clustering','regionalization','compact-polsby'].includes(structure);
if(browserWasm){
  const seed=$('seed');seed.type='text';seed.inputMode='numeric';seed.pattern='(0|[1-9][0-9]{0,19})';seed.maxLength=20;
  seed.removeAttribute('min');seed.removeAttribute('max');seed.title='Unsigned seed: 0 through 18446744073709551615';
}


const precomputed = document.body.dataset.backend === 'catalog';

let staticCatalog;

const esc = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));

const number = value => value == null ? '—' : Number(value).toLocaleString('en-US');

const percent = value => value == null ? '—' : `${Number(value).toFixed(3)}%`;

const boundary = value => value == null ? '—' : `${(value===0?0:value/1000).toLocaleString('en-US', {maximumFractionDigits: 1})} km`;

const time = value => value == null ? '—' : value < 60 ? `${value.toFixed(1)} s` : `${(value / 60).toFixed(1)} min`;

const palette = ['#6c9e83','#9386ad','#d6a668','#729ba8','#c4878b','#b4b577','#71999a','#b18f74','#7e91b8','#b194b4','#89b29b','#c3ab78'];

const structureNames = {'proportional-bisect':'Proportional bisection · votes/population','proportional-section':'ProportionalSection · two-party root','ratio-optimal-vra':'VRASection · minority mass alignment','simulated-annealing':'Simulated annealing','capacity-clustering':'Capacity clustering','regionalization':'Regionalization','flow-construction':'Flow construction','standard-bisect':'Standard bisection','nway':'Direct n-way partition','ratio-optimal':'GeoSection · ratio scan','ratio-optimal-area':'AreaSection · population + area','prime-factor':'ApportionRegions · prime-factor tree','compact-polsby':'CompactBisect · perimeter scoring','bfs-growth':'BFS region growing','centroidal-voronoi':'Centroidal Voronoi','moving-knife':'Moving-knife sweep','spectral':'Spectral-style smoothing'};

const structureDescriptions = {'proportional-bisect':'At each node, choose child seat counts from Democratic votes divided by census population. This legacy heuristic uses census population as its denominator, not two-party votes. Children follow the chosen counts. It does not guarantee partisan seat outcomes.','proportional-section':'Round the statewide two-party Democratic quota at the geometric mean threshold. Both root children get at least one seat. Apply population and Democratic vote constraints at the root; descendants use population-only GeoSection. Outcomes are heuristic.','ratio-optimal-vra':'Choose a root seat ratio with an alignment bonus based on minority mass concentration. Mass is tract minority fraction times total tract population. Descendants use plain ratio scans. This is a geographic proxy, not a district preservation rule or legal VRA certification.','simulated-annealing':'Refine each prescribed recursive split with boundary flips and geometric cooling. Population targets follow the floor/ceil seat ratio. Initial METIS partitions use boundary weights; annealing minimizes unweighted edge cuts. Initial refinement is fixed at 100 iterations.','capacity-clustering':'Deterministic farthest-point graph centers and nearest-center capacity assignment, with exhaustive repair on small graphs. Boundary weights affect reported costs, not construction.','regionalization':'Deterministic adjacent-region agglomeration with a population-capacity preference. Preserves the native merge history; small-graph repair may change the final plan. Boundary weights affect reported costs, not merges.','flow-construction':'Deterministic farthest-point graph seeds and population-capacity frontier growth, with optional native BFS repair. Uses unweighted adjacency; boundary weights affect reported costs. Random seeds and METIS iterations do not affect construction. Invalid plans are reported as failures.','standard-bisect':'Split floor(k/2) : ceiling(k/2) recursively. A heuristic implementation of the fixed schedule.','nway':'Assign all districts in a direct partition rather than a recursive tree.','ratio-optimal':'Explore different seat ratios. This changes the schedule from standard bisection.','ratio-optimal-area':'Explore root seat ratios with population and half-area targets. Root population multiplier: 1.001. Descendants use population-only cuts.','prime-factor':'Use the existing ApportionRegions schedule: largest prime factor first, with floor/ceil bisection for primes above 3. Single seed. Native research limit: 3% population deviation.','compact-polsby':'Choose recursive candidates using the engine’s geometric scoring.','bfs-growth':'Construct districts by graph growth. Uses its own search; select Single.','centroidal-voronoi':'Construct graph-distance regions around moving centers. Uses its own search.','moving-knife':'Sweep geographic orientations using the engine’s compactness proxy. Uses its own search.','spectral':'Deterministic spectral-style smoothing of unweighted adjacency, followed by a population-balanced sweep at each recursive split. Boundary weights affect reported costs, not smoothing or cut selection. Random seeds and METIS refinement iterations do not affect this method.'};

const searchNames = {'vra-recom':'VRA ReCom · minority preservation','smc-percentile':'SMC · weighted percentile','single':'Single seed','multi':'Multiple seeds · best candidate','convergence':'No-improvement seed sweep','percentile':'Seed distribution · percentile','bisection-ensemble':'Local bisection ensemble','short-burst':'Short-burst ReCom','short-burst-forest':'Short-burst forest ReCom','short-burst-merge-split':'Short-burst merge-split','flip':'Boundary flip chain','forest-recom':'Forest ReCom chain','merge-split':'Merge-split chain','parallel-tempering':'Parallel tempering'};

const searchDescriptions = {'single':'One seeded run. Fast baseline for exploring different dimensions.','multi':'Search a fixed seed budget and keep the engine’s best candidate.','convergence':'Search complete feasible standard-bisection plans from consecutive seeds. Stop after the requested non-improvement tail or maximum seed count. Rank by recursively normalized weighted cut. A seed limit is reported separately from convergence; neither proves global optimality.','percentile':'Rank a fixed set of seed candidates and choose the requested percentile.','bisection-ensemble':'Explore cuts locally at each bisection node and select a boundary percentile.','short-burst':'Explore full-plan endpoints in bursts of 20 steps.','short-burst-forest':'Use forest ReCom for full-plan bursts of 20 steps.','short-burst-merge-split':'Use merge-split for full-plan bursts of 20 steps.','flip':'Explore full plans through proposed boundary moves.','forest-recom':'Explore full plans with a forest ReCom chain.','merge-split':'Explore full plans with a merge-split chain.','parallel-tempering':'Use four interacting replicas; the seed budget controls tempering steps.'};

let catalog, mode = 'state', selectedStates = new Set(['RI','IA','NC']);

let job = null, history = [], selectedCode = 'RI', activeTab = 'districts', view = 'district';

let geometryRequest = 0, comparisonRequest = 0, polling = false;

let labProjectTools, projectRevision=0, projectSequence=0, projectImport, projectSaveSequence=0, projectSaveQueue=Promise.resolve();

let mapFeatures = [], mapGeoJSON = null, selectedDistrict = null, zoom = 1, pan = [0,0], base = null, drag = null;

const canvas = $('map'), ctx = canvas.getContext('2d');



async function api(path, options = {}) {

  if (precomputed || browserWasm) return staticCatalog.api(path, options);

  const response = await fetch(path, {cache:'no-store', ...options, headers:{'Content-Type':'application/json', ...(catalog ? {'X-Lab-Token':catalog.token} : {}), ...options.headers}});

  const value = await response.json();

  if (!response.ok) throw new Error(value.error || `Request failed (${response.status})`);

  return value;

}

function toast(message) { $('toast').textContent = message; $('toast').hidden = false; clearTimeout(toast.timer); toast.timer = setTimeout(() => $('toast').hidden = true, 6000); }

const stateInfo = code => catalog.states.find(state => state.code === code);

const yearInfo = (code, year = $('year').value) => stateInfo(code)?.years.find(entry => entry.year === year);

const seatCount = code => yearInfo(code)?.districts[$('chamber').value];

const scale = code => { const k = seatCount(code); return k <= 3 ? 'small' : k <= 12 ? 'medium' : 'large'; };



function populate() {

  const rootDetail=document.createElement('p');rootDetail.id='root-constraint-detail';rootDetail.className='help';rootDetail.hidden=true;
  if(browserWasm){const details=document.createElement('details');details.id='run-details';details.className='advanced';details.hidden=true;const summary=document.createElement('summary');summary.textContent='Run details';details.append(summary,rootDetail);document.querySelector('.metric-strip').after(details);}else{document.querySelector('.metric-strip').after(rootDetail);}

  $('structure').innerHTML = catalog.structures.map(value => `<option value="${esc(value)}">${esc(structureNames[value] || value)}</option>`).join('');

  $('weights').innerHTML = catalog.weights.map(value => `<option value="${value}">${{geographic:'Physical boundary length',unweighted:'Uniform · cut edge count',county:'Boundary length + county preference',partisan:'Partisan clusters · adaptive unit weights','economic-character':'Economic character · LODES similarity','housing-character':'Housing character · ACS similarity'}[value]}</option>`).join('');

  $('search').innerHTML = catalog.searches.map(value => `<option value="${value}">${esc(searchNames[value] || value)}</option>`).join('');

  $('search').value = 'single';
  if(browserWasm){$('structure').value='standard-bisect';$('weights').value='geographic';}

  $('state').innerHTML = [...catalog.states].sort((a,b) => a.name.localeCompare(b.name)).map(state => `<option value="${state.code}">${esc(state.name)} · ${state.code}</option>`).join('');

  $('state').value = 'RI';

  $('connection').textContent = precomputed ? 'Precomputed results · no live engine' : browserWasm ? 'Rust WASM · runs on your device' : catalog.engine_available ? 'Rust engine connected' : 'Rust engine binary missing';

  if(browserWasm)document.querySelector('.workspace-footer').firstElementChild.textContent='Rust WASM · Local browser execution';

  if (precomputed) {

    $('run').textContent = 'View saved results';

    document.querySelector('.panel-heading p').textContent = 'Choose saved settings. Explore real results. Generate custom runs with the local engine.';

    document.querySelector('.workspace-footer').firstElementChild.textContent = 'Static catalog · Previously computed Rust results';

    $('metric-time').previousElementSibling.textContent = 'Recorded run time';

    $('national-picker').querySelector('.help').textContent = 'Groups use chamber seat counts. Coverage refers to saved results; no national batch is computed in this browser.';

    for (const dimension of ['structure','weights','search']) for (const option of $(dimension).options) {

      if (!staticCatalog.manifest.runs.some(run => run.states.some(state=>state.metrics) && run.config[dimension]===option.value)) option.textContent += ' · not published';

    }

    const label=document.createElement('label');label.textContent='Saved experiment';

    const select=document.createElement('select');select.id='saved-experiment';select.innerHTML='<option value="">Choose published settings</option>' + staticCatalog.manifest.runs.map(run=>`<option value="${esc(run.id)}">${esc(run.config.name)}</option>`).join('');

    select.addEventListener('change',()=>{if(select.value)openHistory(select.value);});label.append(select);$('experiment-form').prepend(label);

    $('state').value = staticCatalog.manifest.runs.find(run=>run.states.some(state=>state.metrics))?.states.find(state=>state.metrics)?.code || 'RI';

    selectedCode=$('state').value;

  }

  const vraFields=document.createElement('section');vraFields.id='vra-fields';vraFields.hidden=true;
  const thresholdLabel=document.createElement('label');thresholdLabel.id='vra-threshold-field';thresholdLabel.textContent='Minority preservation threshold (%)';
  const threshold=document.createElement('input');threshold.id='vra-threshold';threshold.type='number';threshold.min=0;threshold.max=100;threshold.step='any';threshold.value=50;threshold.required=true;thresholdLabel.append(threshold);vraFields.append(thresholdLabel);
  const weightLabel=document.createElement('label');weightLabel.id='vra-weight-field';weightLabel.textContent='Minority alignment weight (%)';const weight=document.createElement('input');weight.id='vra-weight';weight.type='number';weight.min=0;weight.max=100;weight.step='any';weight.value=40;weight.required=true;weightLabel.append(weight);vraFields.append(weightLabel);
  const csvFields=document.createElement('div');csvFields.className='fields two';
  const csvStateLabel=document.createElement('label');csvStateLabel.textContent='CSV state';const csvState=document.createElement('select');csvState.id='demographic-csv-state';
  for(const state of [...catalog.states].sort((a,b)=>a.name.localeCompare(b.name))){const option=document.createElement('option');option.value=state.code;option.textContent=state.name;csvState.append(option);}csvState.value=$('state').value;csvStateLabel.append(csvState);csvFields.append(csvStateLabel);
  const basisLabel=document.createElement('label');basisLabel.textContent='CSV population basis';const csvBasis=document.createElement('select');csvBasis.id='demographic-csv-basis';
  for(const [value,text]of [['total-population','Total population'],['voting-age-population','Voting age · VAP'],['citizen-voting-age-population','Citizen voting age · CVAP']]){const option=document.createElement('option');option.value=value;option.textContent=text;csvBasis.append(option);}basisLabel.append(csvBasis);csvFields.append(basisLabel);vraFields.append(csvFields);
  const csvHelp=document.createElement('p');csvHelp.className='help';csvHelp.textContent='CSV: select its state and basis above; the selected Census year applies. Load one state at a time for national runs. Total population needs GEOID, total_pop, white_non_hispanic. VAP/CVAP needs GEOID, total_vap/vap/cvap, minority_vap. Counts are retained in saved projects; search uses tract fractions.';vraFields.append(csvHelp);
  const fileLabel=document.createElement('label');fileLabel.textContent='Demographic CSV or JSON files';
  const demographicFiles=document.createElement('input');demographicFiles.id='demographic-files';demographicFiles.type='file';demographicFiles.accept='.json,.csv';demographicFiles.multiple=true;fileLabel.append(demographicFiles);vraFields.append(fileLabel);
  const help=document.createElement('p');help.id='demographic-help';help.className='help';help.textContent='Complete tract fractions, state, year, population basis and source label are required. Saved projects include these inputs.';vraFields.append(help);
  const demographicStatus=document.createElement('p');demographicStatus.id='demographic-status';demographicStatus.setAttribute('role','status');vraFields.append(demographicStatus);
  const clearDemographics=document.createElement('button');clearDemographics.type='button';clearDemographics.textContent='Clear demographic inputs';clearDemographics.onclick=()=>{demographicSequence++;demographicLoading=false;demographicInputs=Object.create(null);demographicFiles.value='';renderDemographicInputs();};vraFields.append(clearDemographics);
  demographicFiles.onchange=async()=>{
    const files=[...demographicFiles.files],sequence=++demographicSequence,revision=projectRevision;
    const csvScope={state:csvState.value,year:$('year').value,basis:csvBasis.value};
    if(!files.length)return;
    demographicLoading=true;demographicStatus.textContent='Reading demographic inputs…';
    try{
      if(files.length>50||files.reduce((sum,file)=>sum+file.size,0)>MAX_PROJECT_BYTES)throw new Error('Select at most 50 demographic files totaling at most 25 MiB.');
      const next=Object.create(null);
      for(const file of files){const input=await readDemographicFile(file,{csvMetadata:{...csvScope,source_label:file.name},wasmSha256:staticCatalog?.manifest.wasm_sha256});if(Object.hasOwn(next,input.state))throw new Error('Select only one demographic file per state.');next[input.state]=input;}
      if(sequence!==demographicSequence||revision!==projectRevision)throw new Error('Current project changed while reading demographic files. Select them again.');
      demographicInputs={...demographicInputs,...next};renderDemographicInputs();
    }catch(error){if(sequence===demographicSequence)demographicStatus.textContent=error.message;}
    finally{if(sequence===demographicSequence){demographicLoading=false;demographicFiles.value='';}}
  };
  $('search-description').after(vraFields);renderDemographicInputs();
  if(browserWasm){for(const [value,label]of [['minority','Minority tract fraction'],['root','Root split side']]){const option=document.createElement('option');option.value=value;option.textContent=label;$('color-by').append(option);}}
  const electionFields=document.createElement('div');electionFields.id='election-fields';electionFields.className='fields';
  const etaLabel=document.createElement('label');etaLabel.id='proportional-eta-field';etaLabel.textContent='Root vote tolerance multiplier';const etaInput=document.createElement('input');etaInput.id='proportional-eta';etaInput.type='number';etaInput.min=1;etaInput.max=2;etaInput.step='any';etaInput.value=1.1;etaInput.required=true;etaLabel.append(etaInput);electionFields.append(etaLabel);
  const electionCsvStateLabel=document.createElement('label');electionCsvStateLabel.textContent='Election CSV state';const electionCsvState=document.createElement('select');electionCsvState.id='election-csv-state';
  for(const state of [...catalog.states].sort((a,b)=>a.name.localeCompare(b.name))){const option=document.createElement('option');option.value=state.code;option.textContent=state.name;electionCsvState.append(option);}electionCsvState.value=$('state').value;electionCsvStateLabel.append(electionCsvState);electionFields.append(electionCsvStateLabel);
  const electionYearLabel=document.createElement('label');electionYearLabel.textContent='CSV election year';const electionYear=document.createElement('input');electionYear.id='election-csv-year';electionYear.type='text';electionYear.inputMode='numeric';electionYear.pattern='[0-9]{4}';electionYear.placeholder='e.g. 2020';electionYear.maxLength=4;electionYearLabel.append(electionYear);electionFields.append(electionYearLabel);
  const electionFileLabel=document.createElement('label');electionFileLabel.textContent='Election counts CSV or JSON files';const electionFiles=document.createElement('input');electionFiles.id='election-files';electionFiles.type='file';electionFiles.accept='.json,.csv';electionFiles.multiple=true;electionFileLabel.append(electionFiles);electionFields.append(electionFileLabel);
  const electionHelp=document.createElement('p');electionHelp.className='help';electionHelp.textContent='CSV needs geoid, dem_votes, rep_votes; select its state and election year above. The selected Census year applies. Load one state at a time for national runs. JSON includes its own scope. Counts are retained in saved projects. Missing tracts are rejected; source labels are supplied by the importer. These methods do not certify election outcomes.';electionFields.append(electionHelp);
  const electionStatus=document.createElement('p');electionStatus.id='election-status';electionStatus.setAttribute('role','status');electionFields.append(electionStatus);
  const clearElections=document.createElement('button');clearElections.type='button';clearElections.textContent='Clear election inputs';clearElections.onclick=()=>{electionSequence++;electionLoading=false;electionInputs=Object.create(null);electionFiles.value='';renderElectionInputs();};electionFields.append(clearElections);
  electionFiles.onchange=async()=>{
    const files=[...electionFiles.files],sequence=++electionSequence,revision=projectRevision;if(!files.length)return;electionLoading=true;electionStatus.textContent='Reading election counts…';
    const csvScope={state:electionCsvState.value,year:$('year').value,election_year:electionYear.value};
    try{if(files.length>50||files.reduce((sum,file)=>sum+file.size,0)>MAX_PROJECT_BYTES)throw new Error('Select at most fifty election files totaling at most 25 MiB.');const next=Object.create(null);for(const file of files){const input=await readElectionFile(file,{csvMetadata:{...csvScope,source_label:file.name},wasmSha256:staticCatalog?.manifest.wasm_sha256});if(Object.hasOwn(next,input.state))throw new Error('Select one election file per state.');next[input.state]=input;}if(sequence!==electionSequence||revision!==projectRevision)throw new Error('The project changed while reading counts.');electionInputs={...electionInputs,...next};renderElectionInputs();}
    catch(error){if(sequence===electionSequence)electionStatus.textContent=error.message;}
    finally{if(sequence===electionSequence){electionLoading=false;electionFiles.value='';}}
  };
  $('structure-description').after(electionFields);renderElectionInputs();
  const partisanFields=document.createElement('div');partisanFields.id='partisan-fields';partisanFields.className='fields';
  for(const [id,text,value]of [['dem-threshold','Strong Democratic threshold (%)',55],['rep-threshold','Strong Republican threshold (%)',45]]){const label=document.createElement('label');label.textContent=text;const input=document.createElement('input');input.id=id;input.type='number';input.min=0;input.max=100;input.step='any';input.value=value;input.required=true;label.append(input);partisanFields.append(label);}
  const partisanTsvLabel=document.createElement('label');partisanTsvLabel.textContent='Partisan TSV state';const partisanTsvState=document.createElement('select');partisanTsvState.id='partisan-tsv-state';for(const state of [...catalog.states].sort((a,b)=>a.name.localeCompare(b.name))){const option=document.createElement('option');option.value=state.code;option.textContent=state.name;partisanTsvState.append(option);}partisanTsvState.value=$('state').value;partisanTsvLabel.append(partisanTsvState);partisanFields.append(partisanTsvLabel);
  const partisanFileLabel=document.createElement('label');partisanFileLabel.textContent='Partisan shares TSV or JSON files';const partisanFiles=document.createElement('input');partisanFiles.id='partisan-files';partisanFiles.type='file';partisanFiles.accept='.json,.tsv';partisanFiles.multiple=true;partisanFileLabel.append(partisanFiles);partisanFields.append(partisanFileLabel);
  const partisanHelp=document.createElement('p');partisanHelp.className='help';partisanHelp.textContent='TSV: geoid and dem_share, separated by tabs, with an optional header and # comment lines. Select its state above; the selected Census year applies. Load one state at a time for national runs. Shares must cover every tract. Uses adaptive unit weights for edges joining tracts of the same strong lean. This experimental rule does not certify proportional seats.';partisanFields.append(partisanHelp);
  const partisanStatus=document.createElement('p');partisanStatus.id='partisan-status';partisanStatus.setAttribute('role','status');partisanFields.append(partisanStatus);
  const clearPartisans=document.createElement('button');clearPartisans.type='button';clearPartisans.textContent='Clear partisan inputs';clearPartisans.onclick=()=>{partisanSequence++;partisanLoading=false;partisanInputs=Object.create(null);partisanFiles.value='';renderPartisanInputs();};partisanFields.append(clearPartisans);
  partisanFiles.onchange=async()=>{
    const files=[...partisanFiles.files],sequence=++partisanSequence,revision=projectRevision;if(!files.length)return;partisanLoading=true;partisanStatus.textContent='Reading partisan shares…';
    const tsvScope={state:partisanTsvState.value,year:$('year').value};
    try{if(files.length>50||files.reduce((sum,file)=>sum+file.size,0)>MAX_PROJECT_BYTES)throw new Error('Select at most fifty share files totaling at most 25 MiB.');const next=Object.create(null);for(const file of files){const input=await readPartisanFile(file,{tsvMetadata:{...tsvScope,source_label:file.name},wasmSha256:staticCatalog?.manifest.wasm_sha256});if(Object.hasOwn(next,input.state))throw new Error('Select one share file per state.');next[input.state]=input;}if(sequence!==partisanSequence||revision!==projectRevision)throw new Error('The project changed while reading shares.');partisanInputs={...partisanInputs,...next};renderPartisanInputs();}
    catch(error){if(sequence===partisanSequence)partisanStatus.textContent=error.message;}
    finally{if(sequence===partisanSequence){partisanLoading=false;partisanFiles.value='';}}
  };
  $('weights').after(partisanFields);renderPartisanInputs();
  const characterFields=document.createElement('div');characterFields.id='character-fields';characterFields.className='fields two';
  const blendLabel=document.createElement('label');blendLabel.textContent='Geographic blend alpha';const blend=document.createElement('input');blend.id='character-alpha';blend.type='number';blend.min='0';blend.max='1';blend.step='0.01';blend.value='0.5';blend.required=true;blendLabel.append(blend);characterFields.append(blendLabel);
  const characterStateLabel=document.createElement('label');characterStateLabel.textContent='Character CSV state';const characterState=document.createElement('select');characterState.id='character-csv-state';for(const state of [...catalog.states].sort((a,b)=>a.name.localeCompare(b.name))){const option=document.createElement('option');option.value=state.code;option.textContent=state.name;characterState.append(option);}characterState.value=$('state').value;characterStateLabel.append(characterState);characterFields.append(characterStateLabel);
  const characterYearLabel=document.createElement('label');characterYearLabel.textContent='Observation year';const characterYear=document.createElement('input');characterYear.id='character-data-year';characterYear.type='text';characterYear.inputMode='numeric';characterYear.pattern='[0-9]{4}';characterYear.maxLength=4;characterYear.value=$('year').value;characterYearLabel.append(characterYear);characterFields.append(characterYearLabel);
  const characterFileLabel=document.createElement('label');characterFileLabel.textContent='Character CSV or JSON files';const characterFiles=document.createElement('input');characterFiles.id='character-files';characterFiles.type='file';characterFiles.accept='.json,.csv';characterFiles.multiple=true;characterFileLabel.append(characterFiles);characterFields.append(characterFileLabel);
  const characterHelp=document.createElement('p');characterHelp.className='help';characterHelp.id='character-help';characterFields.append(characterHelp);
  const characterStatus=document.createElement('p');characterStatus.id='character-status';characterStatus.setAttribute('role','status');characterFields.append(characterStatus);
  const clearCharacters=document.createElement('button');clearCharacters.type='button';clearCharacters.textContent='Clear character inputs';clearCharacters.onclick=()=>{characterSequence++;characterLoading=false;characterInputs=Object.create(null);characterFiles.value='';renderCharacterInputs();};characterFields.append(clearCharacters);
  characterFiles.onchange=async()=>{
    const files=[...characterFiles.files],sequence=++characterSequence,revision=projectRevision;if(!files.length)return;characterLoading=true;characterStatus.textContent='Reading character observations…';
    const csvScope={state:characterState.value,year:$('year').value,data_year:characterYear.value,kind:$('weights').value.replace('-character','')};
    try{if(files.length>50||files.reduce((sum,file)=>sum+file.size,0)>MAX_PROJECT_BYTES)throw new Error('Select at most fifty character files totaling at most 25 MiB.');const next=Object.create(null);for(const file of files){const input=validateCharacterInput(await readCharacterFile(file,{csvMetadata:{...csvScope,source_label:file.name},wasmSha256:staticCatalog?.manifest.wasm_sha256}),null,csvScope.kind);if(Object.hasOwn(next,input.state))throw new Error('Select one character file per state.');next[input.state]=input;}if(sequence!==characterSequence||revision!==projectRevision)throw new Error('The project changed while reading observations.');characterInputs={...characterInputs,...next};renderCharacterInputs();}
    catch(error){if(sequence===characterSequence)characterStatus.textContent=error.message;}
    finally{if(sequence===characterSequence){characterLoading=false;characterFiles.value='';}}
  };
  partisanFields.after(characterFields);renderCharacterInputs();
  const saFields=document.createElement('div');saFields.id='sa-fields';saFields.className='fields two';

  for(const [id,text,min,max,value,step]of [['sa-steps','Annealing steps per tract',0,10000,10,1],['sa-factor','Initial temperature factor',0,1000,0.01,'any'],['sa-final','Final temperature',1e-15,1e6,0.0001,'any']]){

    const label=document.createElement('label');label.textContent=text;const input=document.createElement('input');input.id=id;input.type='number';input.min=min;input.max=max;input.value=value;input.step=step;input.required=true;label.append(input);saFields.append(label);

  }

  $('percentile-field').after(saFields);

  const flowField=document.createElement('label');flowField.id='flow-repair-field';flowField.textContent='Flow repair';

  const flowRepair=document.createElement('select');flowRepair.id='flow-repair';

  for(const [value,text]of [['bfs','BFS repair'],['none','No repair']]){const option=document.createElement('option');option.value=value;option.textContent=text;flowRepair.append(option);}

  flowField.append(flowRepair);saFields.after(flowField);

  const smcFields=document.createElement('div');smcFields.id='smc-fields';smcFields.className='fields two';

  for(const [id,text,min,max,value,step]of [['smc-particles','SMC particles',1,10000,5000,1],['smc-resample','ESS resampling threshold',0,1,0.5,'any']]){const label=document.createElement('label');label.textContent=text;const input=document.createElement('input');input.id=id;input.type='number';input.min=min;input.max=max;input.value=value;input.step=step;input.required=true;label.append(input);smcFields.append(label);}

  flowField.after(smcFields);

  const nwayFields=document.createElement('div');nwayFields.id='nway-fields';nwayFields.className='fields';
  const objectiveLabel=document.createElement('label');objectiveLabel.textContent='METIS refinement objective';const objective=document.createElement('select');objective.id='metis-objective';for(const [value,label]of [['cut','Edge cut'],['volume','Communication volume']]){const option=document.createElement('option');option.value=value;option.textContent=label;objective.append(option);}objectiveLabel.append(objective);nwayFields.append(objectiveLabel);
  const trialsLabel=document.createElement('label');trialsLabel.textContent='METIS internal trials';const trials=document.createElement('input');trials.id='metis-trials';trials.type='number';trials.min=1;trials.max=100;trials.step=1;trials.value=1;trials.required=true;trialsLabel.append(trials);nwayFields.append(trialsLabel);
  const nwayHelp=document.createElement('p');nwayHelp.className='help';nwayHelp.textContent='Volume changes refinement. Internal trials rank population imbalance, then edge cut. In recursive Multi, the separate seed budget selects the least weighted cut among balanced contiguous candidates at each tree node. GeoSection retains the least original weighted cut per ratio, then ranks ratios by cut divided by the square root of the smaller seat count. Refinement iterations apply to each trial.';nwayFields.append(nwayHelp);smcFields.after(nwayFields);
  const cvdFields=document.createElement('div');cvdFields.id='cvd-fields';cvdFields.className='fields two';

  const cvdIterationsLabel=document.createElement('label');cvdIterationsLabel.textContent='Voronoi iterations';const cvdIterations=document.createElement('input');cvdIterations.id='cvd-iters';cvdIterations.type='number';cvdIterations.min=0;cvdIterations.max=10000;cvdIterations.step=1;cvdIterations.value=20;cvdIterations.required=true;cvdIterationsLabel.append(cvdIterations);cvdFields.append(cvdIterationsLabel);

  const cvdMetricLabel=document.createElement('label');cvdMetricLabel.textContent='Voronoi distance';const cvdMetric=document.createElement('select');cvdMetric.id='cvd-metric';for(const [value,text]of [['graph-distance','Graph distance'],['geographic','Geographic distance']]){const option=document.createElement('option');option.value=value;option.textContent=text;cvdMetric.append(option);}cvdMetricLabel.append(cvdMetric);cvdFields.append(cvdMetricLabel);smcFields.after(cvdFields);

  const mkaFields=document.createElement('div');mkaFields.id='mka-fields';mkaFields.className='fields two';

  const orientationsLabel=document.createElement('label');orientationsLabel.textContent='Sweep orientations';const orientations=document.createElement('input');orientations.id='mka-orientations';orientations.type='number';orientations.min=0;orientations.max=10000;orientations.step=1;orientations.value=180;orientations.required=true;orientationsLabel.append(orientations);mkaFields.append(orientationsLabel);

  const metricLabel=document.createElement('label');metricLabel.textContent='Sweep score';const metric=document.createElement('select');metric.id='mka-metric';for(const [value,text]of [['reock','Reock'],['polsby','Polsby request · currently scores Reock']]){const option=document.createElement('option');option.value=value;option.textContent=text;metric.append(option);}metricLabel.append(metric);mkaFields.append(metricLabel);cvdFields.after(mkaFields);

  const compactField=document.createElement('label');compactField.id='compact-field';compactField.textContent='Edge-cut slack (%)';const compactSlack=document.createElement('input');compactSlack.id='compact-slack';compactSlack.type='number';compactSlack.min=0;compactSlack.max=100;compactSlack.step='any';compactSlack.value=5;compactSlack.required=true;compactField.append(compactSlack);mkaFields.after(compactField);
  const areaInitField=document.createElement('label');areaInitField.id='area-init-field';areaInitField.textContent='AreaSection initialization';const areaInit=document.createElement('select');areaInit.id='area-init';for(const [value,label]of [['ratio-optimal','Ratio scan'],['moving-knife','Moving-knife direction · 180 orientations']]){const option=document.createElement('option');option.value=value;option.textContent=label;areaInit.append(option);}areaInitField.append(areaInit);compactField.after(areaInitField);
  const burstFields=document.createElement('div');burstFields.id='burst-fields';burstFields.className='fields two';for(const [id,text,max,value]of [['burst-length','Steps per burst',100000,20],['n-bursts','Number of bursts',10000,50]]){const label=document.createElement('label');label.textContent=text;const input=document.createElement('input');input.id=id;input.type='number';input.min=0;input.max=max;input.step=1;input.value=value;input.required=true;label.append(input);burstFields.append(label);}areaInitField.after(burstFields);
  const ptFields=document.createElement('div');ptFields.id='pt-fields';ptFields.className='fields two';for(const [id,text,min,max,value,step]of [['pt-replicas','Tempering replicas',1,32,4,1],['pt-swap','Swap every steps',1,100000,10,1],['pt-hot','Hot population tolerance (%)',0.01,100,5,'any']]){const label=document.createElement('label');label.textContent=text;const input=document.createElement('input');input.id=id;input.type='number';input.min=min;input.max=max;input.step=step;input.value=value;input.required=true;label.append(input);ptFields.append(label);}burstFields.after(ptFields);
  if(browserWasm){
    const options=document.createElement('details');options.id='experiment-options';options.className='advanced';
    const summary=document.createElement('summary');summary.textContent='Experiment options';options.append(summary);
    const first=document.querySelector('#experiment-form .dimension'),last=document.querySelector('#experiment-form details.advanced');
    const seedLabel=$('seed').closest('label');seedLabel.id='run-seed-field';seedLabel.firstChild.textContent='Run seed';
    first.before(seedLabel,options);
    let node=first;
    while(node){const next=node.nextSibling;options.append(node);if(node===last)break;node=next;}
    document.querySelector('.panel-heading h1').textContent='Run. Inspect. Save.';
    document.querySelector('.panel-heading p').textContent='Choose a state, run a baseline, and explore its districts on the map.';
  }
  updateControls(); updateAvailability();

}

function percentageInput(value){
  for(let digits=1;digits<=17;digits++){const candidate=Number((value*100).toPrecision(digits));if(candidate/100===value)return String(candidate);}
  return String(value*100);
}

function updateControls() {

  const structure = $('structure').value;

  const supported = browserWasm&&structure==='standard-bisect'&&$('weights').value==='partisan'?['single','multi','percentile','convergence','bisection-ensemble']:catalog.search_compatibility[structure];

  for (const option of $('search').options) option.disabled = !supported.includes(option.value);

  if (!supported.includes($('search').value)) $('search').value = 'single';

  const search = $('search').value;
  const electionMethod=browserWasm&&['proportional-bisect','proportional-section'].includes(structure);$('election-fields').hidden=!electionMethod;$('election-files').disabled=!electionMethod;$('proportional-eta-field').hidden=structure!=='proportional-section';$('proportional-eta').disabled=structure!=='proportional-section';

  const standaloneChain=['vra-recom','forest-recom','merge-split','flip','bisection-ensemble'].includes(search);$('steps').min=standaloneChain?0:1;
  const vra=browserWasm&&search==='vra-recom',vraSection=browserWasm&&structure==='ratio-optimal-vra';$('vra-fields').hidden=!vra&&!vraSection;$('vra-threshold-field').hidden=!vra;$('vra-threshold').disabled=!vra;$('vra-weight-field').hidden=!vraSection;$('vra-weight').disabled=!vraSection;$('demographic-files').disabled=!vra&&!vraSection;
  $('demographic-help').textContent=vraSection?'Complete total-population fractions are required. Mass is fraction × tract population; the bonus uses the left share of total mass, compared with 50%. Unequal population halves can receive a bonus even with uniform fractions. Source labels are user supplied; projects retain all inputs.':'Complete fractions, state, year, population basis and source label are required. Source labels are user supplied. The sampler uses an unweighted tract mean; projects retain all inputs.';
  const pt=search==='parallel-tempering';$('seeds').min=pt?0:1;$('pt-fields').hidden=!pt;for(const id of ['pt-replicas','pt-swap','pt-hot'])$(id).disabled=!pt;
  const burst=search.startsWith('short-burst');$('burst-fields').hidden=!burst;for(const id of ['burst-length','n-bursts'])$(id).disabled=!burst;
  const smc=search==='smc-percentile';$('smc-fields').hidden=!smc;for(const id of ['smc-particles','smc-resample'])$(id).disabled=!smc;

  $('search').disabled = supported.length === 1;

  $('structure-description').textContent = structureDescriptions[structure] || '';

  $('search-description').textContent = (searchDescriptions[search] || '') + ' Only searches executed by this structure are enabled.';

  $('area-init-field').hidden=structure!=='ratio-optimal-area';$('area-init').disabled=structure!=='ratio-optimal-area';
  const compact=structure==='compact-polsby';$('compact-field').hidden=!compact;$('compact-slack').disabled=!compact;
  const mka=structure==='moving-knife';$('mka-fields').hidden=!mka;for(const id of ['mka-orientations','mka-metric'])$(id).disabled=!mka;

  const nway=browserWasm&&(structure==='nway'||(['ratio-optimal','ratio-optimal-area','ratio-optimal-vra'].includes(structure)&&['single','multi'].includes(search))||(structure==='standard-bisect'&&['single','multi','percentile','convergence','bisection-ensemble'].includes(search)));$('nway-fields').hidden=!nway;for(const id of ['metis-objective','metis-trials'])$(id).disabled=!nway;
  const cvd=structure==='centroidal-voronoi';$('cvd-fields').hidden=!cvd;for(const id of ['cvd-iters','cvd-metric'])$(id).disabled=!cvd;

  const annealing=structure==='simulated-annealing';

  $('flow-repair-field').hidden=structure!=='flow-construction';$('flow-repair').disabled=structure!=='flow-construction';

  $('sa-fields').hidden=!annealing;for(const id of ['sa-steps','sa-factor','sa-final'])$(id).disabled=!annealing;

  const partisanAllowed=browserWasm&&structure==='standard-bisect'&&['single','multi','percentile','convergence','bisection-ensemble'].includes(search);
  const partisanOption=[...$('weights').options].find(option=>option.value==='partisan');if(partisanOption)partisanOption.disabled=!partisanAllowed;
  if($('weights').value==='partisan'&&!partisanAllowed)$('weights').value='geographic';
  const partisan=browserWasm&&$('weights').value==='partisan';$('partisan-fields').hidden=!partisan;for(const id of ['dem-threshold','rep-threshold','partisan-files'])$(id).disabled=!partisan;
  const character=browserWasm&&usesCharacterWeights($('weights').value);$('character-fields').hidden=!character;for(const id of ['character-alpha','character-csv-state','character-data-year','character-files'])$(id).disabled=!character;
  $('character-help').textContent=($('weights').value==='economic-character'?'Economic CSV: geoid, c000, cns01, cns02, cns05, cns07, cns08, cns09, cns10, cns11. Jobs proxy is c000 / 10,000, capped at one.':'Housing CSV: geoid, pct_single_family, pct_multifamily, pct_owner, housing_vintage. These are derived native ACS housing values in [0,1].')+' All tracts are required. Select CSV state and observation year; the selected Census year identifies geometry. Load one CSV state at a time for national runs. Boundary weight = geographic length × (alpha + (1 − alpha) × cosine similarity). Alpha 1 retains geographic weights.';
  const spectral=structure==='spectral', flow=['flow-construction','capacity-clustering','regionalization'].includes(structure);

  if(standaloneChain)$('search-description').textContent='Run the chosen number of proposals using portable forward and reverse random streams. Retain the initial plan and accepted states; select by unweighted cut percentile. Zero steps return the initial plan.';
  if(pt)$('search-description').textContent='Run replicas on a geometric population tolerance ladder. The cold tolerance is the balance tolerance in run settings; the hot tolerance must be at least as large. Exchange only plans satisfying both receiving tolerances. Select a cold-chain record by unweighted cut percentile.';
  if(burst)$('search-description').textContent='Run the chosen number of bursts, restarting each chain from its preceding endpoint. Select an endpoint by unweighted cut percentile. Zero bursts return the initial plan; zero steps retain it. Total proposals are limited to 100,000.';
  if(structure==='proportional-section')$('search-description').textContent='Test seeds 1 through the seed budget at the root (one for Single). Keep the least weighted root cut. Descendants use population-only ratio scans with up to fifty seeds per ratio; the fixed seed schedule is recorded with the result.';
  if(compact)$('search-description').textContent='At each split, test seeds 1 through the seed budget (one candidate for single search). Among candidates within the cut slack, select by compactness. The starting seed does not apply.';
  if(flow)$('search-description').textContent='Deterministic graph construction; no random seed sampling.';

  if(spectral){$('search-description').textContent='Deterministic smoothing and sweep. This method does not sample random seeds.';}

  if(browserWasm)$('run-seed-field').hidden=!usesRunSeed(structure);
  $('weights').disabled=false;$('seed').disabled=structure==='proportional-section'||spectral||flow||compact;$('iterations').disabled=spectral||flow||annealing||smc||cvd||mka||search==='flip';if(search==='flip')$('iterations').value=100;

  $('tolerance').disabled=smc||vra;if(smc||vra)$('tolerance').value=0.5;

  if(search==='bisection-ensemble')$('search-description').textContent='At each recursive split, propose local ReCom cuts with portable Wilson trees. Preserve the prescribed seat ratio, including unequal splits. Rank the initial cut and accepted cuts by unweighted boundary percentile. Regions of at most four tracts use one METIS split. Zero proposals return initialization.';
  if(partisan&&search==='bisection-ensemble')$('search-description').textContent+=' Partisan boosts apply to METIS initialization; Wilson sampling and percentile ranking use unweighted adjacency.';
  if(partisan&&search==='percentile')$('search-description').textContent+=' Partisan boosts apply to each seeded construction; complete candidates are ranked by unweighted edge cuts.';
  if(character)$('search-description').textContent+=' Character blends apply wherever this method uses boundary weights. Unweighted sampling and ranking retain their existing rules.';
  if(search==='flip')$('search-description').textContent='Propose boundary-tract moves with portable random draws. Retain the initial plan and accepted moves; rank by unweighted cut percentile. Moves must preserve nonempty connected districts and the requested tolerance relative to ideal district population. Initialization uses 100 refinement iterations. Zero steps return the initial plan.';
  if(smc)$('search-description').textContent='Weighted SMC sample ranked by unweighted edge cuts. Native proposal tolerance is 0.5% of remaining component population, so final district balance can miss 0.5%; actual checks remain visible. Boundary weights affect reported costs. No optimality or distribution certification is claimed.';

  if(vraSection)$('search-description').textContent='At each root ratio, keep the seed with minimum weighted cut. Then select the ratio using its normalized cut minus an alignment bonus. Weight zero removes the bonus. Descendants have no demographic bonus. Supply total-population fractions; this does not preserve protected districts.';
  if(vra)$('search-description').textContent='Preserve districts whose initial unweighted mean of tract minority fractions reaches the threshold. Complete demographic inputs are required for every selected state. Population tolerance is fixed at 0.5%. This heuristic does not establish legal VRA compliance.';
  if(smc)$('structure-description').textContent='SMC constructs complete plans through sequential district proposals; this compositor supplies the selected plan.';

  $('steps-field').firstChild.textContent=spectral?'Spectral smoothing iterations':search==='convergence'?'Maximum seeds tried':search==='bisection-ensemble'?'Proposals per split':'Step budget';

  $('county-field').hidden = $('weights').value !== 'county';

  $('seeds-field').hidden = ['single','vra-recom','bisection-ensemble','flip','forest-recom','merge-split','short-burst','short-burst-forest','short-burst-merge-split'].includes($('search').value);

  $('steps-field').hidden = !spectral && ['single','multi','percentile','parallel-tempering'].includes($('search').value);

  if(burst)$('steps-field').hidden=true;
  if(smc){$('steps-field').hidden=true;$('seeds-field').hidden=true;}

  $('seeds-field').firstChild.textContent = search === 'convergence' ? 'No-improvement seeds' : search === 'parallel-tempering' ? 'Tempering steps' : 'Seed budget';

  $('percentile-field').hidden = ['single','multi','convergence'].includes($('search').value);

  $('area-field').hidden = structure !== 'ratio-optimal-area';

  $('percentile-output').textContent = Number($('percentile').value) === 0 ? 'Minimum · 0%' : `${$('percentile').value}%`;

}

function updateAvailability() {

  if($('save-lab-project'))$('save-lab-project').disabled=!job||(!job.id.startsWith('wasm-')&&!job.restored_from_project);

  for (const option of $('state').options) {

    const info = yearInfo(option.value), state = stateInfo(option.value), k = seatCount(option.value);

    option.textContent = `${state.name} · ${k ?? '?'} seats${info?.available ? '' : precomputed ? ' · not published' : ' · inputs missing'}`;

  }

  renderChecklist();

  const codes = mode === 'state' ? [$('state').value] : [...selectedStates];

  const ready = codes.filter(code => yearInfo(code)?.available && seatCount(code));

  $('data-status').classList.toggle('warning', ready.length !== codes.length || !codes.length);

  $('data-status').textContent = mode === 'state' ? (ready.length ? `${yearInfo(codes[0]).cached ? 'Graph ready' : 'Local Census inputs ready · graph prepared on first run'}` : 'Inputs or chamber allocation unavailable for this state/year.') : `${ready.length} of ${codes.length} selected states ready for this chamber and year.`;

  $('run').disabled = (!precomputed && !catalog.engine_available) || !codes.length || ready.length !== codes.length || (browserWasm && !!staticCatalog.active);

  if(precomputed) {const count=staticCatalog.coverage(config());$('data-status').textContent=`${count}/${codes.length} selected states have this exact saved configuration. No live computation.`;$('data-status').classList.toggle('warning',count!==codes.length);}

  $('selection-count').textContent = `${selectedStates.size} selected`;

  if (!job) { selectedCode = $('state').value; renderHeader(); }

}

function renderChecklist() {

  const filter = $('state-filter').value.trim().toLowerCase(), group = $('scale-filter').value;

  $('state-checklist').innerHTML = [...catalog.states].sort((a,b) => a.name.localeCompare(b.name)).filter(state => (!filter || `${state.name} ${state.code}`.toLowerCase().includes(filter)) && (group === 'all' || scale(state.code) === group)).map(state => {

    const available = yearInfo(state.code)?.available && seatCount(state.code), count = seatCount(state.code);

    return `<label class="${available ? '' : 'unavailable'}"><input type="checkbox" data-code="${state.code}" ${selectedStates.has(state.code) ? 'checked' : ''} ${available ? '' : 'disabled'}>${esc(state.name)}<span>${count ?? '—'} seats${available ? '' : ' · unavailable'}</span></label>`;

  }).join('');

}

function setMode(next) {

  mode = next; job = null; clearResult();

  document.querySelectorAll('[data-mode]').forEach(button => button.classList.toggle('active', button.dataset.mode === mode));

  $('state-label').hidden = mode === 'national'; $('national-picker').hidden = mode !== 'national'; $('districts-label').hidden = mode === 'national'; $('national-view').hidden = mode !== 'national'; $('national-map-view').hidden = mode !== 'national';

  updateAvailability(); setView(mode === 'national' ? 'national' : 'district');

  if (mode === 'state') loadGeometry($('state').value, $('year').value); else renderNationalGrid();

}

function renderElectionInputs(){if(!$('election-status'))return;$('election-status').textContent=Object.values(electionInputs).map(input=>`${input.state} ${input.year} Census · ${input.election_year} election · ${Object.keys(input.counts).length.toLocaleString()} tracts · ${input.source_label}`).join('; ')||'No election inputs loaded.';}
function renderPartisanInputs(){if(!$('partisan-status'))return;$('partisan-status').textContent=Object.values(partisanInputs).map(input=>`${input.state} ${input.year} · ${Object.keys(input.dem_shares).length.toLocaleString()} tracts · ${input.source_label}`).join('; ')||'No partisan inputs loaded.';}
function renderCharacterInputs(){if(!$('character-status'))return;$('character-status').textContent=Object.values(characterInputs).map(input=>`${input.state} ${input.year} Census · ${input.data_year} observations · ${input.data.kind} · ${Object.keys(input.data.characters).length.toLocaleString()} tracts · ${input.source_label}`).join('; ')||'No character inputs loaded.';}

function renderDemographicInputs(){
  const status=$('demographic-status');if(!status)return;
  status.textContent=Object.values(demographicInputs).map(input=>`${input.state} ${input.year} · ${input.basis} · ${Object.keys(input.minority_fractions).length.toLocaleString()} tracts${input.counts?' · counts retained':''} · ${input.source_label}`).join('; ')||'No demographic inputs loaded.';
}

function config() {
  if(electionLoading&&['proportional-bisect','proportional-section'].includes($('structure').value))throw new Error('Wait for election count files to finish loading.');
  if(partisanLoading&&$('weights').value==='partisan')throw new Error('Wait for partisan share files to finish loading.');
  if(characterLoading&&usesCharacterWeights($('weights').value))throw new Error('Wait for character files to finish loading.');
  if(demographicLoading&&($('search').value==='vra-recom'||$('structure').value==='ratio-optimal-vra'))throw new Error('Wait for demographic files to finish loading.');

  return {...(browserWasm&&usesCharacterWeights($('weights').value)?{character_alpha:Number($('character-alpha').value),characters:Object.fromEntries((mode==='state'?[$('state').value]:[...selectedStates]).map(code=>[code,characterInputs[code]]))}:{}),...(browserWasm&&['proportional-bisect','proportional-section'].includes($('structure').value)?{elections:Object.fromEntries((mode==='state'?[$('state').value]:[...selectedStates]).map(code=>[code,electionInputs[code]])),...($('structure').value==='proportional-section'?{proportional_eta:Number($('proportional-eta').value)}:{})}:{}),...(browserWasm&&$('weights').value==='partisan'?{dem_threshold:Number($('dem-threshold').value)/100,rep_threshold:Number($('rep-threshold').value)/100,partisans:Object.fromEntries((mode==='state'?[$('state').value]:[...selectedStates]).map(code=>[code,partisanInputs[code]]))}:{}),...(browserWasm&&($('structure').value==='nway'||(['ratio-optimal','ratio-optimal-area','ratio-optimal-vra'].includes($('structure').value)&&['single','multi'].includes($('search').value))||($('structure').value==='standard-bisect'&&['single','multi','percentile','convergence','bisection-ensemble'].includes($('search').value)))?{metis_objective:$('metis-objective').value,metis_trials:Number($('metis-trials').value)}:{}),...($('structure').value==='ratio-optimal-vra'?{w_vra:Number($('vra-weight').value)/100,demographics:Object.fromEntries((mode==='state'?[$('state').value]:[...selectedStates]).map(code=>[code,demographicInputs[code]]))}:{}),...($('search').value==='vra-recom'?{vra_threshold:Number($('vra-threshold').value)/100,demographics:Object.fromEntries((mode==='state'?[$('state').value]:[...selectedStates]).map(code=>[code,demographicInputs[code]]))}:{}),...($('search').value==='parallel-tempering'?{pt_replicas:Number($('pt-replicas').value),pt_swap_interval:Number($('pt-swap').value),pt_cold_tol:Number($('tolerance').value)/100,pt_hot_tol:Number($('pt-hot').value)/100}:{}),...($('search').value.startsWith('short-burst')?{burst_length:Number($('burst-length').value),n_bursts:Number($('n-bursts').value)}:{}),...($('structure').value==='ratio-optimal-area'?{area_init:$('area-init').value}:{}),...($('structure').value==='compact-polsby'?{compact_epsilon:Number($('compact-slack').value)/100}:{}),...($('structure').value==='moving-knife'?{mka_orientations:Number($('mka-orientations').value),mka_metric:$('mka-metric').value}:{}),...($('structure').value==='centroidal-voronoi'?{cvd_iters:Number($('cvd-iters').value),cvd_metric:$('cvd-metric').value}:{}),...($('search').value==='smc-percentile'?{smc_particles:Number($('smc-particles').value),smc_resample_threshold:Number($('smc-resample').value)}:{}),...($('structure').value==='flow-construction'?{flow_repair:$('flow-repair').value}:{}),...($('structure').value==='simulated-annealing'?{sa_steps_per_tract:Number($('sa-steps').value),sa_t0_factor:Number($('sa-factor').value),sa_t_final:Number($('sa-final').value)}:{}),name:$('name').value.trim(),mode,states:mode === 'state' ? [$('state').value] : [...selectedStates].sort(),year:$('year').value,chamber:$('chamber').value,districts:mode === 'state' && $('districts').value ? Number($('districts').value) : null,structure:$('structure').value,weights:$('weights').value,search:$('search').value,seed:browserWasm?parseSeed($('seed').value):Number($('seed').value),seeds:Number($('seeds').value),steps:Number($('steps').value),percentile:Number($('percentile').value)/100,alpha_county:$('weights').value === 'county' ? Number($('alpha').value) : 0,balance_tolerance:Number($('tolerance').value),area_swing:Number($('area-swing').value),iterations:Number($('iterations').value),timeout_seconds:Number($('timeout').value)};

}

async function submit(event) {

  event.preventDefault(); $('form-error').hidden = true; $('run').disabled = true;

  try {

    job = await api('/api/runs', {method:'POST',body:JSON.stringify(config())});

    selectedCode = job.config.states[0]; selectedDistrict = null; clearResult();

    renderJob(); await refreshHistory(); setView(job.config.mode === 'national' ? 'national' : 'district');

    if(browserWasm)autosaveLabProject();

    loadGeometry(selectedCode, job.config.year); toast(precomputed ? 'Saved results loaded. Missing combinations are shown explicitly.' : 'Experiment queued. Results will appear as each state completes.');

  } catch (error) { $('form-error').textContent = error.message; $('form-error').hidden = false; }

  finally { updateAvailability(); }

}

function renderHeader() {

  const state = stateInfo(selectedCode), configuration = job?.config;

  $('map-title').textContent = view.startsWith('national') ? 'National laboratory' : state?.name || selectedCode;

  const year = configuration?.year || $('year').value, chamber = configuration?.chamber || $('chamber').value;

  $('map-subtitle').textContent = `${year} Census · ${chamber === 'congressional' ? 'Congressional' : chamber === 'house' ? 'State House' : 'State Senate'}${configuration ? ` · ${configuration.name}` : ' · input preview'}`;

  if(browserWasm&&configuration&&usesRunSeed(configuration.structure)){
    const stateResult=job.states.find(state=>state.code===selectedCode);
    const evidence=stateResult?.metrics?.structure_evidence;
    const selected=evidence?.method==='convergence'?evidence.selected_seed:null;
    $('map-subtitle').textContent+=` · Run seed ${configuration.seed}${selected!=null?` · Selected seed ${selected}`:''}`;
  }

  if(browserWasm&&configuration?.structure==='proportional-section'&&job.states.find(state=>state.code===selectedCode)?.metrics?.district_count>1){const budget=configuration.search==='multi'?configuration.seeds:1;$('map-subtitle').textContent+=budget===1?' · Seed 1':` · Seeds 1–${budget}`;}
  const status = job?.status || 'preview'; $('run-status').textContent = status; $('run-status').className = `badge ${status}`;

  $('cancel').hidden = !job || !['queued','running'].includes(job.status);

  $('national-view').hidden = (configuration?.mode || mode) !== 'national';

  $('national-map-view').hidden = $('national-view').hidden;

}

function clearResult() {

  if($('root-constraint-detail'))$('root-constraint-detail').hidden=true;

  $('execution-warning').hidden = true;

  for (const key of ['districts','deviation','boundary','counties','time']) $(`metric-${key}`).textContent = '—';

  $('district-table').hidden = true; $('district-empty').hidden = false; $('batch-results').hidden = true;

  $('download-config').disabled = !job; $('download-map').disabled = true; $('legend').innerHTML = '';

  $('compare-run').value = ''; $('comparison').textContent = 'Compare the same state across configurations to see the tradeoffs.';

  $('metric-deviation').classList.remove('deviation-bad');

  $('unit-count').textContent = 'No result selected';

  $('map-caption').textContent = 'Input preview · Display geometry simplified';

}

function renderJob() {

  $('root-constraint-detail').hidden=true;

  if($('merge-history'))$('merge-history').hidden=true;if($('smc-diagnostics'))$('smc-diagnostics').hidden=true;

  if($('save-lab-project'))$('save-lab-project').disabled=!job.id.startsWith('wasm-')&&!job.restored_from_project;

  renderHeader(); $('download-config').disabled = false;

  const ignored = !catalog.search_compatibility[job.config.structure]?.includes(job.config.search);

  const missingConvergenceEvidence=job.config.search==='convergence'&&job.states.some(state=>state.metrics?.district_count>1&&state.metrics.structure_evidence?.method!=='convergence');

  $('execution-warning').hidden = !ignored && !missingConvergenceEvidence && !job.restored_from_project;

  $('execution-warning').textContent = [job.restored_from_project ? 'Imported project: assignments and metrics checked against hashed inputs. Recorded engine provenance is unverified. No engine was executed when opening this project.' : '',missingConvergenceEvidence?'Recorded convergence settings lack stopping evidence. Do not interpret the seed budget as verified execution.':'',ignored ? 'Historical run: the requested search is not executed by this structure in the current engine. Do not interpret its requested search budget as executed sampling.' : ''].filter(Boolean).join(' ');

  $('log').textContent = job.logs.join('\n') || 'Waiting for the engine…';

  if (activeTab === 'log') $('log').scrollTop = $('log').scrollHeight;

  const result = job.states.find(state => state.code === selectedCode), metrics = result?.metrics;

  if (metrics) renderMetrics(metrics, result.elapsed_seconds); else {

    for (const key of ['districts','deviation','boundary','counties','time']) $(`metric-${key}`).textContent = '—';

    $('district-table').hidden = true; $('district-empty').hidden = false;

    $('district-empty').textContent = result?.error || (result?.status === 'preparing' ? 'Preparing the geographic graph from Census inputs…' : 'District results appear when this state completes.');

  }

  $('batch-results').hidden = job.config.mode !== 'national';

  const done = job.states.filter(state => ['completed','failed','cancelled','interrupted','unavailable'].includes(state.status)).length;

  $('batch-progress').textContent = `${done} / ${job.states.length} states finished`;

  $('batch-bar').style.width = `${100 * done / job.states.length}%`;

  $('batch-rows').innerHTML = job.states.map(state => {

    const m = state.metrics;

    return `<tr><td><button data-result-state="${state.code}" title="Inspect ${esc(stateInfo(state.code)?.name)}">${state.code}</button></td><td><span class="badge ${state.status}">${esc(state.status)}</span></td><td>${number(m?.district_count)}</td><td>${number(m?.units)}</td><td>${percent(m?.max_deviation_percent)}</td><td>${boundary(m?.graph_boundary_m)}</td><td>${number(m?.split_counties)}</td><td>${state.elapsed_seconds ? time(state.elapsed_seconds) : '—'}</td></tr>${state.error ? `<tr><td colspan="8" class="state-error">${esc(state.code)}: ${esc(state.error)}</td></tr>` : ''}`;

  }).join('');

  if (view === 'national') renderNationalGrid();

  $('download-map').disabled = !metrics;

  if(view.startsWith('national'))renderCohortMetrics();

  if ($('compare-run').value) updateComparison();

}

function renderMetrics(metrics, elapsed) {

  const root=metrics.root_split, evidence=metrics.structure_evidence;

  $('root-constraint-detail').hidden=metrics.district_count!==1&&!root&&!(evidence&&(['proportional-bisect','proportional-section','nway-metis','recursive-metis','percentile-metis'].includes(evidence.method)||job.config.structure==='prime-factor'||job.config.structure==='spectral'||['simulated-annealing','flow-construction','capacity-clustering','regionalization'].includes(job.config.structure)||['convergence','smc-percentile','short-burst','short-burst-forest','short-burst-merge-split','vra-recom','parallel-tempering','forest-recom','merge-split','flip','bisection-ensemble'].includes(job.config.search)));

  if(metrics.district_count===1)$('root-constraint-detail').textContent='One district: all tracts are assigned together. No split or search was needed. Population and graph connectivity are checked below.';
  if(root)$('root-constraint-detail').textContent=`Root split ${root.left_districts}:${root.right_districts} · ${percent(root.area_fraction_left==null?null:root.area_fraction_left*100)} of land on left${root.area_constraint_active?` · area swing ${root.area_swing} ${root.area_within_requested_swing?'satisfied':'missed'} · root population multiplier ${root.population_multiplier}`:''} · ${root.seeds_per_ratio} seed${root.seeds_per_ratio===1?'':'s'} per ratio, starting at ${root.base_seed}. ${root.area_constraint_active?'Area constraints apply only at the root. ':''}District population and connectivity checks are shown below.${metrics.structure_evidence?.method==='areasection-initialization'?` Initialization: ${metrics.structure_evidence.initialization}${metrics.structure_evidence.direction_radians==null?'':` at ${(metrics.structure_evidence.direction_radians*180/Math.PI).toFixed(1)} degrees (180 orientations, directional bias 1)`}.`:''}`;

  if(root&&evidence?.metis_refinement)$('root-constraint-detail').textContent+=` Ratio search uses ${evidence.metis_refinement.objective} refinement with ${evidence.metis_refinement.internal_trials} internal trials per candidate at each recursive ratio node. Internal trials rank population excess then edge cut; root area or demographic selection rules remain as recorded.`;
  if(root&&evidence?.method==='geosection-metis')$('root-constraint-detail').textContent+=` GeoSection uses ${evidence.refinement_objective} refinement with ${evidence.internal_trials_per_candidate} internal trials at each recursive ratio search. Internal trials rank population excess then edge cut; ratio candidates retain minimum original weighted cut and ratios rank by cut divided by the square root of the smaller seat count.`;
  if(!root&&evidence?.method==='percentile-metis')$('root-constraint-detail').textContent=`Percentile sweep · ${evidence.seed_count} complete plans ranked by unweighted edge cut · selected rank ${evidence.rank+1}/${evidence.seed_count}, seed index ${evidence.selected_seed_index}, ${evidence.selected_edge_cut} cut edges · ${evidence.refinement_objective} refinement with ${evidence.internal_trials_per_candidate} internal trials per recursive split. Ties use seed index. This describes the sampled bisection family, not all valid plans.`;
  if(!root&&evidence?.method==='recursive-metis')$('root-constraint-detail').textContent=`Recursive METIS · ${evidence.refinement_objective} refinement · ${evidence.internal_trials_per_candidate} internal trials per candidate · ${evidence.candidates_per_node} outer candidates per tree node. Internal trials rank imbalance then edge cut; Multi selects the least weighted cut among balanced contiguous candidates.`;
  if(!root&&evidence?.method==='nway-metis')$('root-constraint-detail').textContent=`Direct METIS · ${evidence.refinement_objective} refinement · ${evidence.internal_trials} internal trials, ranked by imbalance then edge cut.`;
  if(!root&&evidence?.method==='local-bisection-ensemble')$('root-constraint-detail').textContent=`Local bisection ensemble · ${evidence.steps_per_split} proposals per recursive split · initial and accepted cuts ranked by unweighted percentile ${percent(evidence.percentile*100)} · portable Wilson trees · prescribed floor/ceil seat targets. ${evidence.metis_initialization?`${evidence.metis_initialization.objective} initialization with ${evidence.metis_initialization.internal_trials} internal trials. `:""}Regions of at most four tracts use one METIS split. Optimality and sampling distribution unproved.`;
  if(!root&&evidence?.method==='boundary-flip')$('root-constraint-detail').textContent=`Boundary flip · ${evidence.steps} proposals · ${evidence.record_count} retained records · selected rank ${evidence.selected_rank} · initial refinement 100 iterations · portable ChaCha12 draws. Accepted moves check population against ideal district size and retain nonempty connected districts. Optimality and sampling distribution unproved.`;
  if(!root&&['forest-recom','merge-split'].includes(evidence?.method))$('root-constraint-detail').textContent=`${evidence.method} · ${evidence.steps} proposals · retain initial and accepted states · unweighted cut percentile ${percent(evidence.percentile*100)} · portable ChaCha12 random draws. Boundary weights affect initialization. Optimality and sampling distribution unproved.`;
  if(root&&evidence?.method==='vra-section')$('root-constraint-detail').textContent+=` Minority mass proxy on left: ${percent(evidence.minority_share_left==null?null:evidence.minority_share_left*100)} · alignment ${percent(evidence.alignment*100)} · alignment weight ${percent(evidence.w_vra*100)} · selected normalized cut ${number(evidence.normalised_cut)} and adjusted score ${number(evidence.selection_score)}. Bonus applies only at the root. Source: ${job.config.demographics[selectedCode].source_label}. No minority-preservation threshold is enforced.`;
  if(!root&&evidence?.method==='vra-recom')$('root-constraint-detail').textContent=`Minority preservation · ${evidence.run.protected_districts.length} protected districts · threshold ${percent(evidence.threshold*100)} · unweighted tract mean (${evidence.basis}). ${evidence.run.accepted_moves} accepted of ${evidence.run.proposals} proposals; ${evidence.run.minority_rejections} minority-rule rejections. Source: ${job.config.demographics[selectedCode].source_label}. Demographic identity and final preservation checked on Open; recorded execution provenance is unverified.`;
  if(!root&&evidence?.method==='parallel-tempering')$('root-constraint-detail').textContent=`Parallel tempering · ${evidence.replicas} replicas · ${evidence.steps} steps each · swap interval ${evidence.swap_interval} · population ladder ${percent(evidence.cold_tolerance*100)} to ${percent(evidence.hot_tolerance*100)} · exchanges require both receiving tolerances. Cold records selected by unweighted cut percentile; optimality unproved.`;
  if(!root&&evidence?.method==='short-burst')$('root-constraint-detail').textContent=`${evidence.chain} · ${evidence.n_bursts} bursts × ${evidence.burst_length} proposals · endpoints selected by unweighted cut percentile · portable ChaCha12 random draws. Initialization uses the selected boundary weights. Optimality unproved.`;
  if(!root&&job.config.structure==='prime-factor'&&evidence?.method==='apportion-regions')$('root-constraint-detail').textContent=`ApportionRegions · factors ${evidence.factor_sequence.join(' × ')} · tree depth ${evidence.tree_depth} · per-level population tolerance ${percent(evidence.per_level_tolerance*100)} · native research limit ${evidence.research_balance_limit_percent}% · seed ${evidence.seed}. Largest prime first; primes above 3 use floor/ceil bisection.`;

  if(!root&&job.config.structure==='simulated-annealing'&&evidence?.method==='simulated-annealing')$('root-constraint-detail').textContent=`Simulated annealing · ${evidence.steps_per_tract} steps per tract per split · initial temperature factor ${evidence.t0_factor} · final temperature ${evidence.t_final} · ${evidence.objective}. Population targets follow the floor/ceil seat ratio; optimality unproved.`;

  if(!root&&job.config.structure==='capacity-clustering'&&evidence?.method==='capacity-clustering')$('root-constraint-detail').textContent=`Capacity clustering · ${evidence.seed_method} graph centers · ${evidence.repair_method} repair · ${evidence.capacity_status} · unweighted edge cut ${evidence.edge_cut}. Population and connectivity checked; optimality unproved.`;

  if(!root&&job.config.structure==='regionalization'&&evidence?.method==='regionalization')$('root-constraint-detail').textContent=`Regionalization · ${evidence.summary.merge_count} recorded merges · hierarchy depth ${evidence.summary.hierarchy_depth} · ${evidence.summary.repair_method} repair · unweighted edge cut ${evidence.summary.edge_cut}. Merge history is retained in saved projects and run records. Population and connectivity checked; optimality unproved.`;

  if(!root&&job.config.structure==='flow-construction'&&evidence?.method==='flow-construction')$('root-constraint-detail').textContent=`Flow construction · ${evidence.seed_method} graph seeds · ${evidence.seeds.length} centers · ${evidence.repair_method} repair · ${evidence.status} · unweighted edge cut ${evidence.edge_cut}. Population and connectivity checked; optimality unproved.`;

  if(!root&&evidence?.method==='smc-percentile')$('root-constraint-detail').textContent=`SMC · ${evidence.particles} weighted particles · selected particle ${evidence.selected_particle} · percentile ${percent(evidence.percentile*100)} · ${evidence.resample_count} resampling rounds · unweighted edge cut ${evidence.selected_edge_cut}. Native sampler tolerance: 0.5% of remaining component population. Final district checks are independent; no distribution or optimality certificate.`;

  if(!root&&job.config.structure==='spectral'&&evidence?.method==='spectral')$('root-constraint-detail').textContent=`Spectral smoothing · ${evidence.max_iters} maximum iterations per split · ${evidence.nodes.length} split node${evidence.nodes.length===1?'':'s'} · ${evidence.nodes.filter(node=>node.converged).length} converged · unweighted edge cut ${evidence.edge_cut}. Smoothing convergence does not certify an optimal cut; global population and connectivity checks remain separate.`;

  if(!root&&job.config.search==='convergence'&&evidence?.method==='convergence')$('root-constraint-detail').textContent=`Convergence search · ${evidence.attempted} seeds tried, ${evidence.rejected} rejected · selected seed ${evidence.selected_seed} · normalized cut ${Number(evidence.best_normalized_cut).toFixed(3)} · ${evidence.consecutive_non_improving}/${evidence.threshold} non-improving seeds · ${evidence.halt==='threshold'?'threshold reached':'seed limit reached before convergence'}. ${evidence.metis_refinement?`${evidence.metis_refinement.objective} refinement with ${evidence.metis_refinement.internal_trials_per_candidate} internal trials per split. `:""}This is a best-observed full-plan search, not an optimality certificate.`;

  if(evidence?.method==='proportional-bisect')$('root-constraint-detail').textContent=`Proportional recursion · ${evidence.splits.length} chosen splits · Democratic votes divided by census population · rounded child seat counts · run seed ${evidence.seed}. Input: ${job.config.elections[selectedCode].source_label}. No partisan outcome guarantee.`;
  if(evidence?.method==='proportional-section')$('root-constraint-detail').textContent=`ProportionalSection · root ${evidence.left_seats}:${evidence.right_seats} · statewide two-party Democratic share ${percent(evidence.statewide_democratic_share*100)} · vote tolerance ${evidence.eta} · seeds 1–${evidence.seeds_per_root} · root population ${evidence.root_population_within_requested_multiplier?'meets':'misses'} requested 1.001 multiplier; supplied vote counts ${evidence.root_democratic_within_requested_multiplier?'meet':'miss'} the requested vote multiplier · descendants use population-only GeoSection. Input: ${job.config.elections[selectedCode].source_label}. No partisan outcome guarantee.`;
  if(metrics.weighting_evidence){const w=metrics.weighting_evidence;if($('root-constraint-detail').hidden)$('root-constraint-detail').textContent='';$('root-constraint-detail').hidden=false;$('root-constraint-detail').textContent+=w.method==='character-cosine-blend'?` ${w.kind==='economic'?'Economic':'Housing'} character weighting · geographic blend alpha ${w.alpha} · ${w.data_year} observations · ${w.tracts} tracts. Input: ${job.config.characters[selectedCode].source_label}. Character observations, blend and input hash are retained in saved projects.`:` Partisan weighting: ${w.strong_tracts}/${w.tracts} strong tracts, adaptive boost ${w.alpha.toFixed(3)} on same-lean edges; baseline unit weights. Input: ${job.config.partisans[selectedCode].source_label}. Shares and thresholds are retained in saved projects.`;}
  if(browserWasm)$('run-details').hidden=$('root-constraint-detail').hidden;
  renderMergeHistory(evidence);

  renderSmcDiagnostics(evidence);

  $('metric-districts').textContent = number(metrics.district_count); $('metric-deviation').textContent = percent(metrics.max_deviation_percent);

  $('metric-boundary').textContent = boundary(metrics.graph_boundary_m); $('metric-counties').textContent = number(metrics.split_counties); $('metric-time').textContent = time(elapsed);

  $('metric-deviation').classList.toggle('deviation-bad', !metrics.balance_passed);

  $('district-empty').hidden = true; $('district-table').hidden = false;

  $('district-rows').innerHTML = metrics.districts.map(d => `<tr><td><button data-district="${d.district}"><i class="district-dot" style="background:${districtColor(d.district)}"></i>District ${d.district}</button></td><td>${number(d.population)}</td><td>${number(d.units)}</td><td class="${Math.abs(d.deviation_percent) > job.config.balance_tolerance ? 'deviation-bad' : ''}">${d.deviation_percent > 0 ? '+' : ''}${percent(d.deviation_percent)}</td><td class="${d.components === 1 ? 'check-ok' : 'deviation-bad'}">${d.components === 1 ? 'Connected' : `${d.components} components`}</td></tr>`).join('');

  $('unit-count').textContent = `${selectedCode} · ${number(metrics.units)} assigned tracts · ${metrics.contiguous ? 'Graph connectivity checked' : 'Disconnected districts detected'} · Optimality unproved`;

  $('metric-time').title = `Total includes preparation and output processing. Engine: ${metrics.engine_seconds == null ? 'not separately recorded' : time(metrics.engine_seconds)}; preparation: ${metrics.preparation_seconds == null ? 'not separately recorded' : time(metrics.preparation_seconds)}.`;

  renderLegend();

}

function renderSmcDiagnostics(evidence){

  let panel=$('smc-diagnostics');

  if(!panel){panel=document.createElement('details');panel.id='smc-diagnostics';panel.className='table-scroll';$('root-constraint-detail').after(panel);}

  panel.hidden=evidence?.method!=='smc-percentile'||!Array.isArray(evidence.ranked_particles);panel.replaceChildren();if(panel.hidden)return;

  const heading=document.createElement('summary');heading.textContent='SMC weights and effective sample sizes';panel.append(heading);

  const note=document.createElement('p');note.textContent='First 50 ranked particles and stages shown, plus the selected particle when outside that range. Full diagnostics are in the run record and saved project. Particle IDs are zero-based. Weights and ESS do not certify distribution calibration.';panel.append(note);

  function table(headers,rows){const table=document.createElement('table'),head=document.createElement('tr');for(const label of headers){const cell=document.createElement('th');cell.textContent=label;head.append(cell);}table.append(head);for(const values of rows){const row=document.createElement('tr');for(const value of values){const cell=document.createElement('td');cell.textContent=String(value);row.append(cell);}table.append(row);}panel.append(table);}

  const particles=evidence.ranked_particles.slice(0,50),selected=evidence.ranked_particles.find(p=>p.particle===evidence.selected_particle);

  if(selected&&!particles.includes(selected))particles.push(selected);

  table(['Particle','Edge cuts','Weight','Selection'],particles.map(p=>[number(p.particle),number(p.edge_cut),Number(p.weight).toPrecision(6),p.particle===evidence.selected_particle?'Selected':'']));

  const rounds=new Set(evidence.resample_rounds);

  table(['Stage','ESS before resampling','Resampled'],evidence.ess_trace.slice(0,50).map((ess,i)=>[i+1,Number(ess).toPrecision(6),rounds.has(i+1)?'Yes':'No']));

}



function renderMergeHistory(evidence){

  let panel=$('merge-history');

  if(!panel){panel=document.createElement('details');panel.id='merge-history';panel.className='table-scroll';$('root-constraint-detail').after(panel);}

  panel.hidden=evidence?.method!=='regionalization'||!Array.isArray(evidence.merge_log);

  panel.replaceChildren();if(panel.hidden)return;

  const heading=document.createElement('summary');heading.textContent=`Merge history · ${evidence.merge_log.length} merges`;panel.append(heading);

  const note=document.createElement('p');note.textContent='First 50 merges shown. Download the run record or save the project for the full history. Region IDs are zero-based graph indices; a merge retains its left region ID. Small-graph repair may change the final assignments.';panel.append(note);

  const table=document.createElement('table'),header=document.createElement('tr');

  for(const label of ['Step','Left region','Right region','Merged population','Shared cut edges']){const cell=document.createElement('th');cell.textContent=label;header.append(cell);}table.append(header);

  for(const merge of evidence.merge_log.slice(0,50)){

    const row=document.createElement('tr');

    for(const key of ['step','left_region','right_region','merged_population','cut_edges_between']){const cell=document.createElement('td');cell.textContent=Number.isSafeInteger(merge?.[key])?number(merge[key]):'—';row.append(cell);}table.append(row);

  }

  panel.append(table);

}



function renderCohortMetrics(){

  $('root-constraint-detail').hidden=true;

  if($('merge-history'))$('merge-history').hidden=true;if($('smc-diagnostics'))$('smc-diagnostics').hidden=true;

  const completed=job.states.filter(state=>state.metrics);

  if(!completed.length)return;

  const sum=key=>completed.reduce((total,state)=>total+Number(state.metrics[key]||0),0);

  $('metric-districts').textContent=number(sum('district_count'));

  $('metric-deviation').textContent=percent(Math.max(...completed.map(state=>state.metrics.max_deviation_percent)));

  $('metric-boundary').textContent=boundary(sum('graph_boundary_m'));

  $('metric-counties').textContent=number(sum('split_counties'));

  $('metric-time').textContent=time(completed.reduce((total,state)=>total+state.elapsed_seconds,0));

  $('metric-deviation').classList.toggle('deviation-bad',completed.some(state=>!state.metrics.balance_passed));

  $('metric-time').title='Sum of completed state elapsed times, including graph preparation and output processing.';

  $('unit-count').textContent=`${completed.length}/${job.states.length} state results · ${number(sum('units'))} assigned tracts · Optimality unproved`;

  $('map-subtitle').textContent+=` · Summary of ${completed.length} completed states`;

}

function renderNationalGrid() {

  const states = job?.config.mode === 'national' ? job.states : [...selectedStates].sort().map(code => ({code,status:'planned'}));

  $('state-tiles').innerHTML = states.map(state => `<button class="state-tile ${esc(state.status)}" data-result-state="${state.code}"><strong>${state.code}</strong><small>${esc(stateInfo(state.code)?.name)}</small><small>${state.metrics ? `${percent(state.metrics.max_deviation_percent)} deviation` : esc(state.status)}</small><small>${state.metrics ? `${state.metrics.split_counties} county splits` : `${seatCount(state.code) ?? '—'} target districts`}</small></button>`).join('');

}

function setView(next) {

  view = next; $('national-grid').hidden = view !== 'national'; $('map-container').hidden = view === 'national';

  $('district-view').classList.toggle('active', view === 'district'); $('national-view').classList.toggle('active', view === 'national');

  $('national-map-view').classList.toggle('active', view === 'national-map');

  renderHeader(); if(job)renderJob(); if (view === 'national') renderNationalGrid(); else if (view === 'national-map') loadNationalGeometry(); else { if(mapGeoJSON?.state !== selectedCode)loadGeometry(selectedCode,job?.config.year || $('year').value); resizeMap(); }

}

async function selectResult(code) {

  selectedCode = code; selectedDistrict = null; setView('district');

  if (job) { renderJob(); await loadGeometry(code, job.config.year); } else { $('state').value = code; await loadGeometry(code, $('year').value); }

}

async function refreshHistory() {

  history = await api('/api/runs'); $('history-count').textContent = history.length;

  $('history-list').innerHTML = history.length ? history.map(run => `<div class="history-item" role="button" tabindex="0" data-history-id="${esc(run.id)}"><time>${new Date(run.created_unix*1000).toLocaleString()}</time><span class="badge ${esc(run.status)}">${esc(run.status)}</span><strong>${esc(run.name)}</strong><p>${run.state_count} state${run.state_count === 1 ? '' : 's'} · ${esc(run.year)} · ${esc(structureNames[run.structure] || run.structure)}</p><p>${esc(run.weights)} · ${esc(searchNames[run.search] || run.search)}</p></div>`).join('') : '<div class="empty-results">Your experiments will appear here.</div>';

  const selected = $('compare-run').value;

  $('compare-run').innerHTML = '<option value="">Choose an earlier run</option>' + history.filter(run => run.id !== job?.id && run.completed).map(run => `<option value="${esc(run.id)}">${esc(run.name)} · ${run.year} · ${run.completed} states</option>`).join('');

  if (history.some(run => run.id === selected)) $('compare-run').value = selected;

}

async function openHistory(id) {

  try {

    const saved = await api(`/api/runs/${id}`), configuration = saved.config;
    if(browserWasm)$('experiment-options').open=true;

    for (const key of ['name','year','chamber','structure','weights','search','seed','seeds','steps','iterations']) $(key).value = configuration[key];

    $('sa-steps').value=configuration.sa_steps_per_tract??10;$('sa-factor').value=configuration.sa_t0_factor??0.01;$('sa-final').value=configuration.sa_t_final??0.0001;

    $('flow-repair').value=configuration.flow_repair??'bfs';

    electionSequence++;electionLoading=false;electionInputs=structuredClone(configuration.elections??Object.create(null));$('proportional-eta').value=configuration.proportional_eta??1.1;renderElectionInputs();
    $('election-csv-state').value=configuration.states[0];$('election-csv-year').value=electionInputs[configuration.states[0]]?.election_year??'';
    partisanSequence++;partisanLoading=false;partisanInputs=structuredClone(configuration.partisans??Object.create(null));$('dem-threshold').value=percentageInput(configuration.dem_threshold??.55);$('rep-threshold').value=percentageInput(configuration.rep_threshold??.45);renderPartisanInputs();
    $('partisan-tsv-state').value=configuration.states[0];
    characterSequence++;characterLoading=false;characterInputs=structuredClone(configuration.characters??Object.create(null));$('character-alpha').value=configuration.character_alpha??.5;$('character-csv-state').value=configuration.states[0];$('character-data-year').value=configuration.characters?.[configuration.states[0]]?.data_year??configuration.year;renderCharacterInputs();
    demographicInputs=structuredClone(configuration.demographics??Object.create(null));$('vra-threshold').value=(configuration.vra_threshold??0.5)*100;$('vra-weight').value=(configuration.w_vra??0.4)*100;$('demographic-csv-state').value=configuration.states[0];$('demographic-csv-basis').value=demographicInputs[configuration.states[0]]?.basis??'total-population';renderDemographicInputs();
    $('pt-replicas').value=configuration.pt_replicas??4;$('pt-swap').value=configuration.pt_swap_interval??10;$('pt-hot').value=(configuration.pt_hot_tol??configuration.balance_tolerance/100*4)*100;
    $('burst-length').value=configuration.burst_length??20;$('n-bursts').value=configuration.n_bursts??Math.ceil(configuration.steps/20);
    $('metis-objective').value=configuration.metis_objective??'cut';$('metis-trials').value=configuration.metis_trials??1;
    $('area-init').value=configuration.area_init??'ratio-optimal';
    $('compact-slack').value=(configuration.compact_epsilon??0.05)*100;
    $('mka-orientations').value=configuration.mka_orientations??36;$('mka-metric').value=configuration.mka_metric??'reock';

    $('cvd-iters').value=configuration.cvd_iters??50;$('cvd-metric').value=configuration.cvd_metric??'geographic';

    $('smc-particles').value=configuration.smc_particles??5000;$('smc-resample').value=configuration.smc_resample_threshold??0.5;

    $('state').value = configuration.states[0]; $('districts').value = configuration.districts ?? '';

    $('percentile').value = configuration.percentile * 100; $('alpha').value = configuration.alpha_county;

    $('tolerance').value = configuration.balance_tolerance; $('area-swing').value = configuration.area_swing; $('timeout').value = configuration.timeout_seconds;

    selectedStates = new Set(configuration.states); updateControls(); setMode(configuration.mode);

    job = saved; selectedCode = job.states.find(state => state.metrics)?.code || job.states[0].code; selectedDistrict = null;

    clearResult(); renderJob(); $('history').hidden = true; await refreshHistory();

    setView(job.config.mode === 'national' ? 'national' : 'district'); await loadGeometry(selectedCode,job.config.year);

    if(browserWasm)autosaveLabProject();

  }

  catch (error) { toast(error.message); }

}

async function updateComparison() {

  const id = $('compare-run').value, revision = ++comparisonRequest;

  if (!id || !job) return;

  try {

    const other = await api(`/api/runs/${id}`); if (revision !== comparisonRequest) return;

    if ([job,other].some(run => !catalog.search_compatibility[run.config.structure]?.includes(run.config.search))) { $('comparison').textContent = 'A historical run requested a search that this structure does not execute. Choose supported runs to compare executed search methods.'; return; }

    const current = job.states.find(state => state.code === selectedCode), previous = other.states.find(state => state.code === selectedCode);

    if (!current?.metrics || !previous?.metrics) { $('comparison').textContent = 'Both experiments need a completed result for this state.'; return; }

    if (job.config.year !== other.config.year || job.config.chamber !== other.config.chamber || current.metrics.district_count !== previous.metrics.district_count) { $('comparison').textContent = 'These runs use different years, chambers, or district counts. Select matching runs for a direct comparison.'; return; }

    if(current.metrics.graph_sha256 !== previous.metrics.graph_sha256 || current.metrics.geoid_join_sha256 !== previous.metrics.geoid_join_sha256){$('comparison').textContent='These runs use different graph or unit-join inputs. Their metrics are not a direct comparison of search methods.';return;}

    if (JSON.stringify(current.metrics.engine_provenance) !== JSON.stringify(previous.metrics.engine_provenance)) { $('comparison').textContent = 'These runs use different engine builds. Compare runs from the same build to isolate configuration changes; individual metrics remain available in each record.'; return; }

    const rows = [['Max population deviation','max_deviation_percent',percent],['Graph boundary including bridges','graph_boundary_m',boundary],['Split counties','split_counties',number],['Elapsed time','elapsed_seconds',time]].map(([label,key,format]) => {

      const a = key === 'elapsed_seconds' ? current.elapsed_seconds : current.metrics[key], b = key === 'elapsed_seconds' ? previous.elapsed_seconds : previous.metrics[key];

      return `<tr><td>${label}</td><td>${format(b)}</td><td>${format(a)}</td><td>${a-b > 0 ? '+' : a-b < 0 ? '−' : ''}${format(Math.abs(a-b))}</td></tr>`;

    }).join('');

    $('comparison').className = '';

    $('comparison').innerHTML = `<p class="help" style="padding:0 16px">${esc(selectedCode)} · ${esc(other.config.name)} → ${esc(job.config.name)}. Different structures, weights, or searches are different experimental rules. A lower metric alone does not establish a universally better plan. Total elapsed time includes graph preparation; warm and cold runs are not directly comparable for engine speed.</p><div class="table-scroll"><table><thead><tr><th>Metric</th><th>Earlier run</th><th>This run</th><th>Change</th></tr></thead><tbody>${rows}</tbody></table></div>`;

  } catch (error) { $('comparison').textContent = error.message; }

}



function districtColor(district) { return district ? palette[(district-1)%palette.length] : '#d0dfd3'; }

function featureColor(feature) {

  const district = feature.properties.district;
  if($('color-by').value==='root'){
    const root=job?.states.find(state=>state.code===(feature.properties.state??selectedCode))?.metrics?.root_split;
    return root&&district?(district<=root.left_districts?'#608f82':'#9386ad'):'#d0dfd3';
  }
  if($('color-by').value==='minority'){
    const value=job?.config.demographics?.[feature.properties.state??selectedCode]?.minority_fractions[feature.properties.geoid];
    return Number.isFinite(value)?`hsl(270 38% ${95-value*60}%)`:'#d0dfd3';
  }

  if ($('color-by').value === 'county') { const code = Number(feature.properties.county.slice(2)); return palette[Math.floor(code/2)%palette.length]; }

  if ($('color-by').value === 'deviation' && job) {

    const result = job.states.find(state => state.code === feature.properties.state), d = result?.metrics?.districts.find(d => d.district === district);

    if (d) { const intensity = Math.min(1,Math.abs(d.deviation_percent)/Math.max(job.config.balance_tolerance,0.01)); return d.deviation_percent >= 0 ? `hsl(17 40% ${87-intensity*30}%)` : `hsl(171 25% ${87-intensity*35}%)`; }

  }

  return districtColor(district);

}

function renderLegend() {

  renderMinorityDetails();
  if($('color-by').value==='root'){$('legend').textContent='Root split · green: left district labels · purple: right district labels · no root split: neutral.';return;}
  if($('color-by').value==='minority'){$('legend').textContent='Minority fraction per tract · pale 0% → purple 100% · missing data neutral. Fractions use the declared population basis.';return;}

  if(view==='national-map'){$('legend').innerHTML='<span>Assigned districts are colored. States without completed results remain neutral. Click a state to inspect it.</span>';return;}

  const result = job?.states.find(state => state.code === selectedCode);

  if ($('color-by').value === 'deviation' && result?.metrics) { $('legend').innerHTML = '<span><i style="background:#608f82"></i>Below ideal population</span><span><i style="background:#c7927e"></i>Above ideal population</span>'; return; }

  if ($('color-by').value === 'county') { $('legend').innerHTML = '<span>County colors repeat. Click a tract to inspect its county FIPS.</span>'; return; }

  $('legend').innerHTML = result?.metrics ? result.metrics.districts.map(d => `<button data-district="${d.district}"><i style="background:${districtColor(d.district)}"></i>District ${d.district}${selectedDistrict === d.district ? ' · selected' : ''}</button>`).join('') : '<span><i style="background:#d0dfd3"></i>Census tract geometry · no plan assigned</span>';

}

function renderMinorityDetails(){
  let panel=$('minority-details');
  if(!panel){panel=document.createElement('details');panel.id='minority-details';panel.className='demographic-details';$('root-constraint-detail').after(panel);}
  const input=job?.config.demographics?.[selectedCode],state=job?.states.find(s=>s.code===selectedCode),e=state?.metrics?.structure_evidence;
  const recom=job?.config.search==='vra-recom';
  panel.replaceChildren();panel.hidden=(!recom&&!input?.counts)||!input||!state?.metrics||!mapGeoJSON||mapGeoJSON.state!==selectedCode||mapGeoJSON.year!==job.config.year;if(panel.hidden)return;
  function appendTable(table){const wrapper=document.createElement('div');wrapper.className='table-scroll';wrapper.append(table);panel.append(wrapper);}
  if(recom){
  const title=document.createElement('summary');title.textContent='District minority fractions · unweighted tract means';panel.append(title);
  const note=document.createElement('p');note.textContent=`${input.basis} · ${input.source_label}. Each tract contributes equally. These values do not aggregate population counts or establish legal VRA compliance.`;panel.append(note);
  const sums=Array(state.metrics.district_count).fill(0),counts=Array(sums.length).fill(0);
  for(const feature of mapGeoJSON.features){const d=feature.properties.district;if(d){sums[d-1]+=input.minority_fractions[feature.properties.geoid];counts[d-1]++;}}
  const table=document.createElement('table'),head=document.createElement('tr');
  for(const text of ['District','Tract mean','Initially protected']){const cell=document.createElement('th');cell.textContent=text;head.append(cell);}table.append(head);
  sums.forEach((sum,i)=>{const mean=counts[i]?sum/counts[i]:null,row=document.createElement('tr');for(const value of [i+1,percent(mean==null?null:mean*100),e?.run.protected_districts.includes(i+1)?'Yes':'No']){const cell=document.createElement('td');cell.textContent=String(value);row.append(cell);}table.append(row);});appendTable(table);
  }
  if(input.counts){
    if(!recom){const title=document.createElement('summary');title.textContent='District demographic counts';panel.append(title);}
    const note=document.createElement('p');note.textContent=`Count aggregation · ${input.basis}. Minority count divided by total count. Search uses tract fractions; these descriptive totals do not establish legal compliance or authenticate the source.`;panel.append(note);
    try{
      const graph={state:selectedCode,year:job.config.year,geoids:mapGeoJSON.features.map(f=>f.properties.geoid)},assignments=Object.fromEntries(mapGeoJSON.features.map(f=>[f.properties.geoid,f.properties.district]));
      const rows=districtCountShares(graph,assignments,input,state.metrics.district_count),table=document.createElement('table'),head=document.createElement('tr');
      for(const text of ['District','Total count','Minority count','Count share']){const cell=document.createElement('th');cell.textContent=text;head.append(cell);}table.append(head);
      for(const d of rows){const row=document.createElement('tr');for(const value of [d.district,d.total.toLocaleString(undefined,{maximumFractionDigits:6}),d.minority.toLocaleString(undefined,{maximumFractionDigits:6}),percent(d.share===null?null:d.share*100)]){const cell=document.createElement('td');cell.textContent=String(value);row.append(cell);}table.append(row);}appendTable(table);
    }catch(error){const note=document.createElement('p');note.textContent=error.message;panel.append(note);}
  }
}

async function loadGeometry(code, year) {

  if (view === 'national-map') return;

  const revision = ++geometryRequest; $('map-loading').hidden = false; $('map-tooltip').hidden = true;

  const selectedJob = job?.id, ready = job?.states.find(state => state.code === code)?.metrics;

  try {

    const url = selectedJob && ready ? `/api/runs/${selectedJob}/${code}/map` : `/api/geometry?state=${code}&year=${year}`;

    const geojson = await api(url);

    if (revision !== geometryRequest || code !== selectedCode) return;

    mapGeoJSON = geojson; buildPaths(geojson,code); $('map-empty').hidden = true;

    $('map-caption').textContent = `${year} TIGER tracts · ${ready ? 'Engine assignments' : 'Input preview'} · Display geometry simplified`;

    if (!ready) $('unit-count').textContent = `${number(geojson.features.length)} Census tracts · No plan assigned`;

    $('download-map').disabled = !ready; renderLegend(); resizeMap();

  } catch (error) {

    if (revision !== geometryRequest) return;

    mapFeatures = []; mapGeoJSON = null; drawMap(); $('map-empty').hidden = false;

    $('map-empty').querySelector('h3').textContent = 'Geometry unavailable'; $('map-empty').querySelector('p').textContent = error.message;

  } finally { if (revision === geometryRequest) $('map-loading').hidden = true; }

}

function buildPaths(geojson,code) {

  const coordinates = geojson.features.flatMap(feature => feature.geometry.coordinates.flatMap(polygon => polygon.flat()));

  const centerLat = coordinates.reduce((sum,c) => sum+c[1],0)/coordinates.length;

  const cos = Math.cos(centerLat*Math.PI/180);

  const albers = coordinate => {

    const r=Math.PI/180,n=(Math.sin(29.5*r)+Math.sin(45.5*r))/2,c=Math.cos(29.5*r)**2+2*n*Math.sin(29.5*r);

    const rho=Math.sqrt(c-2*n*Math.sin(coordinate[1]*r))/n,origin=Math.sqrt(c-2*n*Math.sin(23*r))/n,theta=n*(coordinate[0]+96)*r;

    return [rho*Math.sin(theta),rho*Math.cos(theta)-origin];

  };

  const insetBounds={};

  if(code==='US')for(const feature of geojson.features){const state=feature.properties.state;if(!['AK','HI'].includes(state))continue;const b=insetBounds[state]||=[Infinity,Infinity,-Infinity,-Infinity];for(const polygon of feature.geometry.coordinates)for(const ring of polygon)for(const raw of ring){const x=state==='AK'&&raw[0]>0?raw[0]-360:raw[0],y=-raw[1];b[0]=Math.min(b[0],x);b[1]=Math.min(b[1],y);b[2]=Math.max(b[2],x);b[3]=Math.max(b[3],y);}}

  const project = (coordinate,state) => {

    if(code!=='US')return [(code === 'AK' && coordinate[0] > 0 ? coordinate[0]-360 : coordinate[0])*cos,-coordinate[1]];

    if(insetBounds[state]){const [x1,y1,x2,y2]=insetBounds[state],box=state==='AK'?[-.5,.05,-.27,.19]:[-.20,.09,-.08,.18],x=state==='AK'&&coordinate[0]>0?coordinate[0]-360:coordinate[0];const s=Math.min((box[2]-box[0])/(x2-x1),(box[3]-box[1])/(y2-y1));return [box[0]+(x-x1)*s,box[1]+(-coordinate[1]-y1)*s];}

    return albers(coordinate);

  };

  let bounds = [Infinity,Infinity,-Infinity,-Infinity];

  mapFeatures = geojson.features.map(feature => {

    const path = new Path2D();

    for (const polygon of feature.geometry.coordinates) for (const ring of polygon) {

      ring.forEach((coordinate,index) => { const [x,y] = project(coordinate,feature.properties.state); if(index===0) path.moveTo(x,y); else path.lineTo(x,y); bounds=[Math.min(bounds[0],x),Math.min(bounds[1],y),Math.max(bounds[2],x),Math.max(bounds[3],y)]; }); path.closePath();

    }

    return {path,properties:feature.properties};

  });

  base = {bounds}; zoom = 1; pan = [0,0];

}

function resizeMap() {

  if (view === 'national') return;

  const rectangle = $('map-container').getBoundingClientRect(), dpr = window.devicePixelRatio || 1;

  if (!rectangle.width || !rectangle.height) return;

  canvas.width = rectangle.width*dpr; canvas.height = rectangle.height*dpr; drawMap();

}

function transform() {

  if (!base) return null;

  const w=canvas.clientWidth,h=canvas.clientHeight,[x1,y1,x2,y2]=base.bounds;

  const fit=Math.min((w-80)/(x2-x1),(h-70)/(y2-y1));

  return {s:fit*zoom,x:w/2-(x1+x2)/2*fit*zoom+pan[0],y:h/2-(y1+y2)/2*fit*zoom+pan[1]};

}

function drawMap() {

  ctx.setTransform(1,0,0,1,0,0);ctx.clearRect(0,0,canvas.width,canvas.height);

  const t = transform(); if (!t || !mapFeatures.length) return;

  const dpr = window.devicePixelRatio || 1;ctx.setTransform(t.s*dpr,0,0,t.s*dpr,t.x*dpr,t.y*dpr);

  for (const feature of mapFeatures) {

    ctx.globalAlpha=selectedDistrict && feature.properties.district !== selectedDistrict ? .3 : .96;

    ctx.fillStyle=featureColor(feature);ctx.fill(feature.path,'evenodd');

    ctx.strokeStyle=selectedDistrict === feature.properties.district ? '#2f5b47' : '#ffffffa0';ctx.lineWidth=.35/t.s;ctx.stroke(feature.path);

  }

  ctx.globalAlpha=1;ctx.setTransform(1,0,0,1,0,0);

}

function hit(event) {

  const t=transform();if(!t)return null;const rect=canvas.getBoundingClientRect();const x=(event.clientX-rect.left-t.x)/t.s,y=(event.clientY-rect.top-t.y)/t.s;

  return mapFeatures.findLast(feature => ctx.isPointInPath(feature.path,x,y,'evenodd'));

}

function showTooltip(event) {

  if (drag) return;

  const feature=hit(event), tooltip=$('map-tooltip');tooltip.hidden=!feature;if(!feature)return;

  const d=job?.states.find(state=>state.code===feature.properties.state)?.metrics?.districts.find(d=>d.district===feature.properties.district);

  tooltip.innerHTML=`<strong>${esc(feature.properties.state)} · ${feature.properties.district ? `District ${feature.properties.district}` : 'Unassigned Census tract'}</strong>GEOID ${esc(feature.properties.geoid)}<br>County FIPS ${esc(feature.properties.county)}${d ? `<br>${number(d.population)} district population<br>${percent(d.deviation_percent)} deviation` : ''}`;

  const rect=canvas.getBoundingClientRect();tooltip.style.left=`${Math.min(event.clientX-rect.left+13,rect.width-235)}px`;tooltip.style.top=`${Math.max(8,Math.min(event.clientY-rect.top+13,rect.height-110))}px`;

}

function highlightDistrict(district) {selectedDistrict=selectedDistrict===district?null:district;renderLegend();drawMap();}

function zoomMap(factor) {zoom=Math.min(12,Math.max(.7,zoom*factor));drawMap();}

function download(value,name) {const url=URL.createObjectURL(new Blob([JSON.stringify(value,null,2)],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download=name;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}



$('experiment-form').addEventListener('submit',submit);

document.querySelectorAll('[data-mode]').forEach(button=>button.addEventListener('click',()=>setMode(button.dataset.mode)));

for (const id of ['structure','weights','search','percentile']) $(id).addEventListener('input',()=>{updateControls();if(precomputed)updateAvailability();});

if(precomputed)$('experiment-form').addEventListener('input',updateAvailability);

for (const id of ['year','chamber']) $(id).addEventListener('change',()=>{job=null;clearResult();updateAvailability();if(mode==='state')loadGeometry($('state').value,$('year').value);else renderNationalGrid();});

$('state').addEventListener('change',()=>{job=null;selectedCode=$('state').value;selectedDistrict=null;clearResult();updateAvailability();loadGeometry(selectedCode,$('year').value);});

for (const id of ['state-filter','scale-filter']) $(id).addEventListener('input',renderChecklist);

$('state-checklist').addEventListener('change',event=>{const code=event.target.dataset.code;if(code){if(event.target.checked)selectedStates.add(code);else selectedStates.delete(code);updateAvailability();renderNationalGrid();}});

document.querySelectorAll('[data-batch]').forEach(button=>button.addEventListener('click',()=>{selectedStates=new Set(button.dataset.batch==='all'?catalog.states.filter(state=>yearInfo(state.code)?.available&&seatCount(state.code)).map(state=>state.code):button.dataset.batch==='scale'?['RI','IA','NC'].filter(code=>yearInfo(code)?.available&&seatCount(code)):[]);updateAvailability();renderNationalGrid();}));

$('cancel').addEventListener('click',async()=>{try{await api(`/api/runs/${job.id}/cancel`,{method:'POST',body:'{}'});toast(browserWasm?'Browser Worker terminated. Completed state results are retained.':'Cancellation requested. Active engine execution will stop; graph preparation completes first.');await poll();}catch(error){toast(error.message);}});

$('district-view').addEventListener('click',()=>setView('district'));$('national-view').addEventListener('click',()=>setView('national'));$('national-map-view').addEventListener('click',()=>setView('national-map'));

$('reset-view').addEventListener('click',()=>{zoom=1;pan=[0,0];selectedDistrict=null;drawMap();renderLegend();});

$('zoom-in').addEventListener('click',()=>zoomMap(1.25));$('zoom-out').addEventListener('click',()=>zoomMap(.8));

$('color-by').addEventListener('change',()=>{drawMap();renderLegend();});

canvas.addEventListener('wheel',event=>{event.preventDefault();zoomMap(event.deltaY<0?1.12:.89);},{passive:false});

canvas.addEventListener('pointerdown',event=>{drag={x:event.clientX,y:event.clientY,pan:[...pan],moved:false};canvas.setPointerCapture(event.pointerId);$('map-tooltip').hidden=true;});

canvas.addEventListener('pointermove',event=>{if(drag){const dx=event.clientX-drag.x,dy=event.clientY-drag.y;drag.moved ||= Math.abs(dx)+Math.abs(dy)>4;pan=[drag.pan[0]+dx,drag.pan[1]+dy];drawMap();}else showTooltip(event);});

canvas.addEventListener('pointerup',event=>{const moved=drag?.moved;drag=null;if(!moved){const feature=hit(event);if(view==='national-map'&&feature){selectResult(feature.properties.state);return;}if(feature?.properties.district)highlightDistrict(feature.properties.district);showTooltip(event);}});

canvas.addEventListener('pointercancel',()=>drag=null);canvas.addEventListener('pointerleave',()=>$('map-tooltip').hidden=true);

document.addEventListener('click',event=>{const result=event.target.closest('[data-result-state]');if(result)selectResult(result.dataset.resultState);const district=event.target.closest('[data-district]');if(district)highlightDistrict(Number(district.dataset.district));const item=event.target.closest('[data-history-id]');if(item)openHistory(item.dataset.historyId);});

$('history-list').addEventListener('keydown',event=>{if(event.key==='Enter'||event.key===' '){const item=event.target.closest('[data-history-id]');if(item){event.preventDefault();openHistory(item.dataset.historyId);}}});

$('history-toggle').addEventListener('click',()=>{$('history').hidden=!$('history').hidden;refreshHistory().catch(error=>toast(error.message));});$('close-history').addEventListener('click',()=>$('history').hidden=true);

document.querySelectorAll('[data-result-tab]').forEach(button=>button.addEventListener('click',()=>{activeTab=button.dataset.resultTab;document.querySelectorAll('[data-result-tab]').forEach(b=>b.classList.toggle('active',b===button));for(const name of ['districts','log','compare'])$(`results-${name}`).hidden=name!==activeTab;}));

$('compare-run').addEventListener('change',updateComparison);

$('download-config').addEventListener('click',()=>{if(job)download(job,`${job.id}_record.json`);});$('download-map').addEventListener('click',()=>{if(mapGeoJSON&&job)download(mapGeoJSON,`${job.id}_${mapGeoJSON.state || selectedCode}.geojson`);});

new ResizeObserver(resizeMap).observe($('map-container'));



async function poll() {

  if(polling||!job||!['queued','running'].includes(job.status))return;

  polling=true;const id=job.id;

  try {

    const next=await api(`/api/runs/${id}`);if(job?.id!==id)return;

    const before=job.states.find(state=>state.code===selectedCode)?.metrics,beforeCount=job.states.filter(state=>state.metrics).length;job=next;renderJob();updateAvailability();

    if(view==='national-map'&&job.states.filter(state=>state.metrics).length!==beforeCount)await loadNationalGeometry();

    else if(!before&&job.states.find(state=>state.code===selectedCode)?.metrics)await loadGeometry(selectedCode,job.config.year);

    if(!['queued','running'].includes(job.status)){await refreshHistory();toast(`Experiment ${job.status}. Individual state results are saved.`);}

    if(browserWasm&&(job.states.filter(state=>state.metrics).length!==beforeCount||!['queued','running'].includes(job.status)))autosaveLabProject();

  }catch(error){toast(error.message);}finally{polling=false;}

}

async function start() {

  try {if(precomputed)staticCatalog=await loadStaticCatalog();if(browserWasm)staticCatalog=await (await import('./wasm-catalog.js')).loadWasmCatalog();catalog=await api('/api/catalog');populate();if(browserWasm)await setupLabProjects();await refreshHistory();renderHeader();loadGeometry(selectedCode,$('year').value);if(!precomputed)setInterval(poll,1500);}

  catch(error){$('connection').textContent='Connection unavailable';$('form-error').textContent=error.message;$('form-error').hidden=false;$('run').disabled=true;}

}

start();



async function setupLabProjects(){

  labProjectTools=await import('./laboratory-project.js');

  const section=document.createElement('section');section.className='lab-project-controls';

  const heading=document.createElement('h2');heading.textContent='Local project';section.append(heading);

  const save=document.createElement('button');save.id='save-lab-project';save.type='button';save.textContent='Save project';save.disabled=true;section.append(save);

  const label=document.createElement('label');label.textContent='Open project';

  const open=document.createElement('input');open.type='file';open.id='open-lab-project';open.accept='.bisect,.json';label.append(open);section.append(label);

  const restore=document.createElement('button');restore.id='restore-lab-project';restore.type='button';restore.textContent='Restore autosave';restore.disabled=true;section.append(restore);

  const status=document.createElement('p');status.id='lab-project-status';status.setAttribute('role','status');status.textContent='Save settings and results together. Opening a project never runs the engine.';section.append(status);

  document.querySelector('.configuration').append(section);

  document.querySelector('#history > p').textContent='Published runs remain available. Local runs can be saved as projects; the latest experiment is autosaved in this browser.';

  for(const type of ['input','change','click'])document.addEventListener(type,()=>projectRevision++,{capture:true});

  save.onclick=async()=>{try{const project=await labProjectTools.createLabProject(staticCatalog,job?.id);download(project,(project.experiment.config.name.replace(/[^a-z0-9_-]+/gi,'-')||'experiment')+'.bisect');status.textContent='Project download requested. Keep the file as a portable backup.';}catch(error){status.textContent=error.message;}};

  if(browserWasm){
    let preparedAudit;
    const exportButton=document.createElement('button');exportButton.id='export-audit-project';exportButton.type='button';exportButton.textContent='Prepare selected state for auditing';section.append(exportButton);
    const auditDownload=document.createElement('button');auditDownload.id='download-audit-project';auditDownload.type='button';auditDownload.textContent='Download audit project';auditDownload.disabled=true;section.append(auditDownload);
    const workbenchLink=document.createElement('a');workbenchLink.href='./toolkit/';workbenchLink.textContent='Open practitioner workbench';section.append(workbenchLink);
    const help=document.createElement('p');help.textContent='Select a completed state on the map, export its plan and context, then open the downloaded project in the workbench. Load an explicit legal profile before auditing. National runs export one selected state at a time.';section.append(help);
    const practitioner=document.createElement('details');practitioner.id='practitioner-tools';practitioner.className='advanced';const title=document.createElement('summary');title.textContent='Practitioner tools';practitioner.append(title);section.append(practitioner);
    for(const element of [exportButton,auditDownload,workbenchLink,help])practitioner.append(element);
    exportButton.onclick=async()=>{
      exportButton.disabled=true;
      preparedAudit=null;auditDownload.disabled=true;
      const expectedJob=job?.id,expectedCode=selectedCode;
      try{
        status.textContent=`Checking ${expectedCode} assignments and preparing native audit files…`;
        const {project}=await staticCatalog.exportPractitionerProject(expectedJob,expectedCode);
        if(job?.id!==expectedJob||selectedCode!==expectedCode)throw new Error('Selection changed during export. Select the state and export again.');
        preparedAudit={project,jobId:expectedJob,code:expectedCode};auditDownload.disabled=false;
        status.textContent=`${expectedCode} audit project ready. Download it, then open it in the practitioner workbench and supply a legal profile. Generation provenance remains unverified.`;
      }catch(error){status.textContent=error.message;}
      finally{exportButton.disabled=false;}
    };
    auditDownload.onclick=()=>{
      if(!preparedAudit||job?.id!==preparedAudit.jobId||selectedCode!==preparedAudit.code){preparedAudit=null;auditDownload.disabled=true;status.textContent='Selection changed. Prepare the selected state again.';return;}
      download(preparedAudit.project,`${preparedAudit.jobId}_${preparedAudit.code}_audit.bisect`);
      status.textContent=`Audit project download requested for ${preparedAudit.code}. Open it in the practitioner workbench.`;
    };
  }

  async function load(read){

    projectImport?.abort();projectImport=new AbortController();

    const sequence=++projectSequence, revision=projectRevision, signal=projectImport.signal;

    try{

      if(staticCatalog.active)throw new Error('Finish or cancel the active experiment first.');

      status.textContent='Validating project and assignment evidence…';

      const project=await read(signal);if(!project)throw new Error('No autosaved experiment found.');

      const restored=await staticCatalog.restoreProject(project,{signal,isCurrent:()=>sequence===projectSequence&&revision===projectRevision});

      await openHistory(restored.id);status.textContent='Opened project. Assignment evidence checked; engine provenance remains unverified.';

    }catch(error){if(sequence===projectSequence)status.textContent=error.message;}

  }

  open.onchange=async()=>{const file=open.files[0];if(file)await load(signal=>labProjectTools.readLabProjectFile(file,{signal}));open.value='';};

  restore.onclick=()=>load(()=>labProjectTools.readLabAutosave());

  labProjectTools.readLabAutosave().then(project=>{if(project){restore.disabled=false;status.textContent='An autosaved experiment is available. Restore it to reopen its settings and results.';}}).catch(()=>{status.textContent='Browser autosave unavailable. Save project to keep a backup.';});

}

function autosaveLabProject(){

  if(!labProjectTools||!job)return;

  const id=job.id,sequence=++projectSaveSequence;

  projectSaveQueue=projectSaveQueue.catch(()=>{}).then(async()=>{

    if(sequence!==projectSaveSequence)return;

    // Published catalog runs are already persistent; only browser executions

    // and imported browser projects have portable assignment outputs.

    if(!id.startsWith('wasm-')&&!staticCatalog.jobs.get(id)?.restored_from_project)return;

    const project=await labProjectTools.createLabProject(staticCatalog,id);await labProjectTools.writeLabAutosave(project);

    $('restore-lab-project').disabled=false;

    if(sequence===projectSaveSequence)$('lab-project-status').textContent='Latest experiment autosaved in this browser. Save project for a portable backup.';

  }).catch(error=>{$('lab-project-status').textContent='Autosave unavailable: '+error.message+' Save project to keep a backup.';});

}



async function loadNationalGeometry() {

  const revision=++geometryRequest, currentId=job?.id, year=job?.config.year || $('year').value;

  const codes=job?.config.mode==='national'?job.config.states:[...selectedStates].sort();

  $('map-loading').hidden=false;selectedDistrict=null;

  const features=[],failures=[];

  for(const code of codes){

    if(revision!==geometryRequest||view!=='national-map')return;

    $('map-loading').textContent=`Loading selected states · ${features.length ? codes.indexOf(code) : 0}/${codes.length}`;

    const ready=job?.states.find(state=>state.code===code)?.metrics;

    try{const collection=await api(currentId&&ready?`/api/runs/${currentId}/${code}/map`:`/api/geometry?state=${code}&year=${year}`);features.push(...collection.features);}catch(error){failures.push(code);}

  }

  if(revision!==geometryRequest||view!=='national-map')return;

  $('map-loading').hidden=true;$('map-loading').textContent='Loading geometry…';

  if(!features.length){mapFeatures=[];drawMap();$('map-empty').hidden=false;$('map-empty').querySelector('h3').textContent='Choose states with available geometry';return;}

  mapGeoJSON={type:'FeatureCollection',state:'US',year,features};buildPaths(mapGeoJSON,'US');$('map-empty').hidden=true;resizeMap();

  $('map-caption').textContent=`Selected state coverage · Albers projection · Alaska and Hawaii inset${failures.length?` · unavailable: ${failures.join(', ')}`:''}`;

  $('legend').innerHTML='<span>Assigned districts are colored. States without completed results remain neutral. Click a state to inspect it.</span>';

  $('download-map').disabled=!job?.states.some(state=>state.metrics);

}
