//! Strongly typed curriculum-vitae domain data shared by import and presentation.
//!
//! The model contains semantic values and a deliberately small rich-text tree.
//! It cannot carry arbitrary HTML. Imported data owns its values, while the
//! generated website module borrows static values through [`Cow`].

use std::borrow::Cow;

/// A fully validated CV document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cv<'a> {
    /// Personal and contact information from the document heading.
    pub profile: Profile<'a>,
    /// Education entries in source order.
    pub education: Cow<'a, [Education<'a>]>,
    /// Professional roles in source order.
    pub experience: Cow<'a, [Experience<'a>]>,
    /// Portfolio projects in source order.
    pub projects: Cow<'a, [Project<'a>]>,
    /// Technical skill groups in source order.
    pub skills: Cow<'a, [SkillGroup<'a>]>,
}

/// Personal identity, direct contact details, and professional profiles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile<'a> {
    /// Full display name.
    pub full_name: Cow<'a, str>,
    /// Direct contact details.
    pub contact: ContactDetails<'a>,
    /// Supported social profiles in heading order.
    pub social_links: Cow<'a, [SocialLink<'a>]>,
}

/// Direct, non-social contact details.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactDetails<'a> {
    /// Email address without the `mailto:` scheme.
    pub email: Cow<'a, str>,
}

/// A supported professional social platform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocialPlatform {
    /// GitHub profile.
    GitHub,
    /// LinkedIn profile.
    LinkedIn,
}

/// A validated social link and its authored label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SocialLink<'a> {
    /// Recognised platform.
    pub platform: SocialPlatform,
    /// Absolute HTTPS profile URL.
    pub url: Cow<'a, str>,
    /// Safe structured label.
    pub label: RichText<'a>,
}

/// One education record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Education<'a> {
    /// Institution name.
    pub institution: RichText<'a>,
    /// Qualification or studied subjects.
    pub qualification: RichText<'a>,
    /// Institution location.
    pub location: Location<'a>,
    /// Attendance dates.
    pub dates: DateRange,
}

/// One professional role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Experience<'a> {
    /// Role title.
    pub role: RichText<'a>,
    /// Employing organisation.
    pub organisation: RichText<'a>,
    /// Role location.
    pub location: Location<'a>,
    /// Employment dates.
    pub dates: DateRange,
    /// Accomplishments and responsibilities in source order.
    pub highlights: Cow<'a, [RichText<'a>]>,
}

/// One project record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project<'a> {
    /// Authored project title, including any safe link or emphasis nodes.
    pub title: RichText<'a>,
    /// Technologies parsed from the project heading.
    pub technologies: Cow<'a, [Cow<'a, str>]>,
    /// Optional authored project period or status.
    pub period: Option<RichText<'a>>,
    /// Project outcomes and details in source order.
    pub highlights: Cow<'a, [RichText<'a>]>,
}

/// One technical skill category.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillGroup<'a> {
    /// Authored category name.
    pub category: Cow<'a, str>,
    /// Individual skills in source order.
    pub skills: Cow<'a, [Cow<'a, str>]>,
}

/// A city and country pair parsed from the CV's location convention.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Location<'a> {
    /// City or locality.
    pub city: Cow<'a, str>,
    /// Country label as authored.
    pub country: Cow<'a, str>,
}

/// A month-precision calendar date.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CvDate {
    /// Four-digit Gregorian year.
    pub year: u16,
    /// Calendar month.
    pub month: Month,
}

/// An inclusive authored date range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateRange {
    /// Starting month.
    pub start: CvDate,
    /// Ending month or an explicitly current role.
    pub end: DateRangeEnd,
}

/// The end of a date range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DateRangeEnd {
    /// A concrete ending month.
    Date(CvDate),
    /// The source says `Present`.
    Present,
}

/// Calendar month.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

/// Safe inline content represented without HTML.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RichText<'a> {
    /// Inline nodes in authored order.
    pub nodes: Cow<'a, [Inline<'a>]>,
}

/// One supported inline content node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Inline<'a> {
    /// Plain text.
    Text(Cow<'a, str>),
    /// Strong importance from `\textbf`.
    Strong(RichText<'a>),
    /// Emphasis from `\emph` or `\textit`.
    Emphasis(RichText<'a>),
    /// Underlining from `\underline`.
    Underline(RichText<'a>),
    /// An absolute HTTPS or `mailto:` link from `\href`.
    Link {
        /// Validated link destination.
        target: Cow<'a, str>,
        /// Safe structured link label.
        label: RichText<'a>,
    },
}
