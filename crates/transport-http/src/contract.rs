//! Deterministic, explicit generation from the actual Serde HTTP types.

use super::{
    ConfigurationDto, CreateRunRequest, ErrorDto, EventDto, EventKindDto, GroupDto, HealthDto,
    ResultDto, RunDto, StatusDto, ValidationMessageDto,
};
use ts_rs::{Config, TS};

/// Returns the complete browser/native JSON contract from the HTTP DTOs.
#[must_use]
pub fn typescript_contract() -> String {
    let config = Config::default();
    let declarations = [
        CreateRunRequest::decl(&config),
        ConfigurationDto::decl(&config),
        StatusDto::decl(&config),
        EventKindDto::decl(&config),
        GroupDto::decl(&config),
        ResultDto::decl(&config),
        ValidationMessageDto::decl(&config),
        EventDto::decl(&config),
        RunDto::decl(&config),
        HealthDto::decl(&config),
        ErrorDto::decl(&config),
    ];
    let mut output = String::from(
        "// Generated from Rust HTTP DTOs. Run npm run contract:generate; do not edit.\n\n",
    );
    for declaration in declarations {
        output.push_str("export ");
        output.push_str(&declaration);
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn checked_in_contract_matches_http_dtos() {
        assert_eq!(
            include_str!("../../../apps/web/src/generated/api-contract.ts").replace("\r\n", "\n"),
            super::typescript_contract(),
            "Run npm run contract:generate, then review the client changes"
        );
    }
}
