export const CONFIG_TEMPLATE_DEFAULTS={name:'browser_baseline',structure:'apportion-regions',weights:'geographic',alpha_county:null,search:'single',convergence_threshold:null,seeds:null,balance_tolerance:0.5,years:['2020','2010','2000'],workers:4,engine:'metis-core',seed:null};
export function validateConfigTemplate(value){
 const fields=Object.keys(CONFIG_TEMPLATE_DEFAULTS);
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==fields.length||fields.some(k=>!Object.hasOwn(value,k)))throw new Error('Invalid configuration template fields.');
 if(typeof value.name!=='string'||! /^[A-Za-z0-9_-]{1,100}$/.test(value.name))throw new Error('Template name needs 1–100 ASCII letters, digits, underscores or hyphens.');
 for(const k of ['structure','weights','search','engine'])if((k==='structure'||value[k]!==null)&&(typeof value[k]!=='string'||value[k].length<1||value[k].length>60))throw new Error('Invalid template '+k+'.');
 for(const k of ['alpha_county','balance_tolerance'])if(value[k]!==null&&!Number.isFinite(value[k]))throw new Error('Template '+k+' must be finite.');
 for(const k of ['workers','seeds','convergence_threshold'])if((k==='workers'||value[k]!==null)&&(!Number.isSafeInteger(value[k])||value[k]<0||value[k]>4294967295))throw new Error('Template '+k+' must be an unsigned 32-bit integer.');
 if(!Array.isArray(value.years)||value.years.length<1||value.years.length>100||value.years.some(y=>typeof y!=='string'||!/^\d{4}$/.test(y)))throw new Error('Supply 1–100 four-digit census years.');
 if(value.seed!==null&&(typeof value.seed!=='string'||!/^(0|[1-9][0-9]{0,19})$/.test(value.seed)||BigInt(value.seed)>18446744073709551615n))throw new Error('Seed must be a canonical decimal u64 string.');
 return value;
}
export function configTemplateRequest(settings){return {operation:'create-algorithm-config',settings:validateConfigTemplate(settings)};}
