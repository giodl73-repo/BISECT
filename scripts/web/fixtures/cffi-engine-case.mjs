export function cffiEngineCase(width=8,height=8){
  const n=width*height,adjacency=Array.from({length:n},()=>[]),edges=[];
  for(let i=0;i<n;i++)for(const j of [i%width<width-1?i+1:-1,i<n-width?i+width:-1])if(j>=0){
    adjacency[i].push(j);adjacency[j].push(i);edges.push([i,j,1+(i%7)/4]);
  }
  for(const row of adjacency)row.sort((a,b)=>a-b);
  return {graph:{schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>'44001'+String(i+1).padStart(6,'0')),
    adjacency,population:Array(n).fill(100),edges,areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(1),
    centroids:Array.from({length:n},(_,i)=>[-71+(i%width)*.001,41+Math.floor(i/width)*.001])},
    options:{structure:'standard-bisect',weights:'geographic',search:'single',districts:4,seed:'7',
      seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10}};
}
