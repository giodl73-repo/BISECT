// Presence hints only. Native readers and command validation remain authoritative.
export function practitionerInputChecklist(operation,files={}){
 const rows=[],add=(label,target,value)=>rows.push({label,target,present:!!value,...(typeof value?.name==='string'?{source_name:value.name.slice(0,200)}:{})});
 if(operation==='run-tui-configured'){
  add('Algorithm configuration','config-source',files.configSource);
  add('Prepared state graph','config-run-graph',files.configRunInputs?.graph);
 }else if(operation==='run-native-selected-state-source'){
  add('Single-State source mode','native-state-source-enabled',files.nativeStateSourceSettings?.enabled&&!['run','states'].includes(files.nativeStateSourceSettings?.command_kind));
  const kind=files.nativeSelectedStateSourceSettings?.kind??'native';
  if(kind==='native'){
   add('Native adjacency binary','native-adjacency-adjacency',files.nativeAdjacencyInputs?.adjacency);
   add('Ordered unit IDs','native-adjacency-geoids',files.nativeAdjacencyInputs?.geoids);
   if(['precinct','imported'].includes(files.nativeAdjacencySettings?.unit_kind))add('Aligned county IDs','native-adjacency-county_ids',files.nativeAdjacencyInputs?.county_ids);
  }else if(['census','tract'].includes(kind)){
   add('Polygon SHP','tiger-shp',files.tigerInputs?.shp);
   add('Polygon DBF','tiger-dbf',files.tigerInputs?.dbf);
   add('NAD83 projection PRJ','tract-preparation-prj',files.tractPreparationSettings?.projection);
   add('PL94 geography','pl94-geography',files.pl94Inputs?.geography);
   if(String(files.pl94Inputs?.year)!=='2000')add('PL94 population','pl94-population',files.pl94Inputs?.population);
  }
 }else if(operation==='run-state-batch')add('Prepared state jobs','state-run-batch-jobs',files.stateRunBatchInputs?.jobs);
 else if(operation==='validate-rplan-text')add('RPLAN plan','auditPlanText-file',files.auditPlanText);
 return rows;
}
