//! Descriptive measurements using the same population and graph kernels as audits.
//! No legal profile, certificate or optimality claim is implied.
use super::*;

#[derive(Debug, Clone, Serialize)]
pub struct DistrictMeasurement {
    pub district_id: u32,
    pub units: usize,
    pub population: Option<PopulationWitness>,
    pub component_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubdivisionMeasurement {
    pub known_units: usize,
    pub missing_units: usize,
    pub splits: Vec<SplitWitness>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanMeasurements {
    pub districts: Vec<DistrictMeasurement>,
    pub total_population: Option<i64>,
    pub county: Option<SubdivisionMeasurement>,
    pub municipal: Option<SubdivisionMeasurement>,
}

pub fn measure_plan(
    plan: &DistrictPlan,
    context: Option<&RplanContext>,
) -> Result<PlanMeasurements, String> {
    plan.validate().map_err(|e| e.to_string())?;
    if let Some(ctx) = context {
        ctx.validate().map_err(|e| e.to_string())?;
        if ctx.units.unit_ids != plan.units.unit_ids
            || ctx.units.unit_kind != plan.units.unit_kind
            || ctx.units.state != plan.units.state
            || ctx.units.year != plan.units.year
            || ctx.units.unit_universe_hash != plan.units.unit_universe_hash
        {
            return Err("Plan/context ordered unit identity mismatch.".into());
        }
    }
    let populations = context
        .and_then(|c| c.populations.as_deref())
        .map(|p| population_witnesses(plan, p))
        .transpose()?;
    let total_population = populations
        .as_ref()
        .map(|w| w.iter().map(|w| w.population).sum());
    let districts = (0..plan.k)
        .map(|id| {
            let members: Vec<_> = plan
                .assignment
                .iter()
                .enumerate()
                .filter_map(|(i, &d)| (d as usize == id).then_some(i))
                .collect();
            DistrictMeasurement {
                district_id: id as u32,
                units: members.len(),
                population: populations.as_ref().map(|p| p[id].clone()),
                component_count: context
                    .and_then(|c| c.graph.as_ref())
                    .map(|g| district_components(&members, &g.adjacency).len()),
            }
        })
        .collect();
    let subdivision = |kind: &str| {
        context
            .and_then(|c| c.subdivisions.as_ref())
            .and_then(|s| subdivision_ids(s, kind))
            .map(|ids| {
                let known_units = ids.iter().filter(|id| id.is_some()).count();
                SubdivisionMeasurement {
                    known_units,
                    missing_units: ids.len() - known_units,
                    splits: split_witnesses(kind, ids, &plan.assignment)
                        .into_iter()
                        .filter_map(|w| {
                            if let Witness::Split(s) = w {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .collect(),
                }
            })
    };
    Ok(PlanMeasurements {
        districts,
        total_population,
        county: subdivision("county"),
        municipal: subdivision("municipal"),
    })
}

pub(super) fn population_witnesses(
    plan: &DistrictPlan,
    populations: &[i64],
) -> Result<Vec<PopulationWitness>, String> {
    if populations.len() != plan.assignment.len() || populations.iter().any(|&p| p < 0) {
        return Err("Invalid population coverage or value.".into());
    }
    let total = populations
        .iter()
        .try_fold(0i64, |s, &p| s.checked_add(p))
        .ok_or("Population sum exceeds i64.")?;
    let ideal = total as f64 / plan.k as f64;
    let mut totals = vec![0i64; plan.k];
    for (i, &d) in plan.assignment.iter().enumerate() {
        let slot = totals
            .get_mut(d as usize)
            .ok_or("Invalid district label.")?;
        *slot = slot
            .checked_add(populations[i])
            .ok_or("District population exceeds i64.")?;
    }
    Ok(totals
        .into_iter()
        .enumerate()
        .map(|(id, population)| {
            let absolute_deviation = (population as f64 - ideal).abs();
            PopulationWitness {
                district_id: id as u32,
                population,
                ideal,
                absolute_deviation,
                percent_deviation: if ideal == 0. {
                    0.
                } else {
                    absolute_deviation / ideal * 100.
                },
                deviation_ppm: if ideal == 0. {
                    0
                } else {
                    (absolute_deviation / ideal * 1_000_000.).round() as i64
                },
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (DistrictPlan, RplanContext) {
        (
            rplan_io::read_rplan_str(include_str!("../fixtures/grid3x3-valid.rplan"))
                .unwrap()
                .plan,
            rplan_io::read_rctx_str(include_str!("../fixtures/grid3x3.rctx")).unwrap(),
        )
    }
    #[test]
    fn measurements_reuse_audit_witnesses_and_preserve_missing_inputs() {
        let (p, c) = fixture();
        let m = measure_plan(&p, Some(&c)).unwrap();
        assert_eq!(m.districts.len(), 3);
        assert_eq!(
            m.total_population,
            Some(c.populations.as_ref().unwrap().iter().sum())
        );
        assert!(m
            .districts
            .iter()
            .all(|d| d.component_count == Some(1) && d.units == 3));
        let absent = measure_plan(&p, None).unwrap();
        assert!(absent.total_population.is_none());
        assert!(absent
            .districts
            .iter()
            .all(|d| d.population.is_none() && d.component_count.is_none()));
        let mut overflow = c.clone();
        overflow.populations = Some(vec![i64::MAX; 9]);
        assert!(measure_plan(&p, Some(&overflow))
            .unwrap_err()
            .contains("exceeds i64"));
        let mut mismatched = c.clone();
        mismatched.units.state = Some("RI".into());
        assert!(measure_plan(&p, Some(&mismatched)).is_err());
    }
    #[test]
    fn disconnected_empty_and_partial_memberships_are_descriptive() {
        let (mut p, mut c) = fixture();
        p.assignment = vec![0, 1, 0, 1, 0, 1, 0, 1, 0];
        p.allow_empty_districts = true;
        c.subdivisions = Some(SubdivisionContext {
            county_ids: Some(vec![Some("synthetic-county".into()); 9]),
            municipal_ids: None,
        });
        let ids = c
            .subdivisions
            .as_mut()
            .unwrap()
            .county_ids
            .as_mut()
            .unwrap();
        ids[0] = None;
        let m = measure_plan(&p, Some(&c)).unwrap();
        assert!(m.districts[0].component_count.unwrap() > 1);
        assert_eq!(m.districts[2].component_count, Some(0));
        assert_eq!(m.county.unwrap().missing_units, 1);
    }
}
