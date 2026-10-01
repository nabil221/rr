//! Deterministic synthetic computation used to prove the execution path.

use std::collections::BTreeMap;

use crate::{GroupSummary, RunConfiguration, RunResult, ValidationMessage};

const RECORD_COUNT: u32 = 24;
const REGIONS: [&str; 3] = ["north", "south", "west"];
const CATEGORIES: [&str; 3] = ["amber", "blue", "green"];
const LCG_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
const LCG_INCREMENT: u64 = 1_442_695_040_888_963_407;

/// Generates records from the configuration seed, filters by region, and summarizes scores.
///
/// Records are generated with a fixed 64-bit linear congruential sequence. Each record consumes
/// three sequence values for region, category, and an integer score in `20..=100`. Matching rows
/// are grouped by category in alphabetical order. The threshold is a minimum average score.
///
/// # Errors
///
/// Returns one validation message for an intentional rejection, no matching records, or an
/// average score below the configured threshold. No infrastructure errors are possible.
pub fn calculate(configuration: &RunConfiguration) -> Result<RunResult, Vec<ValidationMessage>> {
    if configuration.force_validation_failure {
        return Err(vec![ValidationMessage {
            code: "forced-rejection".to_owned(),
            message: "Run rejected by the requested validation demonstration".to_owned(),
            field: Some("force_validation_failure".to_owned()),
        }]);
    }

    let mut state = configuration.seed;
    let mut total_score = 0.0;
    let mut matched_records = 0;
    let mut groups: BTreeMap<&'static str, (u32, f64)> = BTreeMap::new();

    for _ in 0..RECORD_COUNT {
        let region = next_index(&mut state, REGIONS.len());
        let category = CATEGORIES[next_index(&mut state, CATEGORIES.len())];
        let score = f64::from(20_u8 + next_byte(&mut state) % 81);

        if REGIONS[region] == configuration.region {
            matched_records += 1;
            total_score += score;
            let group = groups.entry(category).or_default();
            group.0 += 1;
            group.1 += score;
        }
    }

    if matched_records == 0 {
        return Err(vec![ValidationMessage {
            code: "no-matching-records".to_owned(),
            message: format!(
                "No synthetic records matched region '{}'",
                configuration.region
            ),
            field: Some("region".to_owned()),
        }]);
    }

    let average_score = total_score / f64::from(matched_records);
    if average_score < configuration.threshold {
        return Err(vec![ValidationMessage {
            code: "threshold-not-met".to_owned(),
            message: format!(
                "Average score {average_score:.2} is below the required threshold {:.2}",
                configuration.threshold
            ),
            field: Some("threshold".to_owned()),
        }]);
    }

    Ok(RunResult {
        total_records: RECORD_COUNT,
        matched_records,
        total_score,
        average_score,
        groups: groups
            .into_iter()
            .map(|(category, (count, score))| GroupSummary {
                category: category.to_owned(),
                matched_records: count,
                total_score: score,
                average_score: score / f64::from(count),
            })
            .collect(),
    })
}

fn next_index(state: &mut u64, count: usize) -> usize {
    usize::from(next_byte(state)) % count
}

fn next_byte(state: &mut u64) -> u8 {
    *state = state
        .wrapping_mul(LCG_MULTIPLIER)
        .wrapping_add(LCG_INCREMENT);
    state.to_le_bytes()[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configuration(
        seed: u64,
        region: &str,
        threshold: f64,
        force_failure: bool,
    ) -> RunConfiguration {
        RunConfiguration::new(seed, region, threshold, force_failure).expect("valid config")
    }

    #[test]
    fn calculation_is_deterministic_for_a_known_configuration() {
        let config = configuration(42, "north", 0.0, false);

        let first = calculate(&config).expect("north has matching records");
        let second = calculate(&config).expect("same configuration succeeds");
        assert_eq!(first, second);
        assert_eq!(
            first,
            RunResult {
                total_records: 24,
                matched_records: 7,
                total_score: 449.0,
                average_score: 449.0 / 7.0,
                groups: vec![
                    GroupSummary {
                        category: "amber".to_owned(),
                        matched_records: 3,
                        total_score: 215.0,
                        average_score: 215.0 / 3.0,
                    },
                    GroupSummary {
                        category: "blue".to_owned(),
                        matched_records: 1,
                        total_score: 75.0,
                        average_score: 75.0,
                    },
                    GroupSummary {
                        category: "green".to_owned(),
                        matched_records: 3,
                        total_score: 159.0,
                        average_score: 53.0,
                    },
                ],
            }
        );
    }

    #[test]
    fn intentional_rejection_is_a_validation_result() {
        let error = calculate(&configuration(42, "north", 0.0, true))
            .expect_err("forced failure should reject");

        assert_eq!(error[0].code, "forced-rejection");
        assert_eq!(error[0].field.as_deref(), Some("force_validation_failure"));
    }

    #[test]
    fn threshold_and_empty_region_return_controlled_validation() {
        let threshold_error = calculate(&configuration(42, "north", 100.0, false))
            .expect_err("threshold above generated scores should reject");
        assert_eq!(threshold_error[0].code, "threshold-not-met");

        let region_error = calculate(&configuration(42, "east", 0.0, false))
            .expect_err("unknown region should match no records");
        assert_eq!(region_error[0].code, "no-matching-records");
    }
}
