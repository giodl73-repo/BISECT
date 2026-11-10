export function canResumeLabelYears(result,schema){
 return result?.schema_version===schema&&result.status==='in-progress'&&Array.isArray(result.requested_years)&&result.requested_years.length<=10&&result.requested_years.every(year=>typeof year==='string'&&/^\d{4}$/.test(year))&&new Set(result.requested_years).size===result.requested_years.length&&Array.isArray(result.years)&&result.years.length>0&&result.years.length<result.requested_years.length&&result.years.every((item,i)=>item?.year===result.requested_years[i]&&['completed','failed'].includes(item.status));
}
export function labelYearsResumeRequest(request,checkpoint){
 const analysis=request?.operation==='run-label-analysis-years',report=request?.operation==='run-label-report-years';
 if(!analysis&&!report||!canResumeLabelYears(checkpoint,analysis?'bisect-label-analysis-years-v1':'bisect-label-report-years-v1'))throw new Error('Select an incomplete matching year checkpoint and its original source request.');
 return {operation:analysis?'resume-label-analysis-years':'resume-label-report-years',input:request.input,checkpoint};
}
