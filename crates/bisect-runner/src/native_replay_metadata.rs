//! Architecture-independent recipe metadata. These values do not allocate work.
use crate::native_replay_recipe::{CountDomain,ReadOnlyRecipe};
use crate::state_config::SeedCompositor;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug,Serialize)]
pub struct UnsignedMetadata {
    pub pointer:String,
    pub decimal:String,
    pub fits_u32:bool,
    pub javascript_safe:bool,
}
pub fn unsigned_metadata(recipe:&ReadOnlyRecipe)->Result<Vec<UnsignedMetadata>,String>{
    fn visit(value:&Value,pointer:&str,out:&mut Vec<UnsignedMetadata>){
        match value{
            Value::Number(n)=>if let Some(n)=n.as_u64(){out.push(UnsignedMetadata{pointer:pointer.into(),decimal:n.to_string(),fits_u32:n<=u64::from(u32::MAX),javascript_safe:n<=9_007_199_254_740_991});},
            Value::Array(a)=>for(i,v)in a.iter().enumerate(){visit(v,&format!("{pointer}/{i}"),out)},
            Value::Object(m)=>for(k,v)in m{visit(v,&format!("{pointer}/{}",k.replace('~',"~0").replace('/',"~1")),out)},
            _=>{},
        }
    }
    let value=serde_json::to_value(recipe).map_err(|e|e.to_string())?;
    let mut out=Vec::new();visit(&value,"",&mut out);Ok(out)
}
impl<C:CountDomain> SeedCompositor<C>{
    /// The native display/count policy, without converting metadata to usize.
    pub fn seed_count_u64(&self)->u64{
        match self{
            Self::Single=>1,
            Self::ConvergenceSweep{threshold}=>u64::from(*threshold),
            Self::Multi{seeds}|Self::Percentile{seeds,..}=>(*seeds).to_u64(),
            Self::BisectionEnsemble{ensemble_steps,..}=>(*ensemble_steps).to_u64(),
            Self::Flip{flip_steps,..}=>(*flip_steps).to_u64(),
            Self::ShortBurst{n_bursts,..}|Self::ShortBurstForest{n_bursts,..}|Self::ShortBurstMergeSplit{n_bursts,..}=>(*n_bursts).to_u64(),
            Self::ForestRecom{steps,..}|Self::MergeSplit{steps,..}|Self::ParallelTempering{steps,..}|Self::VraRecom{steps,..}=>(*steps).to_u64(),
            Self::MultiScale{total_steps,..}|Self::MultiScaleAdaptive{total_steps,..}=>(*total_steps).to_u64(),
            Self::SmcPercentile{n_particles,..}=>(*n_particles).to_u64(),
        }
    }
}
