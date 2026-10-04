import {validateJsonTree,readJsonFile,readValidatedFile} from './project.js';
const exact=(value,fields)=>value&&typeof value==='object'&&!Array.isArray(value)&&Object.keys(value).length===fields.length&&fields.every(k=>Object.hasOwn(value,k));
export function validateElectionInput(input,graph){
  validateJsonTree(input);
  if(!exact(input,['schema_version','state','year','election_year','source_label','counts'])||input.schema_version!=='bisect-election-counts-v1'||typeof input.state!=='string'||! /^[A-Z]{2}$/.test(input.state)||typeof input.year!=='string'||! /^(2000|2010|2020)$/.test(input.year)||typeof input.election_year!=='string'||! /^\d{4}$/.test(input.election_year)||typeof input.source_label!=='string'||!input.source_label.trim()||new TextEncoder().encode(input.source_label).length>200)throw new Error('Invalid election input schema, scope or source label.');
  if(!input.counts||typeof input.counts!=='object'||Array.isArray(input.counts)||!Object.keys(input.counts).length||Object.entries(input.counts).some(([id,c])=>! /^\d{11}$/.test(id)||!exact(c,['democratic','two_party'])||[c.democratic,c.two_party].some(v=>!Number.isFinite(v)||v<0||Object.is(v,-0)||v>Number.MAX_SAFE_INTEGER)||c.democratic>c.two_party))throw new Error('Supply complete finite nonnegative Democratic and two-party counts, with Democratic counts no greater than two-party counts.');
  if(graph&&(input.state!==graph.state||input.year!==graph.year||Object.keys(input.counts).length!==graph.geoids.length||graph.geoids.some(id=>!Object.hasOwn(input.counts,id))))throw new Error('Election counts must match the graph state, Census year and complete GEOID coverage.');
  return input;
}
export async function readElectionFile(file,{csvMetadata,wasmSha256,signal}={}){
  if(/\.csv$/i.test(file?.name||'')){
    if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>8*1024*1024)throw new Error('Select a nonempty election CSV of at most 8 MiB.');
    if(!csvMetadata||! /^[a-f0-9]{64}$/.test(wasmSha256||''))throw new Error('CSV import needs explicit state and years and a verified browser engine.');
    return validateElectionInput(await readValidatedFile({size:file.size,file,metadata:csvMetadata,sha256:wasmSha256},new URL('./election-csv-worker.js',import.meta.url),{signal}));
  }
  return validateElectionInput(await readJsonFile(file,{signal}));
}
export async function electionIdentity(input){
  validateElectionInput(input);
  const header=new TextEncoder().encode(`BISECT_ELECTION_COUNTS_V1\0${input.state}\0${input.year}\0${input.election_year}\0`),ids=Object.keys(input.counts).sort();
  const bytes=new Uint8Array(header.length+ids.length*27);bytes.set(header);const view=new DataView(bytes.buffer);let offset=header.length;
  for(const id of ids){bytes.set(new TextEncoder().encode(id),offset);view.setFloat64(offset+11,input.counts[id].democratic,true);view.setFloat64(offset+19,input.counts[id].two_party,true);offset+=27;}
  return [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join('');
}
const near=(actual,expected)=>typeof actual==='number'&&Number.isFinite(actual)&&Math.abs(actual-expected)<=1e-10*Math.max(1,Math.abs(expected));
const same=(a,b)=>Array.isArray(a)&&a.length===b.length&&a.every((v,i)=>v===b[i]);
const fail=()=>{throw new Error('Election execution evidence disagrees with counts, options or assignments.');};
export async function verifyElectionResult(graph,options,input,result){
  validateElectionInput(input,graph);
  const ids=graph.geoids,counts=ids.map(id=>input.counts[id]),totalD=counts.reduce((s,c)=>s+c.democratic,0),totalTwo=counts.reduce((s,c)=>s+c.two_party,0),k=options.districts;
  const evidence=result.metrics.election_input_evidence;
  if(!exact(evidence,['schema_version','counts_sha256','election_year','democratic_total','two_party_total'])||evidence.schema_version!=='bisect-election-evidence-v1'||evidence.election_year!==input.election_year||!near(evidence.democratic_total,totalD)||!near(evidence.two_party_total,totalTwo)||evidence.counts_sha256!==await electionIdentity(input))fail();
  const ledger=result.metrics.structure_evidence;
  if(k===1){if(ledger!==null)fail();return;}
  const assignment=ids.map(id=>result.assignments[id]);
  if(assignment.some(d=>!Number.isInteger(d)||d<1||d>k)||new Set(assignment).size!==k)fail();
  const sum=(groups,values)=>values.reduce((s,v,i)=>s+(groups.has(assignment[i])?v:0),0);
  if(options.structure==='proportional-bisect'){
    if(!exact(ledger,['method','ratio_basis','seat_rounding','seed_policy','seed','split_schedule','splits'])||ledger.method!=='proportional-bisect'||ledger.ratio_basis!=='democratic-votes-over-census-population'||ledger.seat_rounding!=='nearest-clamped-one-through-k-minus-one'||ledger.seed_policy!=='requested-seed-at-every-node'||String(ledger.seed)!==String(options.seed)||ledger.split_schedule!=='chosen-child-seat-counts'||!Array.isArray(ledger.splits)||ledger.splits.length!==k-1)fail();
    const records=new Map();for(const s of ledger.splits){if(!exact(s,['path','left_seats','right_seats','population','dem_votes','share','left_district_ids','right_district_ids'])||typeof s.path!=='string'||! /^[01]*$/.test(s.path)||records.has(s.path))fail();records.set(s.path,s);}
    const visited=new Set();function node(path,groups){if(groups.length===1)return;const s=records.get(path);if(!s)fail();visited.add(path);
      const left=s.left_district_ids,right=s.right_district_ids;
      if(!Array.isArray(left)||!Array.isArray(right)||left.length!==s.left_seats||right.length!==s.right_seats||!left.length||!right.length||!same([...left,...right].sort((a,b)=>a-b),groups)||new Set([...left,...right]).size!==groups.length)fail();
      const set=new Set(groups),population=sum(set,graph.population),dem=sum(set,counts.map(c=>c.democratic)),share=population>0?dem/population:.5,leftSeats=Math.max(1,Math.min(groups.length-1,Math.round(share*groups.length)));
      if(s.left_seats!==leftSeats||s.right_seats!==groups.length-leftSeats||s.population!==population||!near(s.dem_votes,dem)||!near(s.share,share))fail();
      node(path+'0',[...left].sort((a,b)=>a-b));node(path+'1',[...right].sort((a,b)=>a-b));
    }
    node('',Array.from({length:k},(_,i)=>i+1));if(visited.size!==records.size)fail();
  }else if(options.structure==='proportional-section'){
    const d=Math.max(.01,Math.min(.99,totalD/totalTwo)),quota=d*k,floor=Math.floor(quota),left=Math.max(1,Math.min(k-1,floor>0&&quota>Math.sqrt(floor*(floor+1))?floor+1:Math.max(1,floor))),right=k-left,seeds=options.search==='multi'?options.seeds:1;
    const group=new Set(Array.from({length:left},(_,i)=>i+1)),leftD=sum(group,counts.map(c=>c.democratic));
    const populationFraction=sum(group,graph.population)/graph.population.reduce((s,p)=>s+p,0),demFraction=leftD/totalD,demRightTarget=Math.max(.01,Math.min(.99,right/(2*d*k))),eta=options.proportional_eta??1.1;
    const populationOk=populationFraction<=1.001*(left/k)+1e-9&&1-populationFraction<=1.001*(right/k)+1e-9,democraticOk=demFraction<=eta*(1-demRightTarget)+1e-9&&1-demFraction<=eta*demRightTarget+1e-9;
    const cut=graph.edges.reduce((s,[u,v,length])=>s+(group.has(assignment[u])!==group.has(assignment[v])?(options.weights==='unweighted'?1:options.weights==='county'&&ids[u].slice(0,5)===ids[v].slice(0,5)?length*Math.max(1,options.alpha_county):length):0),0);
    if(!exact(ledger,['method','ratio_basis','statewide_democratic_share','share_clamp','left_seats','right_seats','seat_rounding','eta','root_population_multiplier','root_population_fraction_left','root_population_within_requested_multiplier','root_democratic_within_requested_multiplier','root_constraint_check_basis','root_cut','root_democratic_fraction_left','root_democratic_target_right','seed_policy','seeds_per_root','descendant_seeds_per_ratio','descendants'])||ledger.method!=='proportional-section'||ledger.ratio_basis!=='statewide-two-party-share'||!near(ledger.statewide_democratic_share,d)||!same(ledger.share_clamp,[.01,.99])||ledger.left_seats!==left||ledger.right_seats!==right||ledger.seat_rounding!=='geometric-mean-democratic-quota-complement-positive-children'||ledger.eta!==(options.proportional_eta??1.1)||ledger.root_population_multiplier!==1.001||!near(ledger.root_population_fraction_left,populationFraction)||ledger.root_population_within_requested_multiplier!==populationOk||ledger.root_democratic_within_requested_multiplier!==democraticOk||ledger.root_constraint_check_basis!=='census-population-and-supplied-vote-counts'||!near(ledger.root_cut,cut)||!near(ledger.root_democratic_fraction_left,leftD/totalD)||!near(ledger.root_democratic_target_right,Math.max(.01,Math.min(.99,right/(2*d*k))))||ledger.seed_policy!=='fixed-one-through-budget'||ledger.seeds_per_root!==seeds||ledger.descendant_seeds_per_ratio!==Math.min(50,seeds)||ledger.descendants!=='population-only-geosection')fail();
  }else fail();
}
