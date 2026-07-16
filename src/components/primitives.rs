use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Container(#[prop(optional)] class: &'static str, children: Children) -> impl IntoView {
    let class_name = if class.is_empty() { "container" } else { class };

    view! { <div class=class_name>{children()}</div> }
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
pub fn SectionHeading(
    eyebrow: &'static str,
    title: &'static str,
    #[prop(optional)] introduction: &'static str,
    #[prop(optional)] centred: bool,
) -> impl IntoView {
    let class = if centred {
        "section-heading section-heading--centred"
    } else {
        "section-heading"
    };

    view! {
        <header class=class>
            <p class="eyebrow">{eyebrow}</p>
            <h2>{title}</h2>
            {(!introduction.is_empty()).then(|| view! { <p class="section-heading__intro">{introduction}</p> })}
        </header>
    }
}

#[component]
pub fn SkillBadge(skill: &'static str) -> impl IntoView {
    view! { <span class="skill-badge">{skill}</span> }
}
