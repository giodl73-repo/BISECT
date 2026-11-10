const bytes=s=>new TextEncoder().encode(s).length;
export const DOCTOR_COMMAND_DEFAULTS={mode:'all',output_base:'outputs',version:'v1',year:'2020',label:''};
export function validateDoctorCommand(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).sort().join()!=='label,mode,output_base,version,year'||!['all','label'].includes(value.mode))throw Error('Invalid Doctor report settings.');
 for(const key of ['output_base','version','year','label'])if(typeof value[key]!=='string'||bytes(value[key])>(key==='output_base'?4096:200)||/[\u0000-\u001f\u007f-\u009f]/.test(value[key]))throw Error('Doctor display fields exceed their text bounds.');
 for(const key of ['version','year',...(value.mode==='label'?['label']:[])])if(/[\\/:]/.test(value[key])||['.','..','__proto__','constructor','prototype'].includes(value[key]))throw Error('Use plain version, year and plan names.');
 return value;
}
const join=(...parts)=>parts.filter(Boolean).map(p=>p.replaceAll('\\','/').replace(/\/+$/,'')).join('/');
const line=s=>s+'\n';
export function doctorCommandReport(result,input){
 const command=validateDoctorCommand(input.command??DOCTOR_COMMAND_DEFAULTS);
 if(result?.schema_version!=='bisect-plan-doctor-v1')throw Error('Run a plan inventory before creating a Doctor report.');
 const root=join(command.output_base,command.version,command.year,'plans');
 const report={schema_version:'bisect-doctor-command-v1',command:structuredClone(command),reference_cli_exit_code:null,comparison:'adapted-input-diagnostic',console_text:null,authority:'Portable display prefix only; no path opened. Supplied snapshot data, not current filesystem or legal authority.'};
 if(result.plans.some(p=>p.status==='invalid-manifest'&&/expected usize/.test(p.error??''))){report.comparison='unsupported-native-manifest-width';report.console_text=line('[UNSUPPORTED INPUT] A pointer-sized manifest field could not be represented by this WASM schema.')+line('Inspect the retained parser error. No native process status is inferred.');return report;}
 if(command.mode==='all'){
  report.reference_cli_exit_code=0;report.comparison='native-console-with-portable-paths';report.console_text=input.plans.length?line('=== bisect doctor --all: '+input.plans.length+' plans in '+command.output_base+'/'+command.version+'/'+command.year+'/plans/ ===\n')+result.all_plans_table:line('No plans found in '+root)+line("Run 'bisect state --label ...' to create a plan first.");return report;
 }
 if(!command.label)throw Error('Choose a plan label before running the Doctor report.');
 const p=result.plans.find(p=>p.label===command.label);
 if(!p||!['required-files-present','missing-required-files'].includes(p.status)){
  report.console_text=line('[UNVERIFIED INPUT] '+command.label+': '+(p?.error??'The label is absent from the supplied inventory.'))+line('Review the saved source diagnostic. No native exit status is inferred for unavailable or unsupported input.');return report;
 }
 let out=line('=== bisect doctor: '+command.label+' ===\n')+line('Plan:  '+command.label+' ('+p.state+' '+p.chamber+', '+p.year+', '+p.districts+' districts)')+line('Dir:   '+join(root,command.label)+'\n');
 out+=line('── Analysis files ─────────────────────────────────────────────');
 const missingRequired=[],missingOptional=[];
 for(const row of p.analysis){
  const pad=row.name.padEnd(24,' ');
  if(row.required){if(row.native_status==='MISS'){out+=line('[MISS] '+pad+' -> run: bisect analyze --label '+command.label+' --types '+row.name.slice(0,-5));missingRequired.push(row.name);}else out+=line('['+row.native_status+'] '+pad+' '+row.native_summary);}
  else if(row.native_status!=='MISS')out+=line('[PASS] '+row.name);
  else{
   missingOptional.push(row.name);out+=line('[MISS] '+pad+' -> run: bisect analyze --label '+command.label+' --types '+row.name.slice(0,-5));
   const type=row.name==='partisan.json'?'elections':['demographic.json','vra_analysis.json'].includes(row.name)?'demographics':null;
   if(type)out+=line('       (also needs: bisect fetch --type '+type+' --states '+p.state+' --year '+p.year+')');
  }
 }
 out+=line('\n-- Report readiness ───────────────────────────────────────────');
 if(!missingRequired.length)out+=line('[PASS] Required analysis files present')+line('[INFO] To generate report: bisect report --label '+command.label+' --format html');
 else out+=line('[FAIL] Missing required files: '+missingRequired.join(', '))+line('       Run: bisect analyze --label '+command.label+' --types all --force');
 if(missingOptional.length)out+=line('[WARN] Optional files missing: '+missingOptional.join(', ')+' (report will show as unavailable)');
 out+=line('\n-- Maps ────────────────────────────────────────────────────────');
 if(p.maps.length)for(const name of p.maps)out+=line('[PASS] '+name);
 else out+=line('[MISS] No maps generated')+line('       Run: bisect map --label '+command.label+' --types districts')+line('       (needs: bisect fetch --type tiger --states '+p.state+' --year '+p.year+')');
 out+=line('\n-- Court readiness ─────────────────────────────────────────────');
 out+=p.audit_present?line('[PASS] audit.json present'):line('[WARN] audit.json missing')+line('       Run: bisect report --label '+command.label+' --audit-with-report --format html');
 out+=p.rplan_present?line('[PASS] '+command.label+'.rplan present (reproducibility package)'):line('[INFO] No .rplan file -- run: bisect export --label '+command.label+' --format rplan');
 out+=line('\n[OK] Doctor complete for '+command.label);
 report.reference_cli_exit_code=0;report.comparison='native-console-with-portable-paths';report.console_text=out;return report;
}
