use crate::transport::{
    dto::PaperMetadataInput,
    error::{AppError, ErrorCode},
};

pub fn normalize_metadata(
    mut metadata: PaperMetadataInput,
) -> Result<PaperMetadataInput, AppError> {
    metadata.title = metadata.title.trim().to_owned();
    if metadata.title.is_empty()
        || metadata.title.chars().count() > 1000
        || metadata.authors.len() > 100
    {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    for author in &mut metadata.authors {
        *author = author.trim().to_owned();
        if author.is_empty() {
            return Err(AppError::new(ErrorCode::InvalidInput));
        }
    }
    if metadata
        .year
        .is_some_and(|year| !(1000..=9999).contains(&year))
    {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    metadata.doi = metadata.doi.map(normalize_doi).transpose()?;
    metadata.venue = clean_optional(metadata.venue);
    metadata.domain = clean_optional(metadata.domain);
    Ok(metadata)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let s = s.trim().to_owned();
        (!s.is_empty()).then_some(s)
    })
}

pub fn normalize_doi(input: String) -> Result<String, AppError> {
    let mut doi = input.trim().to_ascii_lowercase();
    if let Some(rest) = doi.strip_prefix("doi:") {
        doi = rest.trim().to_owned();
    }
    for prefix in ["https://doi.org/", "http://doi.org/"] {
        if let Some(rest) = doi.strip_prefix(prefix) {
            doi = rest.to_owned();
            break;
        }
    }
    if !valid_doi(&doi) {
        return Err(AppError::new(ErrorCode::InvalidInput));
    }
    Ok(doi)
}

fn valid_doi(value: &str) -> bool {
    let Some((registrant, suffix)) = value
        .strip_prefix("10.")
        .and_then(|rest| rest.split_once('/'))
    else {
        return false;
    };
    (4..=9).contains(&registrant.len())
        && registrant.bytes().all(|b| b.is_ascii_digit())
        && !suffix.trim().is_empty()
        && !suffix.chars().any(char::is_whitespace)
}
