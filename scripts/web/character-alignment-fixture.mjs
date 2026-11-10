import {multiscaleCharacterFixture} from './multiscale-character-fixture.mjs';
import {alignCharacterInput} from '../../web/lab/character-input.js';
export function sparseCharacterFixture(engine,g,kind,profile='partial'){
 const source=multiscaleCharacterFixture(engine,g,kind),original=structuredClone(source.data.characters[g.geoids[1]??g.geoids[0]]),raw=kind==='economic'?structuredClone(source.data.raw_counts[g.geoids[1]??g.geoids[0]]):null;
 for(const [i,id]of g.geoids.entries())if(profile==='unmatched'||profile==='partial'&&i%2===0){delete source.data.characters[id];if(raw)delete source.data.raw_counts[id];}
 const unused=g.geoids[0].slice(0,5)+'999999';if(g.geoids.includes(unused))throw new Error('Choose another unused fixture tract');source.data.characters[unused]=original;if(raw)source.data.raw_counts[unused]=raw;
 return source;
}
export function alignedCharacterFixture(engine,g,kind){return alignCharacterInput(sparseCharacterFixture(engine,g,kind),g);}
