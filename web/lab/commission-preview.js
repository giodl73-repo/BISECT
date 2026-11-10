// Preview is a separate, restricted copy. Downloads retain the native HTML.
const TAGS=new Set('title style div footer h1 h2 h3 h4 p pre code span strong em b i br hr ol ul li dl dt dd table thead tbody tfoot tr th td caption img a section header main'.split(' '));
const ATTRS=new Set(['class','id','style','title','alt','colspan','rowspan','scope','role','aria-label','aria-labelledby','aria-describedby']);
export const COMMISSION_PREVIEW_CSP="default-src 'none'; script-src 'none'; img-src data:; style-src 'unsafe-inline'; object-src 'none'; frame-src 'none'; form-action 'none'; base-uri 'none'";
export function installCommissionPreview(document,html){
 if(document.querySelector('meta[http-equiv="Content-Security-Policy"]')?.content!==COMMISSION_PREVIEW_CSP)throw new Error('Restricted report preview page is unavailable. Download the original HTML instead.');
 const content=document.createElement('template');content.innerHTML=commissionPreviewDocument(html,document);
 document.body.replaceChildren(content.content);
}
export function commissionPreviewDocument(html,document){
 if(typeof html!=='string'||new TextEncoder().encode(html).length>25*1024*1024)throw new Error('Report preview is limited to 25 MiB. Download the original HTML instead.');
 // Template contents stay detached and inert; never insert original nodes.
 const source=document.createElement('template');source.innerHTML=html;
 const output=document.createElement('template');let count=0;
 function copy(node,parent,depth){
  if(++count>250000||depth>64)throw new Error('Report preview structure exceeds its limits. Download the original HTML instead.');
  if(node.nodeType===3){parent.appendChild(document.createTextNode(node.data));return;}
  if(node.nodeType!==1||node.namespaceURI!=='http://www.w3.org/1999/xhtml'||!TAGS.has(node.localName))return;
  const safe=document.createElement(node.localName);
  for(const {name,value}of node.attributes)if(ATTRS.has(name))safe.setAttribute(name,value);
  // Navigation is omitted, including fragment links, so the frame stays put.
  if(node.localName==='img'){
   const src=node.getAttribute('src')||'';
   if(!/^data:image\/png;base64,[A-Za-z0-9+/]*={0,2}$/.test(src))return;
   safe.setAttribute('src',src);
  }
  for(const child of node.childNodes)copy(child,safe,depth+1);
  parent.appendChild(safe);
 }
 for(const node of source.content.childNodes)copy(node,output.content,0);
 return '<!doctype html><html lang="en"><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="'+COMMISSION_PREVIEW_CSP+'"><meta name="referrer" content="no-referrer"></head><body>'+output.innerHTML+'</body></html>';
}
