import {freshNativeStateRetainedText} from './fresh-native-state-session.js';
import {inspectJsonText,validateJsonTree} from './project.js';

// Use the existing large-workspace value budget within the retained file's
// 64 MiB cap. Parsing saved data never grants current execution authority.
export function parseFreshNativeStateRetainedRun(file){
 const text=freshNativeStateRetainedText(file);
 inspectJsonText(text,4000000);
 const value=JSON.parse(text);
 validateJsonTree(value,4000000,64*1024*1024);
 return value;
}
