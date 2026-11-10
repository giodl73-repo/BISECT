import {validateSourceFile} from './package-files.js';
const types=['districts','political','demographic','compactness'];
export function installWorkflowMapPreview(document){
 const select=document.getElementById('tui-workflow-preview-type'),panel=document.getElementById('tui-workflow-map-preview'),status=document.getElementById('tui-workflow-preview-status');
 let current=null,url=null,shown=null;
 function clear(){if(url)URL.revokeObjectURL(url);url=null;shown=null;panel.replaceChildren();}
 function render(){
  const type=select.value;if(shown===type&&url)return;clear();
  if(!current||!type)return;
  const name='maps/'+type+'.png',base64=current.workflow.files_base64[name];validateSourceFile({name,base64},{maxBytes:64*1024*1024});
  const bytes=Uint8Array.from(atob(base64),c=>c.charCodeAt(0));url=URL.createObjectURL(new Blob([bytes],{type:'image/png'}));shown=type;
  const img=document.createElement('img');img.alt='Native '+type+' map of the accepted plan';img.src=url;panel.append(img);status.textContent='Native '+type+' PNG from this completed workflow.';
 }
 select.onchange=render;
 return {update(result){
  if(current===result)return;
  const previous=select.value;current=result;clear();
  const available=result?types.filter(type=>result.workflow?.files_base64?.['maps/'+type+'.png']&&result.workflow?.stages?.some(s=>s.name==='map:'+type&&s.status==='completed')):[];
  select.replaceChildren(...available.map(type=>{const option=document.createElement('option');option.value=type;option.textContent=type[0].toUpperCase()+type.slice(1);return option;}));
  select.disabled=!available.length;if(available.includes(previous))select.value=previous;
  status.textContent=available.length?'Choose a completed native workflow map.':'Run a workflow to preview its completed maps.';render();
 }};
}
