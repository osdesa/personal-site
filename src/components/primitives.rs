use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Container(children: Children) -> impl IntoView {
    view! { <div class="container">{children()}</div> }
}

#[component]
pub fn ButtonLink(
    href: &'static str,
    children: Children,
    #[prop(optional)] secondary: bool,
) -> impl IntoView {
    let class = if secondary {
        "button button--secondary"
    } else {
        "button button--primary"
    };

    view! {
        <A href=href attr:class=class>
            {children()}
            <span aria-hidden="true" class="button__arrow">"↗"</span>
        </A>
    }
}

#[component]
pub fn SectionHeading(eyebrow: &'static str, title: &'static str) -> impl IntoView {
    view! {
        <header class="section-heading">
            <p class="eyebrow">{eyebrow}</p>
            <h2>{title}</h2>
        </header>
    }
}

#[component]
pub fn SkillBadge(skill: &'static str) -> impl IntoView {
    view! { <span class="skill-badge">{skill}</span> }
}
