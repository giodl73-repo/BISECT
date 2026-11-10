// Supplied WGS84 display geometry joined by exact unit identity.
export function planMapModel(document, geometry) {
  const plan=document?.plan, ids=plan?.units?.unit_ids;
  if(!Array.isArray(ids)||!ids.length||ids.length>100000||!Number.isInteger(plan.k)||plan.k<1||plan.k>500||!Array.isArray(plan.assignment)||plan.assignment.length!==ids.length)throw new Error('Map requires a validated RPLAN with 1–500 districts.');
  const index=new Map(ids.map((id,i)=>[id,i]));
  if(index.size!==ids.length||ids.some(id=>typeof id!=='string'||!id)||plan.assignment.some(d=>!Number.isInteger(d)||d<0||d>=plan.k))throw new Error('Invalid plan unit IDs or district labels.');
  if(geometry?.type!=='FeatureCollection'||!Array.isArray(geometry.features)||geometry.features.length!==ids.length||geometry.crs!=null)throw new Error('Supply WGS84 GeoJSON with exactly one polygon feature per plan unit.');
  for(const key of ['state','year'])if(geometry[key]!=null&&String(geometry[key])!==String(plan.units[key]))throw new Error('Geometry state/year does not match the plan.');
  const positionLimit=['source-exact-large','display-topology'].includes(geometry.geometry_profile)?2000000:500000;
  let points=0;const seen=new Set(),features=[];const bounds=[Infinity,Infinity,-Infinity,-Infinity];
  const project=([lon,lat])=>[lon,180/Math.PI*Math.log(Math.tan(Math.PI/4+lat*Math.PI/360))];
  for(const f of geometry.features){
    const id=f?.properties?.geoid??f?.properties?.GEOID??f?.id;
    if(typeof id!=='string'||!index.has(id)||seen.has(id))throw new Error('Geometry has duplicate, missing or unrelated unit IDs.');seen.add(id);
    if(f.type!=='Feature'||!['Polygon','MultiPolygon'].includes(f.geometry?.type))throw new Error('Geometry must contain Polygon or MultiPolygon features.');
    const polygons=f.geometry.type==='Polygon'?[f.geometry.coordinates]:f.geometry.coordinates;
    if(!Array.isArray(polygons)||!polygons.length)throw new Error('Empty polygon geometry.');
    const rings=[];
    for(const polygon of polygons){
      if(!Array.isArray(polygon)||!polygon.length)throw new Error('Empty polygon rings.');
      for(const ring of polygon){
        if(!Array.isArray(ring)||ring.length<4)throw new Error('Polygon rings must have at least four positions.');
        if(ring[0]?.[0]!==ring.at(-1)?.[0]||ring[0]?.[1]!==ring.at(-1)?.[1])throw new Error('Polygon rings must be closed.');
        const projected=[];
        for(const p of ring){
          if(++points>positionLimit)throw new Error('Display geometry exceeds its '+positionLimit.toLocaleString('en-US')+' position profile.');
          if(!Array.isArray(p)||p.length!==2||!p.every(Number.isFinite)||Math.abs(p[0])>180||Math.abs(p[1])>85)throw new Error('Map requires finite WGS84 positions between 85° south and north.');
          const [x,y]=project(p);bounds[0]=Math.min(bounds[0],x);bounds[1]=Math.min(bounds[1],y);bounds[2]=Math.max(bounds[2],x);bounds[3]=Math.max(bounds[3],y);projected.push([x,y]);
        }
        rings.push(projected);
      }
    }
    features.push({id,district:plan.assignment[index.get(id)],rings});
  }
  if(!(bounds[2]>bounds[0]&&bounds[3]>bounds[1]))throw new Error('Geometry has no drawable extent.');
  const scale=Math.min(760/(bounds[2]-bounds[0]),480/(bounds[3]-bounds[1]));
  const xOffset=(800-scale*(bounds[2]-bounds[0]))/2,yOffset=(520-scale*(bounds[3]-bounds[1]))/2;
  return {units:ids.length,k:plan.k,features:features.map(f=>({...f,path:f.rings.map(r=>r.map(([x,y],i)=>(i?'L':'M')+(xOffset+(x-bounds[0])*scale).toFixed(2)+','+(yOffset+(bounds[3]-y)*scale).toFixed(2)).join(' ')+' Z').join(' ')}))};
}

export function renderPlanMap(container,model,label){
  container.replaceChildren();
  const note=document.createElement('p');note.textContent=label+` · ${model.units} matched units · ${model.k} districts. Supplied display geometry; boundaries are not authenticated. Click a unit to inspect its assignment.`;container.append(note);
  const status=document.createElement('p');status.setAttribute('role','status');status.textContent='Choose a unit on the map.';
  const ns='http://www.w3.org/2000/svg',svg=document.createElementNS(ns,'svg');svg.setAttribute('viewBox','0 0 800 520');svg.setAttribute('role','img');svg.setAttribute('aria-label','District assignments on supplied unit geometry');svg.style.width='100%';
  for(const f of model.features){
    const path=document.createElementNS(ns,'path');path.setAttribute('d',f.path);path.setAttribute('fill',`hsl(${(f.district*137.508)%360} 48% 65%)`);path.setAttribute('stroke','#ffffff');path.setAttribute('stroke-width','0.5');path.setAttribute('fill-rule','evenodd');
    const title=document.createElementNS(ns,'title');title.textContent=`Unit ${f.id} · District ${f.district+1} (native label ${f.district})`;path.append(title);path.setAttribute('tabindex','0');path.setAttribute('role','button');path.setAttribute('aria-label',title.textContent);path.onclick=()=>{status.textContent=title.textContent;};path.onkeydown=event=>{if(event.key==='Enter'||event.key===' '){event.preventDefault();status.textContent=title.textContent;}};svg.append(path);
  }
  container.append(svg,status);
}
