use leptos::prelude::*;

use crate::components::SkillBadge;
use crate::content::TimelineItem;

#[component]
pub fn TimelineEntry(item: TimelineItem) -> impl IntoView {
    view! {
        <article class="timeline-entry">
            <div class="timeline-entry__marker" aria-hidden="true"></div>
            <div class="timeline-entry__content">
                <p class="timeline-entry__period">{item.period}</p>
                <h3>{item.title}</h3>
                <p class="timeline-entry__organisation">
                    {item.organisation}
                    <span aria-hidden="true">" · "</span>
                    {item.location}
                </p>
                <p class="timeline-entry__summary">{item.summary}</p>
                {(!item.highlights.is_empty()).then(|| view! {
                    <ul class="timeline-entry__highlights">
                        {item.highlights.iter().map(|highlight| view! { <li>{*highlight}</li> }).collect_view()}
                    </ul>
                })}
                <div class="skill-list" aria-label="Related topics">
                    {item.tags.iter().map(|tag| view! { <SkillBadge skill=*tag /> }).collect_view()}
                </div>
            </div>
        </article>
    }
}
