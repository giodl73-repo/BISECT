const scalar=value=>['string','number','boolean'].includes(typeof value)?String(value).slice(0,500):'—';
export function countResultTables(result) {
  if(result?.schema_version==='audit-certificate-v1'){
    const checks=Array.isArray(result.checks)?result.checks.slice(0,500):[];
    const tables=[{title:'Plan audit checks',note:'Reports the supplied profile and available evidence. Certificate/source provenance and legal compliance remain unverified.',headings:['Check','Result','Summary'],rows:checks.map(c=>[scalar(c?.name),scalar(c?.status),scalar(c?.summary)])}];
    const witnesses=checks.flatMap(c=>Array.isArray(c?.witnesses)?c.witnesses:[]).slice(0,500);
    const vra=witnesses.filter(w=>w?.type==='vra');
    if(vra.length)tables.push({title:'VRA count witnesses',note:'Native district IDs are zero based. Opportunity reporting uses a strict greater-than threshold; native total_vap/minority_vap field names also carry explicitly selected CVAP counts.',headings:['District ID','Minority group','Total count','Minority count','Share (%)','Threshold (%)','Opportunity reported'],rows:vra.map(w=>[scalar(w.district_id),scalar(w.minority_group),scalar(w.total_vap),scalar(w.minority_vap),scalar(w.minority_vap_percent),scalar(w.threshold_percent),w.is_opportunity_district===true?'Yes':w.is_opportunity_district===false?'No':'—'])});
    const missing=witnesses.filter(w=>w?.type==='missing-input');
    if(missing.length)tables.push({title:'Unavailable audit inputs',headings:['Input','Reason'],rows:missing.map(w=>[scalar(w.input),scalar(w.reason)])});
    return tables;
  }
  if(result?.schema_version==='bisect-demographic-context-v1'){
    const districts=Array.isArray(result.district_totals)?result.district_totals.slice(0,500):[];
    return [{title:'Prepared demographic count context',note:`${scalar(result.basis)} · ${scalar(result.source_label)}. Native district IDs are zero based. Use this context, an explicit reporting profile and the VRA check to generate a certificate. Imported source provenance remains unverified.`,headings:['District ID','Total count','Minority count','Count share (%)'],rows:districts.map(d=>[scalar(d?.district_id),scalar(d?.total),scalar(d?.minority),Number.isFinite(d?.share)?scalar(d.share*100):'—'])}];
  }
  const verification=result?.verification||result;
  const checks=Array.isArray(verification?.checks)?verification.checks.slice(0,500):[];
  const tables=[];
  if(result && Object.hasOwn(result,'package_id') && Object.hasOwn(result,'package_content_hash')) {
    tables.push({title:'Unit history checks',note:typeof result.error==='string'?result.error.slice(0,2000):'Package consistency checks; historical truth and imported provenance remain unverified.',headings:['Check','Result'],rows:checks.filter(check=>typeof check==='string').map(check=>[scalar(check).replaceAll('_',' '),scalar(result.status)])});
    return tables;
  }
  if(result?.schema_version==='bisect-district-aggregation-v1'){
    const districts=Array.isArray(result.district_totals)?result.district_totals.slice(0,500):[];
    tables.push({title:'District count totals',headings:['District','Sources','Counted ballots','Undervotes','Overvotes','Blank contests'],rows:districts.map(d=>[scalar(d?.district_label),scalar(d?.source_reporting_unit_count),scalar(d?.counted_ballots),scalar(d?.undervotes),scalar(d?.overvotes),scalar(d?.blank_contests)])});
    const rows=[];for(const district of districts)for(const total of Array.isArray(district?.totals)?district.totals:[]){if(rows.length>=500)break;rows.push([scalar(district?.district_label),scalar(total?.selection_id),scalar(total?.votes)]);}
    tables.push({title:'Selection totals (first 500 rows)',headings:['District','Selection','Votes'],rows});
  }
  if(checks.length)tables.push({title:'Package checks',headings:['Check','Target','Result','Detail'],rows:checks.map(check=>[
    scalar(check?.equation_id).replaceAll('_',' '),scalar(check?.reporting_unit_id),scalar(check?.status),scalar(check?.error),
  ])});
  const runs=Array.isArray(result?.runs)?result.runs.slice(0,100):[];
  let remaining=500-checks.length;
  for(const run of runs){
    const steps=Array.isArray(run?.steps)?run.steps.slice(0,remaining):[];remaining-=steps.length;
    tables.push({title:`${scalar(run?.method_id)} · ${scalar(run?.status)}`,note:typeof run?.boundary==='string'?run.boundary.slice(0,2000):'',headings:['Step','Sample unit','Statistic','p-value','Decision'],rows:steps.map(step=>[
      scalar(step?.step_index),scalar(step?.sample_unit_id),`${scalar(step?.statistic?.numerator)}/${scalar(step?.statistic?.denominator)}`,
      Number.isInteger(step?.p_value_ppm)&&step.p_value_ppm>=0&&step.p_value_ppm<=1000000?`${step.p_value_ppm/10000}%`:'—',
      step?.stop===true?'Stop':step?.stop===false?'Continue':'—',
    ])});
  }
  return tables;
}
export function renderCountResult(container,result){
  container.replaceChildren();
  for(const model of countResultTables(result)){
    const title=document.createElement('h3');title.textContent=model.title;container.append(title);
    if(model.note){const note=document.createElement('p');note.textContent=model.note;container.append(note);}
    const table=document.createElement('table'),head=document.createElement('thead'),header=document.createElement('tr');
    for(const text of model.headings){const cell=document.createElement('th');cell.scope='col';cell.textContent=text;header.append(cell);}head.append(header);table.append(head);
    const body=document.createElement('tbody');
    for(const row of model.rows){const line=document.createElement('tr');for(const text of row){const cell=document.createElement('td');cell.textContent=text;line.append(cell);}body.append(line);}table.append(body);container.append(table);
  }
}
