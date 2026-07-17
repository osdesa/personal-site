use std::{borrow::Cow, fmt};

use crate::cv::{
    ContactDetails, Cv, CvDate, DateRange, DateRangeEnd, Education, Experience, Inline, Location,
    Month, Profile, Project, RichText, SkillGroup, SocialLink, SocialPlatform,
};

const DOCUMENT_START: &str = "\\begin{document}";
const DOCUMENT_END: &str = "\\end{document}";
const REQUIRED_CUSTOM_COMMAND_DECLARATIONS: &[(&str, &str)] = &[
    ("resumeItem", "\\newcommand{\\resumeItem}[1]"),
    ("resumeSubheading", "\\newcommand{\\resumeSubheading}[4]"),
    (
        "resumeSubSubheading",
        "\\newcommand{\\resumeSubSubheading}[2]",
    ),
    (
        "resumeProjectHeading",
        "\\newcommand{\\resumeProjectHeading}[2]",
    ),
    ("resumeSubItem", "\\newcommand{\\resumeSubItem}[1]"),
    (
        "resumeSubHeadingListStart",
        "\\newcommand{\\resumeSubHeadingListStart}{",
    ),
    (
        "resumeSubHeadingListEnd",
        "\\newcommand{\\resumeSubHeadingListEnd}{",
    ),
    (
        "resumeItemListStart",
        "\\newcommand{\\resumeItemListStart}{",
    ),
    ("resumeItemListEnd", "\\newcommand{\\resumeItemListEnd}{"),
];

/// A deterministic CV parse failure with a source location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CvParseError {
    line: usize,
    column: usize,
    message: String,
}

impl CvParseError {
    /// One-based source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// One-based source column.
    #[must_use]
    pub const fn column(&self) -> usize {
        self.column
    }

    /// Human-readable diagnostic without the location prefix.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for CvParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for CvParseError {}

/// Parses the supported `osdesa/cv` LaTeX document into an owned domain model.
///
/// This intentionally recognises only the documented CV grammar. The preamble
/// must declare the expected custom commands, and the document body is consumed
/// completely so format drift cannot silently discard content.
pub fn parse_cv(source: &str) -> Result<Cv<'static>, CvParseError> {
    let source = strip_comments(source);
    let document_start = source
        .find(DOCUMENT_START)
        .ok_or_else(|| error_at(&source, 0, format!("missing {DOCUMENT_START}")))?;
    let preamble = &source[..document_start];
    for (command, declaration) in REQUIRED_CUSTOM_COMMAND_DECLARATIONS {
        if !preamble.contains(declaration) {
            return Err(error_at(
                &source,
                document_start,
                format!("preamble is missing the expected {command} declaration {declaration}"),
            ));
        }
    }

    let mut parser = Parser::new(&source, document_start + DOCUMENT_START.len(), source.len());
    let profile = parser.parse_profile()?;
    parser.expect_section("Education")?;
    let education = parser.parse_education()?;
    parser.expect_section("Experience")?;
    let experience = parser.parse_experience()?;
    parser.expect_section("Projects")?;
    let projects = parser.parse_projects()?;
    parser.expect_section("Technical Skills")?;
    let skills = parser.parse_skills()?;
    parser.expect(DOCUMENT_END)?;
    parser.finish("content after \\end{document}")?;

    Ok(Cv {
        profile,
        education: Cow::Owned(education),
        experience: Cow::Owned(experience),
        projects: Cow::Owned(projects),
        skills: Cow::Owned(skills),
    })
}

struct Parser<'a> {
    source: &'a str,
    position: usize,
    end: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, position: usize, end: usize) -> Self {
        Self {
            source,
            position,
            end,
        }
    }

    fn parse_profile(&mut self) -> Result<Profile<'static>, CvParseError> {
        self.expect("\\begin{center}")?;
        self.expect("\\textbf")?;
        let (name_start, name_end) = self.take_group()?;
        let mut name = Parser::new(self.source, name_start, name_end);
        name.expect("\\Huge")?;
        name.expect("\\scshape")?;
        name.skip_whitespace();
        let name_position = name.position;
        let full_name = normalize_whitespace(&self.source[name.position..name.end]);
        if full_name.is_empty()
            || full_name.contains('\\')
            || full_name.contains('{')
            || full_name.contains('}')
        {
            return Err(name.error_at(
                name_position,
                "profile name must be plain text after \\Huge \\scshape",
            ));
        }
        name.position = name.end;
        name.finish("unsupported profile-name content")?;

        self.expect("\\\\")?;
        self.expect("\\vspace")?;
        let spacing = self.plain_group("profile heading spacing")?;
        if spacing != "1pt" {
            return Err(self.error("profile heading must use \\vspace{1pt}"));
        }

        let (email_target, email_label) = self.parse_href()?;
        let email = email_target
            .strip_prefix("mailto:")
            .ok_or_else(|| self.error("the first heading link must be a mailto email address"))?;
        if email.is_empty() || email.contains(char::is_whitespace) {
            return Err(self.error("heading email address is malformed"));
        }
        if flatten_rich_text(&email_label) != email {
            return Err(self.error("heading email label must match its mailto address"));
        }
        self.expect("$|$")?;

        let first_social = self.parse_social_link()?;
        self.expect("$|$")?;
        let second_social = self.parse_social_link()?;
        if first_social.platform == second_social.platform {
            return Err(self.error("heading must contain one LinkedIn and one GitHub link"));
        }
        self.expect("\\end{center}")?;

        Ok(Profile {
            full_name: Cow::Owned(full_name),
            contact: ContactDetails {
                email: Cow::Owned(email.to_owned()),
            },
            social_links: Cow::Owned(vec![first_social, second_social]),
        })
    }

    fn parse_social_link(&mut self) -> Result<SocialLink<'static>, CvParseError> {
        let (url, label) = self.parse_href()?;
        if !url.starts_with("https://") {
            return Err(self.error("social links must use absolute HTTPS URLs"));
        }
        let platform = if url.contains("linkedin.com/") {
            SocialPlatform::LinkedIn
        } else if url.contains("github.com/") {
            SocialPlatform::GitHub
        } else {
            return Err(self.error("unsupported social-link platform"));
        };
        Ok(SocialLink {
            platform,
            url: Cow::Owned(url),
            label,
        })
    }

    fn parse_href(&mut self) -> Result<(String, RichText<'static>), CvParseError> {
        self.expect("\\href")?;
        let target = self.plain_group("link target")?;
        validate_link_target(&target).map_err(|message| self.error(message))?;
        let label = self.rich_group()?;
        if label.nodes.is_empty() {
            return Err(self.error("link labels must not be empty"));
        }
        Ok((target, label))
    }

    fn expect_section(&mut self, expected: &str) -> Result<(), CvParseError> {
        self.expect("\\section")?;
        let actual = self.plain_group("section name")?;
        if actual != expected {
            return Err(self.error(format!("expected section {expected:?}, found {actual:?}")));
        }
        Ok(())
    }

    fn parse_education(&mut self) -> Result<Vec<Education<'static>>, CvParseError> {
        self.expect("\\resumeSubHeadingListStart")?;
        let mut entries = Vec::new();
        while !self.next_is("\\resumeSubHeadingListEnd") {
            self.expect("\\resumeSubheading")?;
            let institution = self.required_rich_group("education institution")?;
            let location = parse_location(
                self.plain_group("education location")?,
                self.source,
                self.position,
            )?;
            let qualification = self.required_rich_group("education qualification")?;
            let dates = parse_date_range(
                &self.plain_group("education date range")?,
                self.source,
                self.position,
            )?;
            entries.push(Education {
                institution,
                qualification,
                location,
                dates,
            });
        }
        self.expect("\\resumeSubHeadingListEnd")?;
        require_entries(self, &entries, "education")?;
        Ok(entries)
    }

    fn parse_experience(&mut self) -> Result<Vec<Experience<'static>>, CvParseError> {
        self.expect("\\resumeSubHeadingListStart")?;
        let mut entries = Vec::new();
        while !self.next_is("\\resumeSubHeadingListEnd") {
            self.expect("\\resumeSubheading")?;
            let role = self.required_rich_group("experience role")?;
            let dates = parse_date_range(
                &self.plain_group("experience date range")?,
                self.source,
                self.position,
            )?;
            let organisation = self.required_rich_group("experience organisation")?;
            let location = parse_location(
                self.plain_group("experience location")?,
                self.source,
                self.position,
            )?;
            let highlights = self.parse_highlights()?;
            entries.push(Experience {
                role,
                organisation,
                location,
                dates,
                highlights: Cow::Owned(highlights),
            });
        }
        self.expect("\\resumeSubHeadingListEnd")?;
        require_entries(self, &entries, "experience")?;
        Ok(entries)
    }

    fn parse_projects(&mut self) -> Result<Vec<Project<'static>>, CvParseError> {
        self.expect("\\resumeSubHeadingListStart")?;
        let mut entries = Vec::new();
        while !self.next_is("\\resumeSubHeadingListEnd") {
            self.expect("\\resumeProjectHeading")?;
            let (heading_start, heading_end) = self.take_group()?;
            let (title, technologies) = self.parse_project_heading(heading_start, heading_end)?;
            let period = self.rich_group()?;
            let period = (!period.nodes.is_empty()).then_some(period);
            let highlights = self.parse_highlights()?;
            entries.push(Project {
                title,
                technologies: Cow::Owned(technologies),
                period,
                highlights: Cow::Owned(highlights),
            });
        }
        self.expect("\\resumeSubHeadingListEnd")?;
        require_entries(self, &entries, "projects")?;
        Ok(entries)
    }

    fn parse_project_heading(
        &self,
        start: usize,
        end: usize,
    ) -> Result<(RichText<'static>, Vec<Cow<'static, str>>), CvParseError> {
        let mut heading = Parser::new(self.source, start, end);
        heading.expect("\\textbf")?;
        let title_content = heading.required_rich_group("project title")?;
        let title = RichText {
            nodes: Cow::Owned(vec![Inline::Strong(title_content)]),
        };
        heading.expect("$|$")?;
        heading.expect("\\emph")?;
        let technology_list = heading.plain_group("project technologies")?;
        heading.finish("unsupported project-heading content")?;
        let technologies = split_comma_values(&technology_list, self.source, start, "technology")?;
        Ok((title, technologies))
    }

    fn parse_highlights(&mut self) -> Result<Vec<RichText<'static>>, CvParseError> {
        self.expect("\\resumeItemListStart")?;
        let mut highlights = Vec::new();
        while !self.next_is("\\resumeItemListEnd") {
            self.expect("\\resumeItem")?;
            highlights.push(self.required_rich_group("bullet point")?);
        }
        self.expect("\\resumeItemListEnd")?;
        require_entries(self, &highlights, "bullet points")?;
        Ok(highlights)
    }

    fn parse_skills(&mut self) -> Result<Vec<SkillGroup<'static>>, CvParseError> {
        self.expect("\\begin{itemize}")?;
        let options = self.take_delimited('[', ']')?;
        if normalize_whitespace(&self.source[options.0..options.1]) != "leftmargin=0.15in, label={}"
        {
            return Err(self.error("unsupported Technical Skills itemize options"));
        }
        self.expect("\\small")?;
        let (small_start, small_end) = self.take_group()?;
        let mut small = Parser::new(self.source, small_start, small_end);
        small.expect("\\item")?;
        let (list_start, list_end) = small.take_group()?;
        small.finish("unsupported content around the Technical Skills item")?;

        let mut list = Parser::new(self.source, list_start, list_end);
        let mut groups = Vec::new();
        while !list.is_finished() {
            list.expect("\\textbf")?;
            let category = list.plain_group("skill category")?;
            if category.is_empty() {
                return Err(list.error("skill category must not be empty"));
            }
            let values = list.plain_group("skill values")?;
            let values = values
                .strip_prefix(':')
                .ok_or_else(|| list.error("skill values must begin with a colon"))?;
            let skills = split_comma_values(values, self.source, list.position, "skill")?;
            groups.push(SkillGroup {
                category: Cow::Owned(category),
                skills: Cow::Owned(skills),
            });
            if !list.is_finished() {
                list.expect("\\\\")?;
            }
        }
        self.expect("\\end{itemize}")?;
        require_entries(self, &groups, "skill groups")?;
        Ok(groups)
    }

    fn required_rich_group(&mut self, context: &str) -> Result<RichText<'static>, CvParseError> {
        let value = self.rich_group()?;
        if value.nodes.is_empty() {
            Err(self.error(format!("{context} must not be empty")))
        } else {
            Ok(value)
        }
    }

    fn rich_group(&mut self) -> Result<RichText<'static>, CvParseError> {
        let (start, end) = self.take_group()?;
        parse_rich_text(self.source, start, end)
    }

    fn plain_group(&mut self, context: &str) -> Result<String, CvParseError> {
        let position = self.position;
        let rich = self.rich_group()?;
        plain_only(&rich).ok_or_else(|| {
            self.error_at(
                position,
                format!("{context} does not support inline formatting"),
            )
        })
    }

    fn take_group(&mut self) -> Result<(usize, usize), CvParseError> {
        self.take_delimited('{', '}')
    }

    fn take_delimited(
        &mut self,
        opening: char,
        closing: char,
    ) -> Result<(usize, usize), CvParseError> {
        self.skip_whitespace();
        let opening_position = self.position;
        if self.peek_char() != Some(opening) {
            return Err(self.error(format!("expected {opening:?}")));
        }
        self.position += opening.len_utf8();
        let start = self.position;
        let mut depth = 1usize;
        while self.position < self.end {
            let current = self.peek_char().expect("position is before end");
            if current == '\\' {
                let after_slash = self.position + 1;
                if after_slash < self.end {
                    let escaped = self.source[after_slash..self.end]
                        .chars()
                        .next()
                        .expect("a character follows the slash");
                    if escaped == opening || escaped == closing {
                        self.position = after_slash + escaped.len_utf8();
                        continue;
                    }
                }
            }
            self.position += current.len_utf8();
            if current == opening {
                depth += 1;
            } else if current == closing {
                depth -= 1;
                if depth == 0 {
                    return Ok((start, self.position - closing.len_utf8()));
                }
            }
        }
        Err(self.error_at(opening_position, format!("unclosed {opening:?} group")))
    }

    fn expect(&mut self, expected: &str) -> Result<(), CvParseError> {
        self.skip_whitespace();
        if self.source[self.position..self.end].starts_with(expected) {
            self.position += expected.len();
            Ok(())
        } else {
            Err(self.error(format!("expected {expected}")))
        }
    }

    fn next_is(&mut self, expected: &str) -> bool {
        self.skip_whitespace();
        self.source[self.position..self.end].starts_with(expected)
    }

    fn finish(&mut self, message: &str) -> Result<(), CvParseError> {
        self.skip_whitespace();
        if self.position == self.end {
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn is_finished(&mut self) -> bool {
        self.skip_whitespace();
        self.position == self.end
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.end {
            let character = self.peek_char().expect("position is before end");
            if !character.is_whitespace() {
                break;
            }
            self.position += character.len_utf8();
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.position..self.end].chars().next()
    }

    fn error(&self, message: impl Into<String>) -> CvParseError {
        self.error_at(self.position, message)
    }

    fn error_at(&self, position: usize, message: impl Into<String>) -> CvParseError {
        error_at(self.source, position, message)
    }
}

fn parse_rich_text(
    source: &str,
    start: usize,
    end: usize,
) -> Result<RichText<'static>, CvParseError> {
    let mut parser = Parser::new(source, start, end);
    let mut builder = InlineBuilder::default();
    while parser.position < parser.end {
        let character = parser.peek_char().expect("position is before end");
        if character.is_whitespace() {
            builder.pending_space = true;
            parser.position += character.len_utf8();
            continue;
        }
        if character != '\\' {
            if character == '{' || character == '}' || character == '$' {
                return Err(parser.error("unbraced groups and math mode are unsupported inline"));
            }
            builder.push_char(character);
            parser.position += character.len_utf8();
            continue;
        }

        let command_position = parser.position;
        parser.position += 1;
        let command_start = parser.position;
        while parser.position < parser.end
            && parser
                .peek_char()
                .is_some_and(|next| next.is_ascii_alphabetic())
        {
            parser.position += 1;
        }
        if command_start == parser.position {
            let escaped = parser
                .peek_char()
                .ok_or_else(|| parser.error_at(command_position, "trailing inline backslash"))?;
            parser.position += escaped.len_utf8();
            let literal = match escaped {
                '&' | '%' | '#' | '_' | '$' | '{' | '}' => escaped,
                _ => {
                    return Err(parser.error_at(
                        command_position,
                        format!("unsupported inline command \\{escaped}"),
                    ));
                }
            };
            builder.push_char(literal);
            continue;
        }

        let command = &source[command_start..parser.position];
        let node = match command {
            "textbf" => Inline::Strong(parser.rich_group()?),
            "emph" | "textit" => Inline::Emphasis(parser.rich_group()?),
            "underline" => Inline::Underline(parser.rich_group()?),
            "href" => {
                let target = parser.plain_group("link target")?;
                validate_link_target(&target)
                    .map_err(|message| parser.error_at(command_position, message))?;
                let label = parser.rich_group()?;
                if label.nodes.is_empty() {
                    return Err(parser.error_at(command_position, "link label must not be empty"));
                }
                Inline::Link {
                    target: Cow::Owned(target),
                    label,
                }
            }
            _ => {
                return Err(parser.error_at(
                    command_position,
                    format!("unsupported inline command \\{command}"),
                ));
            }
        };
        builder.push_node(node);
    }
    Ok(RichText {
        nodes: Cow::Owned(builder.finish()),
    })
}

#[derive(Default)]
struct InlineBuilder {
    nodes: Vec<Inline<'static>>,
    text: String,
    pending_space: bool,
}

impl InlineBuilder {
    fn push_char(&mut self, character: char) {
        self.push_pending_space();
        self.text.push(character);
    }

    fn push_node(&mut self, node: Inline<'static>) {
        self.push_pending_space();
        self.flush_text();
        self.nodes.push(node);
    }

    fn push_pending_space(&mut self) {
        if self.pending_space && (!self.text.is_empty() || !self.nodes.is_empty()) {
            self.text.push(' ');
        }
        self.pending_space = false;
    }

    fn flush_text(&mut self) {
        if !self.text.is_empty() {
            self.nodes
                .push(Inline::Text(Cow::Owned(std::mem::take(&mut self.text))));
        }
    }

    fn finish(mut self) -> Vec<Inline<'static>> {
        self.flush_text();
        self.nodes
    }
}

fn parse_location(
    value: String,
    source: &str,
    position: usize,
) -> Result<Location<'static>, CvParseError> {
    let (city, country) = value.rsplit_once(", ").ok_or_else(|| {
        error_at(
            source,
            position,
            "locations must use the `City, Country` form",
        )
    })?;
    if city.trim().is_empty() || country.trim().is_empty() {
        return Err(error_at(
            source,
            position,
            "location city and country must not be empty",
        ));
    }
    Ok(Location {
        city: Cow::Owned(city.trim().to_owned()),
        country: Cow::Owned(country.trim().to_owned()),
    })
}

fn parse_date_range(value: &str, source: &str, position: usize) -> Result<DateRange, CvParseError> {
    let (start, end) = value.split_once(" -- ").ok_or_else(|| {
        error_at(
            source,
            position,
            "date ranges must use `Month Year -- Month Year` or `-- Present`",
        )
    })?;
    let start = parse_date(start, source, position)?;
    let end = if end == "Present" {
        DateRangeEnd::Present
    } else {
        let end = parse_date(end, source, position)?;
        if date_key(end) < date_key(start) {
            return Err(error_at(
                source,
                position,
                "date range ends before it starts",
            ));
        }
        DateRangeEnd::Date(end)
    };
    Ok(DateRange { start, end })
}

fn parse_date(value: &str, source: &str, position: usize) -> Result<CvDate, CvParseError> {
    let (month, year) = value.rsplit_once(' ').ok_or_else(|| {
        error_at(
            source,
            position,
            format!("malformed month/year date {value:?}"),
        )
    })?;
    let month = match month.trim_end_matches('.') {
        "Jan" | "January" => Month::January,
        "Feb" | "February" => Month::February,
        "Mar" | "March" => Month::March,
        "Apr" | "April" => Month::April,
        "May" => Month::May,
        "Jun" | "June" => Month::June,
        "Jul" | "July" => Month::July,
        "Aug" | "August" => Month::August,
        "Sep" | "Sept" | "September" => Month::September,
        "Oct" | "October" => Month::October,
        "Nov" | "November" => Month::November,
        "Dec" | "December" => Month::December,
        _ => {
            return Err(error_at(
                source,
                position,
                format!("unsupported month in date {value:?}"),
            ));
        }
    };
    if year.len() != 4 {
        return Err(error_at(
            source,
            position,
            format!("date year must be four digits in {value:?}"),
        ));
    }
    let year = year.parse::<u16>().map_err(|_| {
        error_at(
            source,
            position,
            format!("date year must be four digits in {value:?}"),
        )
    })?;
    if !(1900..=2200).contains(&year) {
        return Err(error_at(
            source,
            position,
            format!("date year must be four digits in {value:?}"),
        ));
    }
    Ok(CvDate { year, month })
}

fn date_key(date: CvDate) -> (u16, u8) {
    let month = match date.month {
        Month::January => 1,
        Month::February => 2,
        Month::March => 3,
        Month::April => 4,
        Month::May => 5,
        Month::June => 6,
        Month::July => 7,
        Month::August => 8,
        Month::September => 9,
        Month::October => 10,
        Month::November => 11,
        Month::December => 12,
    };
    (date.year, month)
}

fn split_comma_values(
    value: &str,
    source: &str,
    position: usize,
    label: &str,
) -> Result<Vec<Cow<'static, str>>, CvParseError> {
    let values: Vec<_> = value
        .split(',')
        .map(str::trim)
        .map(ToOwned::to_owned)
        .collect();
    if values.is_empty() || values.iter().any(String::is_empty) {
        return Err(error_at(
            source,
            position,
            format!("{label} lists must contain non-empty comma-separated values"),
        ));
    }
    Ok(values.into_iter().map(Cow::Owned).collect())
}

fn require_entries<T>(parser: &Parser<'_>, entries: &[T], label: &str) -> Result<(), CvParseError> {
    if entries.is_empty() {
        Err(parser.error(format!("{label} must contain at least one entry")))
    } else {
        Ok(())
    }
}

fn plain_only(value: &RichText<'_>) -> Option<String> {
    let mut output = String::new();
    for node in value.nodes.iter() {
        match node {
            Inline::Text(text) => output.push_str(text),
            _ => return None,
        }
    }
    Some(output)
}

fn flatten_rich_text(value: &RichText<'_>) -> String {
    let mut output = String::new();
    for node in value.nodes.iter() {
        match node {
            Inline::Text(text) => output.push_str(text),
            Inline::Strong(content) | Inline::Emphasis(content) | Inline::Underline(content) => {
                output.push_str(&flatten_rich_text(content))
            }
            Inline::Link { label, .. } => output.push_str(&flatten_rich_text(label)),
        }
    }
    output
}

fn validate_link_target(target: &str) -> Result<(), &'static str> {
    if target.contains(char::is_whitespace) {
        return Err("link targets must not contain whitespace");
    }
    if target.starts_with("https://") || target.starts_with("mailto:") {
        Ok(())
    } else {
        Err("links must use an absolute HTTPS or mailto target")
    }
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_comments(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = bytes.to_vec();
    let mut position = 0usize;
    while position < bytes.len() {
        if bytes[position] == b'%' {
            let mut slash_count = 0usize;
            let mut before = position;
            while before > 0 && bytes[before - 1] == b'\\' {
                slash_count += 1;
                before -= 1;
            }
            if slash_count.is_multiple_of(2) {
                while position < bytes.len() && bytes[position] != b'\n' {
                    output[position] = b' ';
                    position += 1;
                }
                continue;
            }
        }
        position += 1;
    }
    String::from_utf8(output).expect("replacing ASCII comment bytes preserves UTF-8")
}

fn error_at(source: &str, position: usize, message: impl Into<String>) -> CvParseError {
    let safe_position = position.min(source.len());
    let prefix = &source[..safe_position];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.chars().count() + 1, |(_, line)| {
            line.chars().count() + 1
        });
    CvParseError {
        line,
        column,
        message: message.into(),
    }
}
