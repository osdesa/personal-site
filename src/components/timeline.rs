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
                <p class="timeline-entry__organisation">{item.organisation}</p>
                <p class="timeline-entry__summary">{item.summary}</p>
                <div class="skill-list" aria-label="Related topics">
                    {item.tags.iter().map(|tag| view! { <SkillBadge skill=*tag /> }).collect_view()}
                </div>
            </div>
        </article>
    }
}
