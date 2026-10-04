/// Economic character summary for a single census tract.
///
/// All fields are fractions of total jobs (`c000`).
/// Zero-job tracts (purely residential) have all fields = 0.0.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EconChar {
    /// Commercial intensity: (CNS07+CNS09+CNS10+CNS11) / C000
    pub commercial_intensity: f64,
    /// Industrial fraction: (CNS01+CNS02+CNS05+CNS08) / C000
    pub industrial_fraction: f64,
    /// Native job intensity proxy: C000 / 10,000, capped at 1.0.
    /// This is not jobs divided by resident population.
    pub jobs_per_resident: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LodesWacRaw {
    pub c000: f64,
    pub cns01: f64,
    pub cns02: f64,
    pub cns05: f64,
    pub cns07: f64,
    pub cns08: f64,
    pub cns09: f64,
    pub cns10: f64,
    pub cns11: f64,
}

impl EconChar {
    /// The zero vector — represents a pure residential (zero-job) tract.
    pub fn zero() -> Self {
        Self {
            commercial_intensity: 0.0,
            industrial_fraction: 0.0,
            jobs_per_resident: 0.0,
        }
    }
}

pub fn derive_economic_character(raw: LodesWacRaw) -> EconChar {
    let c000 = raw.c000.max(0.0);
    if c000 < 1e-10 {
        return EconChar::zero();
    }

    let commercial =
        raw.cns07.max(0.0) + raw.cns09.max(0.0) + raw.cns10.max(0.0) + raw.cns11.max(0.0);
    let industrial =
        raw.cns01.max(0.0) + raw.cns02.max(0.0) + raw.cns05.max(0.0) + raw.cns08.max(0.0);
    EconChar {
        commercial_intensity: (commercial / c000).clamp(0.0, 1.0),
        industrial_fraction: (industrial / c000).clamp(0.0, 1.0),
        jobs_per_resident: (c000 / 10_000.0_f64).min(1.0),
    }
}

// ---------------------------------------------------------------------------
// cosine_similarity
// ---------------------------------------------------------------------------

/// Cosine similarity between two `EconChar` vectors, in [0.0, 1.0].
///
/// Special cases:
/// - Both zero (both residential) → 1.0 (maximally similar)
/// - One zero, one non-zero → 0.5 (neutral — one residential, one not)
/// - Otherwise: dot / (|a| * |b|), clamped to [0.0, 1.0]
pub fn cosine_similarity(a: &EconChar, b: &EconChar) -> f64 {
    let dot = a.commercial_intensity * b.commercial_intensity
        + a.industrial_fraction * b.industrial_fraction
        + a.jobs_per_resident * b.jobs_per_resident;

    let mag_a = (a.commercial_intensity.powi(2)
        + a.industrial_fraction.powi(2)
        + a.jobs_per_resident.powi(2))
    .sqrt();

    let mag_b = (b.commercial_intensity.powi(2)
        + b.industrial_fraction.powi(2)
        + b.jobs_per_resident.powi(2))
    .sqrt();

    if mag_a < 1e-15 && mag_b < 1e-15 {
        // Both zero → both residential → maximally similar
        1.0
    } else if mag_a < 1e-15 || mag_b < 1e-15 {
        // One residential, one not → neutral
        0.5
    } else {
        (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
    }
}

/// Preserve the geographic component by alpha; scale the remainder by similarity.
/// Callers validate finite nonnegative base weights and alpha in [0,1].
pub fn blend_edge(weight: f64, a: &EconChar, b: &EconChar, alpha: f64) -> f64 {
    weight * (alpha + (1.0 - alpha) * cosine_similarity(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_job_proxy_and_sector_totals() {
        let raw = LodesWacRaw {
            c000: 1000.,
            cns01: 10.,
            cns02: 20.,
            cns05: 30.,
            cns07: 400.,
            cns08: 40.,
            cns09: 100.,
            cns10: 50.,
            cns11: 50.,
        };
        assert_eq!(
            derive_economic_character(raw),
            EconChar {
                commercial_intensity: 0.6,
                industrial_fraction: 0.1,
                jobs_per_resident: 0.1
            }
        );
        assert_eq!(
            derive_economic_character(LodesWacRaw { c000: 0., ..raw }),
            EconChar::zero()
        );
        assert_eq!(
            derive_economic_character(LodesWacRaw {
                c000: 20_000.,
                cns07: 40_000.,
                ..raw
            })
            .commercial_intensity,
            1.
        );
        assert_eq!(
            derive_economic_character(LodesWacRaw {
                c000: 20_000.,
                ..raw
            })
            .jobs_per_resident,
            1.
        );
    }
    #[test]
    fn residential_zero_policy_and_alpha_endpoints() {
        let a = EconChar {
            commercial_intensity: 1.,
            industrial_fraction: 0.,
            jobs_per_resident: 0.,
        };
        let b = EconChar {
            commercial_intensity: 0.,
            industrial_fraction: 1.,
            jobs_per_resident: 0.,
        };
        assert_eq!(cosine_similarity(&a, &b), 0.);
        assert_eq!(blend_edge(100., &a, &b, 0.5), 50.);
        assert_eq!(blend_edge(100., &a, &b, 0.), 0.);
        assert_eq!(blend_edge(100., &a, &b, 1.), 100.);
        assert_eq!(blend_edge(100., &EconChar::zero(), &a, 0.5), 75.);
        assert_eq!(
            blend_edge(100., &EconChar::zero(), &EconChar::zero(), 0.),
            100.
        );
    }
}
