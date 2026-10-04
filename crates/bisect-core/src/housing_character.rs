#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcsHousingRaw {
    pub total_units: f64,
    pub single_family_detached: f64,
    pub single_family_attached: f64,
    pub multifamily_10_19: f64,
    pub multifamily_20_49: f64,
    pub multifamily_50_plus: f64,
    pub occupied_total: f64,
    pub owner_occupied: f64,
    pub median_year_built: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HousingChar {
    pub pct_single_family: f64,
    pub pct_multifamily: f64,
    pub pct_owner: f64,
    pub housing_vintage: f64,
}

impl HousingChar {
    pub fn neutral() -> Self {
        Self {
            pct_single_family: 0.5,
            pct_multifamily: 0.5,
            pct_owner: 0.5,
            housing_vintage: 0.5,
        }
    }
}

pub fn derive_housing_character(raw: AcsHousingRaw) -> HousingChar {
    let total_units = raw.total_units.max(0.0);
    let sf_units = raw.single_family_detached.max(0.0) + raw.single_family_attached.max(0.0);
    let mf_units = raw.multifamily_10_19.max(0.0)
        + raw.multifamily_20_49.max(0.0)
        + raw.multifamily_50_plus.max(0.0);
    let pct_single_family = if total_units > 0.0 {
        (sf_units / total_units).clamp(0.0, 1.0)
    } else {
        0.5
    };
    let pct_multifamily = if total_units > 0.0 {
        (mf_units / total_units).clamp(0.0, 1.0)
    } else {
        0.5
    };

    let occupied_total = raw.occupied_total.max(0.0);
    let owner_occupied = raw.owner_occupied.max(0.0);
    let pct_owner = if occupied_total > 0.0 {
        (owner_occupied / occupied_total).clamp(0.0, 1.0)
    } else {
        0.5
    };

    let housing_vintage = if raw.median_year_built < -600_000_000.0 || raw.median_year_built == 0.0
    {
        0.5
    } else {
        let v = 1.0 - (raw.median_year_built - 1940.0) / (2020.0 - 1940.0);
        v.clamp(0.0, 1.0)
    };

    HousingChar {
        pct_single_family,
        pct_multifamily,
        pct_owner,
        housing_vintage,
    }
}

pub fn cosine_similarity(a: &HousingChar, b: &HousingChar) -> f64 {
    let dot = a.pct_single_family * b.pct_single_family
        + a.pct_multifamily * b.pct_multifamily
        + a.pct_owner * b.pct_owner
        + a.housing_vintage * b.housing_vintage;
    let mag_a = (a.pct_single_family.powi(2)
        + a.pct_multifamily.powi(2)
        + a.pct_owner.powi(2)
        + a.housing_vintage.powi(2))
    .sqrt();
    let mag_b = (b.pct_single_family.powi(2)
        + b.pct_multifamily.powi(2)
        + b.pct_owner.powi(2)
        + b.housing_vintage.powi(2))
    .sqrt();

    if mag_a < 1e-15 || mag_b < 1e-15 {
        1.0
    } else {
        (dot / (mag_a * mag_b)).clamp(0.0, 1.0)
    }
}

/// Shared geographic/similarity blend; callers validate weight and alpha.
pub fn blend_edge(weight: f64, a: &HousingChar, b: &HousingChar, alpha: f64) -> f64 {
    weight * (alpha + (1.0 - alpha) * cosine_similarity(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_counts_and_vintage_reference() {
        let raw = AcsHousingRaw {
            total_units: 100.,
            single_family_detached: 40.,
            single_family_attached: 10.,
            multifamily_10_19: 10.,
            multifamily_20_49: 20.,
            multifamily_50_plus: 10.,
            occupied_total: 80.,
            owner_occupied: 60.,
            median_year_built: 1980.,
        };
        assert_eq!(
            derive_housing_character(raw),
            HousingChar {
                pct_single_family: 0.5,
                pct_multifamily: 0.4,
                pct_owner: 0.75,
                housing_vintage: 0.5
            }
        );
        assert_eq!(
            derive_housing_character(AcsHousingRaw {
                total_units: 0.,
                occupied_total: 0.,
                median_year_built: -666_666_666.,
                ..raw
            }),
            HousingChar::neutral()
        );
        assert_eq!(
            derive_housing_character(AcsHousingRaw {
                median_year_built: 1930.,
                ..raw
            })
            .housing_vintage,
            1.
        );
        assert_eq!(
            derive_housing_character(AcsHousingRaw {
                median_year_built: 2030.,
                ..raw
            })
            .housing_vintage,
            0.
        );
    }
    #[test]
    fn housing_zero_policy_differs_from_economic() {
        let a = HousingChar {
            pct_single_family: 1.,
            pct_multifamily: 0.,
            pct_owner: 0.,
            housing_vintage: 0.,
        };
        let b = HousingChar {
            pct_single_family: 0.,
            pct_multifamily: 1.,
            pct_owner: 0.,
            housing_vintage: 0.,
        };
        let zero = HousingChar {
            pct_single_family: 0.,
            pct_multifamily: 0.,
            pct_owner: 0.,
            housing_vintage: 0.,
        };
        assert_eq!(cosine_similarity(&a, &b), 0.);
        assert_eq!(blend_edge(100., &a, &b, 0.5), 50.);
        assert_eq!(blend_edge(100., &zero, &a, 0.5), 100.);
        assert_eq!(blend_edge(100., &a, &b, 1.), 100.);
    }
}
