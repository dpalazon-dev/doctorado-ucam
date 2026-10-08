use research_workbench_core::transport::dto::*;
#[test]
fn optional_numbers_reject_explicit_null() {
    for total in [None, Some(serde_json::json!(4))] {
        let mut page = serde_json::json!({"items":[],"nextCursor":null});
        if let Some(total) = total {
            page["total"] = total;
        }
        assert!(serde_json::from_value::<PageDto<String>>(page).is_ok());
    }
    assert!(
        serde_json::from_value::<PageDto<String>>(
            serde_json::json!({"items":[],"nextCursor":null,"total":null})
        )
        .is_err()
    );
    let args =
        serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000001","phaseCode":"PRE"});
    assert!(serde_json::from_value::<WorkflowGetPhaseDefinitionArgs>(args.clone()).is_ok());
    let mut numbered = args.clone();
    numbered["version"] = serde_json::json!(1);
    assert!(serde_json::from_value::<WorkflowGetPhaseDefinitionArgs>(numbered).is_ok());
    let mut null = args;
    null["version"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<WorkflowGetPhaseDefinitionArgs>(null).is_err());
}
#[test]
fn concept_domain_patch_preserves_three_states() {
    let base = serde_json::json!({"requestId":"00000000-0000-4000-8000-000000000001","conceptId":"00000000-0000-4000-8000-000000000002","expectedRevision":0});
    for domain in [
        None,
        Some(serde_json::Value::Null),
        Some(serde_json::json!("agua")),
    ] {
        let mut input = base.clone();
        if let Some(value) = &domain {
            input["domain"] = value.clone();
        }
        let typed: ConceptUpdateArgs = serde_json::from_value(input.clone()).unwrap();
        assert_eq!(typed.domain, domain.map(|v| v.as_str().map(str::to_owned)));
        assert_eq!(serde_json::to_value(typed).unwrap(), input);
    }
}
#[test]
fn ipc_envelope_camel_case_roundtrip() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../contracts/fixtures/app-info.json")).unwrap();
    let typed: IpcResult<AppInfoDto> = serde_json::from_value(fixture.clone()).unwrap();
    assert_eq!(serde_json::to_value(typed).unwrap(), fixture);
    let data: PaperMetadataInput =
        serde_json::from_str(include_str!("../../contracts/fixtures/metadata.json")).unwrap();
    assert!(data.authors[1].len() > 500);
    let preview: ImportPreviewDto =
        serde_json::from_str(include_str!("../../contracts/fixtures/import-preview.json")).unwrap();
    assert_eq!(preview.size_bytes, 524288000);
}
#[test]
fn unknown_enum_and_payload_rejected() {
    assert!(serde_json::from_str::<ReviewType>("\"invented\"").is_err());
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../contracts/fixtures/metadata.json")).unwrap();
    v["extra"] = serde_json::json!(true);
    assert!(serde_json::from_value::<PaperMetadataInput>(v).is_err());
    assert!(serde_json::from_str::<ReadingPositionDto>("{\"documentId\":\"00000000-0000-4000-8000-000000000001\",\"pageIndex\":1,\"zoom\":NaN,\"revision\":0,\"updatedAt\":\"2026-10-01T12:00:00Z\"}").is_err());
}
#[test]
fn required_nullable_fields_cannot_be_omitted() {
    let mut v: serde_json::Value =
        serde_json::from_str(include_str!("../../contracts/fixtures/metadata.json")).unwrap();
    v.as_object_mut().unwrap().remove("year");
    assert!(serde_json::from_value::<PaperMetadataInput>(v).is_err());
}

#[test]
fn closed_registry_matches_normative_contract_and_permissions() {
    let source = include_str!("../../docs/architecture/CONTRACTS.md");
    let section = source
        .split("## Registro de comandos Tauri")
        .nth(1)
        .unwrap()
        .split("\n##")
        .next()
        .unwrap();
    let mut expected: Vec<&str> = section
        .split('`')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, s)| s)
        .filter(|s| s.contains('_'))
        .collect();
    expected.sort();
    let mut actual = research_workbench_core::transport::commands::COMMANDS.to_vec();
    actual.sort();
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 63);
    let files = [
        include_str!("../permissions/research-read.toml"),
        include_str!("../permissions/research-write.toml"),
        include_str!("../permissions/research-maintenance.toml"),
    ];
    let mut permitted = Vec::new();
    for file in files {
        let array = file
            .lines()
            .find_map(|l| l.strip_prefix("commands.allow = "))
            .unwrap();
        let values: Vec<String> = serde_json::from_str(array).unwrap();
        permitted.extend(values);
    }
    permitted.sort();
    assert_eq!(permitted, expected);
}
