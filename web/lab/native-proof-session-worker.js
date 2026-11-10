import {parseProofSession} from './native-proof-session.js';
self.onmessage = ({data}) => {
  try { self.postMessage({session: parseProofSession(new Uint8Array(data.bytes))}); }
  catch (error) { self.postMessage({error: String(error?.message ?? error)}); }
};
