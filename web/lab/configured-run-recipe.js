import {validateConfigRunSettings} from './config-run.js';
export function validateConfiguredRecipe(recipe,document){
 if(!recipe||Object.keys(recipe).sort().join()!=='execution_attempt,schema_version,settings'||recipe.schema_version!=='bisect-configured-run-recipe-v1')throw new Error('Invalid configured run recipe.');
 const s=validateConfigRunSettings(recipe.settings),o=document?.provenance?.producer?.engine_options,c=document?.provenance?.producer?.algorithm_configuration,e=document?.provenance?.producer?.electoral_settings;
 if(!o||s.chamber!==document.metadata?.chamber||s.districts!==document.plan?.k||(s.capture_rounds===true)!==(o.capture_rounds===true)||(s.vra_absent_file_fallback===true)!==(o.vra_absent_file_fallback===true))throw new Error('Run recipe disagrees with native plan settings.');
 if((s.compact_geometry??null)!==(o.compact_geometry??null))throw new Error('Run recipe disagrees with CompactBisect geometry inputs.');
 const attempt=c?.execution_attempt??null;
 if(recipe.execution_attempt!==attempt||(s.native_build===true)!==(attempt!==null)||attempt!==null&&(!Number.isSafeInteger(attempt)||attempt<0||attempt>50))throw new Error('Run recipe disagrees with native build attempt.');
 const hasElectoral=Object.hasOwn(s,'seats_per_district')||Object.hasOwn(s,'total_seats');
 if(hasElectoral?(!e||e.seats_per_district!==(s.seats_per_district??1)||(e.total_seats??null)!==(s.total_seats??null)):e!=null)throw new Error('Run recipe disagrees with native electoral settings.');
 return recipe;
}
