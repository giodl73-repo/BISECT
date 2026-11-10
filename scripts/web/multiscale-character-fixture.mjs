export function multiscaleCharacterFixture(engine,g,kind){
 const header=kind==='economic'?'geoid,c000,cns01,cns02,cns05,cns07,cns08,cns09,cns10,cns11':'geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage';
 const rows=g.geoids.map((id,i)=>id+','+(i%3===0?(kind==='economic'?'0,0,0,0,0,0,0,0,0':'0,0,0,0'):kind==='economic'?(i%2?'1000,10,20,30,400,40,100,50,50':'1000,10,20,30,100,200,200,100,100'):(i%2?'.8,.1,.6,.2':'.1,.7,.2,.8')));
 const csv=header+'\n'+rows.join('\n')+'\n';return engine.execute({operation:'import-character-csv',state:g.state,year:g.year,data_year:'2022',kind,source_label:'SYNTHETIC multiscale parent observations, not observed LODES/ACS',source_base64:Buffer.from(csv).toString('base64')});
}
