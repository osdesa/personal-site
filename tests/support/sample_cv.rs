use std::{borrow::Cow, sync::LazyLock};

use personal_site::cv::{
    ContactDetails, Cv, CvDate, DateRange, DateRangeEnd, Education, Experience, Inline, Location,
    Month, Profile, ProfileWebsite, Project, RichText, SkillGroup, SocialLink, SocialPlatform,
};

pub fn sample_cv() -> &'static Cv<'static> {
    static SAMPLE_CV: LazyLock<Cv<'static>> = LazyLock::new(|| Cv {
        profile: Profile {
            full_name: Cow::Borrowed("Sample Candidate"),
            contact: ContactDetails {
                email: Cow::Borrowed("candidate@example.test"),
            },
            social_links: Cow::Owned(vec![
                SocialLink {
                    platform: SocialPlatform::LinkedIn,
                    url: Cow::Borrowed("https://example.test/profiles/professional"),
                    label: text("Professional profile"),
                },
                SocialLink {
                    platform: SocialPlatform::GitHub,
                    url: Cow::Borrowed("https://example.test/profiles/code"),
                    label: text("Code profile"),
                },
            ]),
            website: Some(ProfileWebsite {
                url: Cow::Borrowed("https://example.test/portfolio"),
                label: text("Portfolio"),
            }),
        },
        education: Cow::Owned(vec![
            Education {
                institution: text("First Example Institute"),
                qualification: text("Example qualification one"),
                location: location("First City"),
                dates: completed_range(2020, 2022),
            },
            Education {
                institution: text("Second Example Institute"),
                qualification: text("Example qualification two"),
                location: location("Second City"),
                dates: completed_range(2022, 2024),
            },
        ]),
        experience: Cow::Owned(vec![Experience {
            role: text("Example role"),
            organisation: text("Example organisation"),
            location: location("Work City"),
            dates: DateRange {
                start: CvDate {
                    year: 2024,
                    month: Month::January,
                },
                end: DateRangeEnd::Present,
            },
            highlights: Cow::Owned(vec![text("Delivered an example outcome.")]),
        }]),
        projects: Cow::Owned(vec![Project {
            title: text("CV-only example project"),
            technologies: Cow::Owned(vec![Cow::Borrowed("Example technology")]),
            period: Some(text("Example period")),
            highlights: Cow::Owned(vec![text(
                "Preserved by parsing but omitted from this page.",
            )]),
        }]),
        skills: Cow::Owned(vec![SkillGroup {
            category: Cow::Borrowed("Example skills"),
            skills: Cow::Owned(vec![
                Cow::Borrowed("First skill"),
                Cow::Borrowed("Second skill"),
            ]),
        }]),
    });

    LazyLock::force(&SAMPLE_CV)
}

fn text(value: &'static str) -> RichText<'static> {
    RichText {
        nodes: Cow::Owned(vec![Inline::Text(Cow::Borrowed(value))]),
    }
}

fn location(city: &'static str) -> Location<'static> {
    Location {
        city: Cow::Borrowed(city),
        country: Cow::Borrowed("Example Country"),
    }
}

fn completed_range(start_year: u16, end_year: u16) -> DateRange {
    DateRange {
        start: CvDate {
            year: start_year,
            month: Month::September,
        },
        end: DateRangeEnd::Date(CvDate {
            year: end_year,
            month: Month::June,
        }),
    }
}
