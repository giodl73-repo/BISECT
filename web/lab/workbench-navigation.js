// Browser routing for the native TUI screens. Choosing a tool never executes it.
import {NATIVE_METRIC_GLOSSARY} from './native-metric-glossary.js';
export const TUI_SCREENS=Object.freeze([
 {name:'Home',operation:'inspect-tui-plan-library',key:'1'},
 {name:'Run',operation:'run-tui-configured',key:'2',alias:'r'},
 {name:'Compare',operation:'compare-tui-plans',key:'3',alias:'c'},
 {name:'Verify',operation:'verify-tui-manifest',key:'4',alias:'v'},
 {name:'Doctor',operation:'diagnose-tui-location',key:'5',alias:'d'}
]);
export function installWorkbenchNavigation({document,selectOperation,isBusy}){
 const get=id=>document.getElementById(id),select=get('operation'),nav=get('tui-navigation'),dialog=get('tool-picker'),query=get('tool-query'),list=get('tool-results'),message=get('tui-navigation-status');
 const tools=Array.from(select.options).filter(option=>!option.disabled).map(option=>({operation:option.value,label:option.textContent}));
 const history=[];let observed=select.value,filtered=[],active=0,priorFocus=null;
 const glossary=get('metric-glossary');let glossaryFocus=null;
 for(const [term,definition]of NATIVE_METRIC_GLOSSARY){const dt=document.createElement('dt'),dd=document.createElement('dd');dt.textContent=term;dd.textContent=definition;get('metric-glossary-entries').append(dt,dd);}
 function closeGlossary(){glossary.close();glossaryFocus?.focus?.();}
 function openGlossary(){if(busyRefusal()||dialog.open)return;glossaryFocus=document.activeElement;glossary.showModal();get('metric-glossary-title').tabIndex=-1;get('metric-glossary-title').focus();glossary.scrollTop=0;}
 get('open-metric-glossary').onclick=openGlossary;get('close-metric-glossary').onclick=closeGlossary;
 glossary.addEventListener('cancel',event=>{event.preventDefault();closeGlossary();});
 glossary.addEventListener('keydown',event=>{if(event.key==='?'||event.key==='Escape'){event.preventDefault();event.stopPropagation();closeGlossary();}});
 const editing=target=>target?.closest?.('input,textarea,select,[contenteditable]:not([contenteditable="false"])');
 function busyRefusal(){if(!isBusy())return false;message.textContent='Finish or cancel the current operation before changing tools.';return true;}
 function choose(operation,{back=false}={}){
  if(!tools.some(tool=>tool.operation===operation)||busyRefusal())return false;
  if(operation===select.value)return true;
  if(back)observed=operation;
  selectOperation(operation);if(select.value!==operation)return false;refresh();message.textContent='Tool opened. Retained sources are unchanged. Run explicitly to compute.';return true;
 }
 function refresh(){
  if(observed!==select.value){history.push(observed);if(history.length>50)history.shift();observed=select.value;}
  for(const button of nav.querySelectorAll('[data-operation]')){button.disabled=isBusy();if(button.dataset.operation===select.value)button.setAttribute('aria-current','page');else button.removeAttribute('aria-current');}
  get('open-tool-picker').disabled=isBusy();get('tui-back').disabled=isBusy()||!history.length;
  get('open-metric-glossary').disabled=isBusy();
 }
 function back(){if(busyRefusal()||!history.length)return;const target=history.pop();choose(target,{back:true});refresh();}
 function close(){dialog.close();priorFocus?.focus?.();}
 function highlight(){for(const [index,row]of Array.from(list.children).entries())row.setAttribute('aria-selected',String(index===active));if(filtered.length){query.setAttribute('aria-activedescendant','tool-option-'+active);list.children[active]?.scrollIntoView({block:'nearest'});}else query.removeAttribute('aria-activedescendant');}
 function search(){
  const text=query.value.trim().toLowerCase(),alias=TUI_SCREENS.find(screen=>[screen.name.toLowerCase(),screen.alias].includes(text));
  filtered=tools.filter(tool=>!text||tool.label.toLowerCase().includes(text)||tool.operation.includes(text)||tool.operation===alias?.operation);
  if(alias)filtered.sort((a,b)=>Number(b.operation===alias.operation)-Number(a.operation===alias.operation));active=0;list.replaceChildren();
  for(const [index,tool]of filtered.entries()){const row=document.createElement('li');row.id='tool-option-'+index;row.role='option';row.textContent=tool.label;row.onclick=()=>{if(choose(tool.operation))close();};list.append(row);}
  get('tool-match-count').textContent=filtered.length?filtered.length+' tools. Arrow keys select; Enter opens without running.':'No matching tool.';highlight();
 }
 function open(){if(busyRefusal())return;priorFocus=document.activeElement;query.value='';search();dialog.showModal();query.focus();}
 for(const screen of TUI_SCREENS){const button=document.createElement('button');button.type='button';button.dataset.operation=screen.operation;button.textContent=screen.name;button.setAttribute('aria-keyshortcuts','Alt+'+screen.key);button.onclick=()=>choose(screen.operation);nav.append(button);}
 get('open-tool-picker').onclick=open;get('close-tool-picker').onclick=close;get('tui-back').onclick=back;query.oninput=search;
 query.onkeydown=event=>{if(event.key==='Escape'){event.preventDefault();close();}else if(['ArrowDown','ArrowUp'].includes(event.key)){event.preventDefault();if(filtered.length)active=(active+(event.key==='ArrowDown'?1:-1)+filtered.length)%filtered.length;highlight();}else if(event.key==='Enter'){event.preventDefault();if(filtered[active]&&choose(filtered[active].operation))close();}};
 dialog.addEventListener('cancel',event=>{event.preventDefault();close();});
 document.addEventListener('keydown',event=>{
  if(event.defaultPrevented||event.isComposing||event.repeat||dialog.open||glossary.open||editing(event.target))return;
  if(event.altKey&&!event.ctrlKey&&!event.metaKey){const screen=TUI_SCREENS.find(item=>item.key===event.key);if(screen){event.preventDefault();choose(screen.operation);}return;}
  if(event.ctrlKey||event.metaKey)return;
  if(event.key===':'){event.preventDefault();open();}
  else if(event.key==='?'&&TUI_SCREENS.some(screen=>screen.operation===select.value)){event.preventDefault();openGlossary();}
  else if(event.key==='Escape'){if(history.length){event.preventDefault();back();}}
  else if(select.value===TUI_SCREENS[0].operation){
   if(event.key.toLowerCase()==='s'){event.preventDefault();if(!busyRefusal())get('tui-library-cycle-sort').click();return;}
   if(['ArrowDown','ArrowUp'].includes(event.key)){
    const plans=get('tui-library-selection');if(busyRefusal()||plans.disabled||!plans.options.length)return;
    event.preventDefault();const next=Math.max(0,Math.min(plans.options.length-1,plans.selectedIndex+(event.key==='ArrowDown'?1:-1)));
    if(next!==plans.selectedIndex){plans.selectedIndex=next;plans.dispatchEvent(new document.defaultView.Event('change',{bubbles:true}));}return;
   }
   const screen=TUI_SCREENS.find(item=>item.alias===event.key.toLowerCase());if(screen){event.preventDefault();choose(screen.operation);}
  }
 });
 refresh();return {refresh};
}
