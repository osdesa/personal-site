//! Shared validation for links imported from external content sources.

use url::Url;

pub(crate) fn is_safe_https_url(value: &str) -> bool {
    let Ok(url) = Url::parse(value) else {
        return false;
    };

    url.scheme() == "https"
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
}

pub(crate) fn is_safe_mailto_url(value: &str) -> bool {
    let Some(address) = value.strip_prefix("mailto:") else {
        return false;
    };
    if address.is_empty()
        || address.contains(['?', '#', '%'])
        || address
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        return false;
    }

    let Some((local, domain)) = address.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}
