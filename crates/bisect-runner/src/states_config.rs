//! Native States configuration boundary, shared with CLI and WASM.
use crate::states_args::StatesArgs;
use crate::state_config::AlgorithmConfig;
#[derive(Debug,Clone)]
pub struct StatesConfiguration<C = usize> {
 pub debug:bool,pub reprocess:bool,pub algo:AlgorithmConfig<C>,pub balance_tolerance:Option<f64>,
 pub multiscale_fine:String,pub multiscale_coarse:String,pub smc_resample_threshold:f64,
}
impl StatesConfiguration {
 pub fn from_args(args:&StatesArgs)->Self {
        let mut config=Self {debug:args.debug,reprocess:args.reprocess,algo:AlgorithmConfig::defaults_for_mode(&args.partition_mode),balance_tolerance:None,multiscale_fine:"tract".into(),multiscale_coarse:"county".into(),smc_resample_threshold:0.5};
        let cfg = &mut config;
        cfg.debug = args.debug;
        cfg.reprocess = args.reprocess;
        cfg.algo = AlgorithmConfig::defaults_for_mode(&args.partition_mode);
        // Override seeds from CLI if non-default
        use crate::state_config::{SeedCompositor, SplitStrategy};
        match &mut cfg.algo.split {
            SplitStrategy::GeoSection | SplitStrategy::AreaSection { .. } => {
                cfg.algo.seeds = SeedCompositor::Multi {
                    seeds: args.geosection_seeds.max(1),
                };
            }
            _ => {}
        }
        // Override area_swing and area_section_init for AreaSection
        if let SplitStrategy::AreaSection {
            area_swing,
            area_section_init,
        } = &mut cfg.algo.split
        {
            *area_swing = args.area_swing;
            *area_section_init = args.area_section_init.into();
        }
        // Wire ILP parameters from CLI args
        if let SplitStrategy::Ilp {
            method,
            fallback,
            time_limit_secs,
            optimality_gap,
            max_tracts,
        } = &mut cfg.algo.split
        {
            *method = args.ilp_method;
            *fallback = args.ilp_fallback;
            *time_limit_secs = args.ilp_time_limit;
            *optimality_gap = args.ilp_gap;
            *max_tracts = args.ilp_max_tracts;
        }
        // Wire MKA parameters from CLI args
        if let SplitStrategy::MovingKnife {
            n_orientations,
            metric,
        } = &mut cfg.algo.split
        {
            *n_orientations = args.mka_orientations;
            *metric = args.mka_metric.clone();
        }
        if let Some(tol) = args.balance_tolerance {
            cfg.balance_tolerance = Some(tol / 100.0);
        }
        // Wire alpha_county into weights
        if args.alpha_county > 0.0 {
            cfg.algo.weights.alpha_county = args.alpha_county;
        }
        // Wire multiscale fine/coarse levels
        cfg.multiscale_fine = args.multiscale_fine.clone();
        cfg.multiscale_coarse = args.multiscale_coarse.clone();
        // Wire search strategy override
        if let Some(search) = args.search {
            use crate::args::SearchMode as SeM;
            let n = args.seeds.unwrap_or(args.geosection_seeds.max(50));
            cfg.algo.seeds = match search {
                SeM::Single => SeedCompositor::Single,
                SeM::Multi => SeedCompositor::Multi { seeds: n },
                SeM::Convergence => SeedCompositor::ConvergenceSweep {
                    threshold: args.convergence_threshold,
                },
                SeM::Percentile => SeedCompositor::Percentile {
                    p: args.percentile.clamp(0.0, 1.0),
                    seeds: n,
                },
                SeM::BisectionEnsemble => SeedCompositor::BisectionEnsemble {
                    p: args.percentile.clamp(0.0, 1.0),
                    ensemble_steps: args.ensemble_steps,
                },
                SeM::ShortBurst => SeedCompositor::ShortBurst {
                    burst_length: args.burst_length,
                    n_bursts: args.n_bursts,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::ShortBurstForest => SeedCompositor::ShortBurstForest {
                    burst_length: args.burst_length,
                    n_bursts: args.n_bursts,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::ShortBurstMergeSplit => SeedCompositor::ShortBurstMergeSplit {
                    burst_length: args.burst_length,
                    n_bursts: args.n_bursts,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::Flip => SeedCompositor::Flip {
                    flip_steps: args.flip_steps,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::ForestRecom => SeedCompositor::ForestRecom {
                    steps: args.forest_steps,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::MultiScale => SeedCompositor::MultiScale {
                    total_steps: args.multiscale_steps,
                    p: args.percentile.clamp(0.0, 1.0),
                    alpha: args.multiscale_alpha,
                },
                SeM::MergeSplit => SeedCompositor::MergeSplit {
                    steps: args.merge_split_steps,
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::MultiScaleAdaptive => SeedCompositor::MultiScaleAdaptive {
                    total_steps: args.multiscale_steps,
                    p: args.percentile.clamp(0.0, 1.0),
                    target_accept: args.ms_target_accept,
                    adapt_interval: args.ms_adapt_interval,
                },
                SeM::ParallelTempering => SeedCompositor::ParallelTempering {
                    n_replicas: args.pt_replicas,
                    swap_interval: args.pt_swap_interval,
                    cold_tolerance: args.pt_cold_tol,
                    hot_tolerance: args.pt_hot_tol,
                    steps: args.seeds.unwrap_or(1000),
                    p: args.percentile.clamp(0.0, 1.0),
                },
                SeM::VraRecom => SeedCompositor::VraRecom {
                    steps: args.seeds.unwrap_or(1000),
                    p: args.percentile.clamp(0.0, 1.0),
                    vap_threshold: args.vra_threshold,
                },
                SeM::SmcPercentile => SeedCompositor::SmcPercentile {
                    n_particles: args.particles.unwrap_or(5000),
                    p: args.percentile.clamp(0.0, 1.0),
                },
            };
        }
        cfg.smc_resample_threshold = args.smc_resample_threshold;
        // BEGIN reviewed bulk CVD override correction
        if let SplitStrategy::CentroidalVoronoi { n_iter, metric } = &mut cfg.algo.split {
            *n_iter = args.cvd_iters;
            *metric = args.cvd_metric.parse::<crate::bisection_runner::VoronoiMetric>()
                .unwrap_or(crate::bisection_runner::VoronoiMetric::GraphDistance);
        }
        // END reviewed bulk CVD override correction
        config
    }
}

use clap::{Parser,CommandFactory,FromArgMatches};
use serde_json::{json,Value};
fn bounded(arguments:&[String])->Result<(),String>{
 if arguments.len()>256||arguments.iter().any(|v|v.len()>4096||v.contains('\0'))||arguments.iter().map(String::len).sum::<usize>()>64*1024{return Err("States commands allow 256 bounded UTF-8 arguments, 4096 bytes each and 64 KiB total, without NUL bytes.".into());}Ok(())
}
pub fn parse(arguments:Vec<String>)->Result<StatesArgs,String>{bounded(&arguments)?;StatesArgs::try_parse_from(std::iter::once("bisect states".to_owned()).chain(arguments)).map_err(|e|e.to_string())}
pub fn inspect(arguments:Vec<String>)->Result<Value,String>{
 bounded(&arguments)?;
 let command=StatesArgs::command().bin_name("bisect states").color(clap::ColorChoice::Never);
 let matches=match command.clone().try_get_matches_from(std::iter::once("bisect states".to_owned()).chain(arguments.iter().cloned())){Ok(m)=>m,Err(e)=>return Ok(json!({"schema_version":"bisect-native-states-command-v1","status":if e.kind()==clap::error::ErrorKind::DisplayHelp{"help"}else{"parse-rejected"},"arguments":arguments,"target_pointer_width":usize::BITS,"parse_error_kind":format!("{:?}",e.kind()),"message":e.to_string(),"execution_performed":false,"filesystem_reads_performed":false}))};
 let args=StatesArgs::from_arg_matches(&matches).map_err(|e|e.to_string())?;
 let config=StatesConfiguration::from_args(&args);
 let profile=crate::state_args::validate_court_profile(args.profile,args.partition_mode,None,None,args.search,&None);
 let levels=crate::runner::validate_multiscale_levels(&args.multiscale_fine,&args.multiscale_coarse);
 let fields=command.get_arguments().filter(|a|a.get_id().as_str()!="help").map(|arg|{let id=arg.get_id().as_str();json!({"id":id,"long":arg.get_long(),"short":arg.get_short().map(|c|c.to_string()),"help":arg.get_help().map(|v|v.to_string()),"source":matches.value_source(id).map(|s|match s{clap::parser::ValueSource::CommandLine=>"command-line",clap::parser::ValueSource::DefaultValue=>"default",_=>"other"}).unwrap_or("absent"),"values":matches.get_raw(id).map(|values|values.map(|v|v.to_string_lossy().into_owned()).collect::<Vec<_>>()).unwrap_or_default()})}).collect::<Vec<_>>();
 let number=|v:f64|if v.is_finite(){json!(v)}else{json!({"numeric_class":if v.is_nan(){"nan"}else if v>0.0{"positive-infinity"}else{"negative-infinity"}})};
 Ok(json!({"schema_version":"bisect-native-states-command-v1","status":"parsed","arguments":arguments,"target_pointer_width":usize::BITS,"fields":fields,"year":args.year.to_string(),"states":args.states,"version":args.version,"output_dir":args.output_dir,"workers":args.workers.to_string(),"print_only":args.print_only,"profile_filter":{"allowed":profile.is_ok(),"error":profile.err()},"multiscale_levels":{"allowed":levels.is_ok(),"error":levels.err()},"effective_configuration":{"algorithm_debug":format!("{:?}",config.algo),"seeds_per_split":config.algo.seeds.seed_count().to_string(),"balance_tolerance_fraction":config.balance_tolerance.map(number),"multiscale_fine":config.multiscale_fine,"multiscale_coarse":config.multiscale_coarse,"smc_resample_threshold":number(config.smc_resample_threshold),"debug":config.debug,"reprocess":config.reprocess},"native_debug":format!("{args:?}"),"execution_performed":false,"filesystem_reads_performed":false,"scope":"Actual native States parser and bulk configuration, shared with StateConfig::apply_states_args. Bulk presets/search budgets differ from State arguments. Native profile and multiscale-level preflights are disclosed. This inspects configuration only: no sources, paths, workers, analysis, reset, output installation or partition runs. Native usize ranges and default METIS backend follow the loaded build target."}))
}
