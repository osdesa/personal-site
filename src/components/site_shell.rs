use leptos::prelude::*;
use leptos_router::components::A;

use crate::content::portfolio;
use crate::cv_presentation::RichTextView;
use crate::generated_cv::CV as GENERATED_CV;
use crate::routes::{HOME, NAVIGATION_ROUTES};

#[component]
pub fn SiteShell(children: Children) -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);
    let site_profile = portfolio().profile;
    let imported_profile = &GENERATED_CV.profile;
    let header_initials = initials(&imported_profile.full_name);
    let footer_initials = header_initials.clone();

    view! {
        <a class="skip-link" href="#main-content">"Skip to main content"</a>
        <header class="site-header">
            <div class="container site-header__inner">
                <A href=HOME.path attr:class="brand" attr:aria-label="Go to homepage">
                    <span class="brand__mark" aria-hidden="true">{header_initials}</span>
                    <span class="brand__name">{imported_profile.full_name.as_ref()}</span>
                    <span class="brand__role">{site_profile.role}</span>
                </A>

                <nav class="desktop-nav" aria-label="Primary navigation">
                    <ul>
                        {NAVIGATION_ROUTES.iter().map(|route| view! {
                            <li><A href=route.path attr:class="nav-link">{route.label}</A></li>
                        }).collect_view()}
                    </ul>
                </nav>

                <div class="site-header__actions">
                    <button
                        type="button"
                        class="menu-toggle"
                        aria-label=move || if menu_open.get() { "Close navigation menu" } else { "Open navigation menu" }
                        aria-expanded=move || menu_open.get().to_string()
                        aria-controls="mobile-navigation"
                        on:click=move |_| set_menu_open.update(|open| *open = !*open)
                    >
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </button>
                </div>
            </div>

            <nav
                id="mobile-navigation"
                class:mobile-nav=true
                class:mobile-nav--open=move || menu_open.get()
                aria-label="Mobile navigation"
                aria-hidden=move || (!menu_open.get()).to_string()
            >
                <ul class="container">
                    {NAVIGATION_ROUTES.iter().enumerate().map(|(index, route)| view! {
                        <li>
                            <A
                                href=route.path
                                attr:class="mobile-nav__link"
                                on:click=move |_| set_menu_open.set(false)
                            >
                                <span aria-hidden="true">{format!("0{}", index + 1)}</span>
                                {route.label}
                            </A>
                        </li>
                    }).collect_view()}
                </ul>
            </nav>
        </header>

        <main id="main-content" tabindex="-1">{children()}</main>

        <footer class="site-footer">
            <div class="container site-footer__grid">
                <div>
                    <A href=HOME.path attr:class="brand brand--footer">
                        <span class="brand__mark" aria-hidden="true">{footer_initials}</span>
                        <span class="brand__name">{imported_profile.full_name.as_ref()}</span>
                    </A>
                    <p>"Software engineering, selected projects and professional experience."</p>
                </div>
                <nav aria-label="Footer navigation">
                    <p class="footer-heading">"Navigate"</p>
                    <ul>
                        {NAVIGATION_ROUTES.iter().skip(1).map(|route| view! {
                            <li><A href=route.path>{route.label}</A></li>
                        }).collect_view()}
                    </ul>
                </nav>
                <div>
                    <p class="footer-heading">"Connect"</p>
                    <ul>
                        {imported_profile.social_links.iter().map(|link| view! {
                            <li>
                                <a href=link.url.as_ref() target="_blank" rel="noreferrer">
                                    <RichTextView text=&link.label />
                                    <span class="sr-only">" (opens in a new tab)"</span>
                                </a>
                            </li>
                        }).collect_view()}
                        <li>
                            <a href=format!("mailto:{}", imported_profile.contact.email)>"Email"</a>
                        </li>
                    </ul>
                </div>
            </div>
            <div class="container site-footer__base">
                <p>{format!("© 2026 {}.", imported_profile.full_name)}</p>
                <p class="site-footer__status"><span aria-hidden="true"></span>"Built for the long term"</p>
            </div>
        </footer>
    }
}

fn initials(full_name: &str) -> String {
    full_name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect()
}
