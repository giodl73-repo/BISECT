// Explicit CLI State presets. Applying one changes controls; it never runs a plan.
export function nativeStatePreset(mode){
 const common={seed:null,iterations:100,alpha_county:0,metis_objective:'cut',metis_trials:1};
 if(mode==='partisan-weighted')return {...common,structure:'standard-bisect',weights:'partisan',search:'multi',seeds:50,dem_threshold:.55,rep_threshold:.45};
 if(mode==='metis-vra')return {...common,structure:'nway',weights:'vra-aligned',search:'single',seeds:1,metis_ufactor:5};
 throw new Error('Unknown native State preset.');
}
