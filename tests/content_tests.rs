use personal_site::content::portfolio;

#[test]
fn homepage_editorial_content_is_complete() {
    let content = portfolio();

    assert!(!content.profile.role.trim().is_empty());
    assert!(!content.profile.home_intro.trim().is_empty());
}
