import {validateNativeRoundProgress} from '../../../web/lab/native-round-progress.js';

// Checkpoint retention consumes its own stream; native observations remain valid
// progress and unknown schemas must fail rather than be silently discarded.
export function isLabelBuildCheckpoint(event){
 if(event?.schema_version==='bisect-label-build-progress-v1')return true;
 validateNativeRoundProgress(event);
 return false;
}
