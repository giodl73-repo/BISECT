// Frozen pre-extraction States configuration path. No filesystem execution.
// Original block SHA256: 419b0b766bc07717f7f2ef9c465e55c57135d407c93e69e8cafea0286c60206f
use bisect_cli::runner::AlgorithmConfig;
pub fn legacy_apply(cfg: &mut bisect_cli::runner::StateConfig,args: &bisect_cli::args::StatesArgs){
cfg.debug = args.debug;
cfg.reprocess = args.reprocess;
cfg.algo = AlgorithmConfig::defaults_for_mode(&args.partition_mode);
// Override seeds from CLI if non-default
use bisect_cli::runner::{SeedCompositor, SplitStrategy};
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
    use bisect_cli::args::SearchMode as SeM;
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
}
