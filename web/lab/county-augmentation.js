const preKinds=['economic-character','housing-character','zone-membership'];
export function validateCountyAugmentation(config){
 if(!Object.hasOwn(config??{},'county_augmentation'))return null;
 const alpha=config.county_augmentation;if(!Number.isFinite(alpha)||alpha<0||alpha>100||Object.is(alpha,-0)||config.weights==='county')throw new Error('County augmentation requires finite alpha in [0,100] and a non-county base weight.');return alpha;
}
const county=(g,i)=>g.county_ids?.[i]??g.geoids[i].slice(0,5);
export const countyPrecedesWeights=config=>preKinds.includes(config.weights);
export function countyLength(g,config,u,v,length){const alpha=validateCountyAugmentation(config);return alpha!==null&&alpha>1e-10&&county(g,u)===county(g,v)?length*(1+alpha):length;}
export function countyPreLength(g,c,u,v,w){return countyPrecedesWeights(c)?countyLength(g,c,u,v,w):w;}
export function countyPreGraph(g,c){return validateCountyAugmentation(c)!==null&&countyPrecedesWeights(c)?{...g,edges:g.edges.map(([u,v,w])=>[u,v,countyLength(g,c,u,v,w)])}:g;}
export function countyAugmentation(g,c,baseEdges,scope='edge-weighting'){
 const alpha=validateCountyAugmentation(c);if(alpha===null)return null;
 let eligible=0,changed=0,total=0;const edges=baseEdges.map(([u,v,w])=>{if(county(g,u)===county(g,v))eligible++;const value=countyLength(g,c,u,v,w);total+=value;if(!Number.isFinite(value)||value<0||!Number.isFinite(total))throw new Error('County augmentation exceeds the native finite range.');if(value!==w)changed++;return [u,v,value];});
 const weights=new Map(edges.map(([u,v,w])=>[u+':'+v,w]));return {edges,weights,evidence:{method:'native-county-augmentation-v1',alpha,within_county_multiplier:alpha>1e-10?1+alpha:1,disabled_at_or_below:1e-10,county_identity:g.county_ids?'explicit-prepared-county-ids':'first-five-geoid-digits',map_entries:edges.length,eligible_edges:eligible,changed_edges:changed,implicit_weights:'not-materialized',order:countyPrecedesWeights(c)?'before-character-or-zone;before-coi':'after-base-override;before-coi',status:alpha<=1e-10?'disabled':edges.length===0?'no-effect-empty-map':changed===0?'applied-no-edge-change':'applied',scope}};
}
export function verifyCountyAugmentation(value,computed){if(!computed){if(value!==undefined)throw new Error('Unrequested county augmentation evidence.');return;}const expected=computed.evidence;if(!value||Object.keys(value).sort().join()!==Object.keys(expected).sort().join()||Object.keys(expected).some(k=>value[k]!==expected[k]))throw new Error('County augmentation evidence differs from native composition inputs.');}
