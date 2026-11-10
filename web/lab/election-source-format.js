// Explicit source adaptation; no inferred geography, party mapping or execution.
export const ELECTION_CSV_FORMATS=['canonical','fekrazad-2020'];
export function validateElectionCsvFormat(value){if(!ELECTION_CSV_FORMATS.includes(value))throw new Error('Choose canonical CSV or the explicit Fekrazad 2020 mapping.');return value;}
export function validateElectionSourceFormat(value){if(!value||Object.keys(value).join()!=='format')throw new Error('Invalid election source format settings.');validateElectionCsvFormat(value.format);return value;}
function sourceBytes(source){if(!source||Object.keys(source).sort().join()!=='base64,name'||typeof source.name!=='string'||source.name.length>200||typeof source.base64!=='string'||source.base64.length>4*1024*1024||!/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(source.base64))throw new Error('Select an original election source of at most 3 MiB.');return Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));}
function csvRows(text){
 const rows=[];let row=[],field='',quoted=false,closed=false;
 const push=()=>{row.push(field);field='';closed=false;if(row.length>512)throw new Error('Election source has too many columns.');};
 const end=()=>{push();if(row.length!==1||row[0].trim()!=='')rows.push(row);row=[];if(rows.length>100001)throw new Error('Election source exceeds 100,000 records.');};
 for(let i=0;i<text.length;i++){
  const c=text[i];if(quoted){if(c==='"'){if(text[i+1]==='"'){field+='"';i++;}else{quoted=false;closed=true;}}else field+=c;continue;}
  if(c===','){push();continue;}if(c==='\r'||c==='\n'){if(c==='\r'&&text[i+1]==='\n')i++;end();continue;}
  if(closed){if(c!==' '&&c!=='\t')throw new Error('Invalid text after a quoted election field.');continue;}
  if(c==='"'){if(field.length)throw new Error('Unexpected quote in election source.');quoted=true;}else field+=c;
 }
 if(quoted)throw new Error('Unterminated quoted election field.');if(field.length||row.length||closed)end();return rows;
}
export function normalizeElectionSource(text,format='canonical'){
 validateElectionCsvFormat(format);if(typeof text!=='string'||!text.length||new TextEncoder().encode(text).length>3*1024*1024)throw new Error('Election source must be nonempty UTF-8 text up to 3 MiB.');
 if(format==='canonical')return {csv:text,mapping:null,row_count:null};
 const rows=csvRows(text.replace(/^\uFEFF/,'')),headers=rows.shift()?.map(x=>x.trim());
 if(!headers?.length||headers.some(h=>!h)||new Set(headers).size!==headers.length)throw new Error('Election source has empty or duplicate headers.');
 const columns=['tract_GEOID','G20PREDBID','G20PRERTRU'].map(name=>{const i=headers.indexOf(name);if(i<0)throw new Error('Fekrazad 2020 requires tract_GEOID, G20PREDBID and G20PRERTRU.');return i;});
 const seen=new Set(),out=['geoid,dem_votes,rep_votes'];
 for(const row of rows){
  if(row.length!==headers.length)throw new Error('Election source record width differs from its header.');
  const raw=row[columns[0]].trim();if(!/^\d{10,11}$/.test(raw))throw new Error('Fekrazad tract IDs must contain 10 or 11 decimal digits.');const id=raw.padStart(11,'0');if(seen.has(id))throw new Error('Election source has duplicate normalized tract GEOIDs.');seen.add(id);
  const votes=columns.slice(1).map(i=>{const v=row[i].trim(),n=Number(v);if(!/^\+?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/.test(v)||!Number.isFinite(n)||n<0||Object.is(n,-0))throw new Error('Fekrazad votes must be finite nonnegative decimal values.');return v;});
  out.push(id+','+votes.join(','));
 }
 if(!seen.size)throw new Error('Election source contains no tract records.');
 return {csv:out.join('\n')+'\n',row_count:seen.size,mapping:{format:'fekrazad-2020',geoid:'tract_GEOID',dem_votes:'G20PREDBID',rep_votes:'G20PRERTRU',geoid_rule:'decimal digits left-padded to 11',vote_rule:'trimmed original decimal text; no rounding or numeric reserialization',excluded:'Other parties and all other columns; no precinct allocation or geographic vintage conversion'}};
}
export function electionAnalysisCsv(files,fallback=null){
 const source=files.electionAnalysisSource||fallback;if(!source)return null;
 const text=new TextDecoder('utf-8',{fatal:true}).decode(sourceBytes(source));
 return normalizeElectionSource(text,files.electionCsvFormat?validateElectionSourceFormat(files.electionCsvFormat).format:'canonical').csv;
}
export async function electionSourcePackage(source,format){
 const original=sourceBytes(source),converted=normalizeElectionSource(new TextDecoder('utf-8',{fatal:true}).decode(original),format),canonical=new TextEncoder().encode(converted.csv),hash=async b=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',b)),v=>v.toString(16).padStart(2,'0')).join('');
 let binary='';for(const b of canonical)binary+=String.fromCharCode(b);
 return {schema_version:'bisect-election-source-adaptation-v1',source:{...source,sha256:await hash(original),byte_length:original.length},canonical:{name:'presidential_by_tract.csv',base64:btoa(binary),sha256:await hash(canonical),byte_length:canonical.length},mapping:converted.mapping,row_count:converted.row_count,source_authenticity_verified:false,execution_performed:false};
}
