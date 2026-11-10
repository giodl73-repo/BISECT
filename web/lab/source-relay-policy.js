// Public immutable source allowlist shared by the client and Cloudflare relay.
// No ACS keys, user URLs, redirects or private network destinations are allowed.
import {CENSUS_ARCHIVE_ENDPOINTS} from './census-endpoints.js';
export const SOURCE_RELAY_PATH='/api/source';
export const SOURCE_RELAY_HEADER='x-bisect-source-relay';
export const SOURCE_RELAY_VERSION='v1';
const pins=new Map(Object.entries(CENSUS_ARCHIVE_ENDPOINTS).map(([key,url])=>[url,(key.endsWith(':pl94171')?128:64)*1024*1024]));
const states='al ak az ar ca co ct de fl ga hi id il in ia ks ky la me md ma mi mn ms mo mt ne nv nh nj nm ny nc nd oh ok or pa ri sc sd tn tx ut vt va wa wv wi wy'.split(' ');
for(const state of [...states,'dc'])for(const year of ['2000','2010','2020'])for(const [kind,suffix]of [['wac','S000_JT00'],['od','main_JT00']])pins.set(`https://lehd.ces.census.gov/data/lodes/LODES8/${state}/${kind}/${state}_${kind}_${suffix}_${year}.csv.gz`,16*1024*1024);
const schoolFips=new Set(['11',...Object.entries(CENSUS_ARCHIVE_ENDPOINTS).filter(([key])=>key.endsWith(':2020:tiger')).map(([,url])=>/tl_2020_([0-9]{2})_tract\.zip$/.exec(url)?.[1]).filter(Boolean)]);
for(const year of ['2000','2010','2020']){for(const fips of schoolFips)pins.set(`https://www2.census.gov/geo/tiger/TIGER${year}/UNSD/tl_${year}_${fips}_unsd.zip`,64*1024*1024);pins.set(`https://www.eia.gov/electricity/data/eia861/zip/f861${year}.zip`,64*1024*1024);}
export function sourceRelayLimit(value){
 if(typeof value!=='string'||!pins.has(value))throw new Error('Source relay accepts only pinned public Census, LODES and EIA source URLs.');
 return pins.get(value);
}
export function sourceRelayUrl(value,origin=globalThis.location?.origin){
 sourceRelayLimit(value);const base=new URL(origin);if(!['http:','https:'].includes(base.protocol)||base.origin!==origin)throw new Error('Source relay requires this site’s HTTP origin.');
 return base.origin+SOURCE_RELAY_PATH+'?url='+encodeURIComponent(value);
}
