use leptos::prelude::*;
use leptos_router::components::A;

use crate::content::portfolio;
use crate::routes::{HOME, NAVIGATION_ROUTES};

#[component]
pub fn SiteShell(children: Children) -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);
    let profile = portfolio().profile;

    view! {
        <a class="skip-link" href="#main-content">"Skip to main content"</a>
        <header class="site-header">
            <div class="container site-header__inner">
                <A href=HOME.path attr:class="brand" attr:aria-label="Go to homepage">
                    <span class="brand__mark" aria-hidden="true">{profile.initials}</span>
                    <span class="brand__name">{profile.name}</span>
                    <span class="brand__role">{profile.role}</span>
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
                        <span class="brand__mark" aria-hidden="true">{profile.initials}</span>
                        <span class="brand__name">{profile.name}</span>
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
                        {portfolio().social_links.iter().map(|link| view! {
                            <li><a href=link.url>{link.label}</a></li>
                        }).collect_view()}
                    </ul>
                </div>
            </div>
            <div class="container site-footer__base">
                <p>{format!("© 2026 {}. Placeholder content is clearly labelled.", profile.name)}</p>
                <p class="site-footer__status"><span aria-hidden="true"></span>"Built for the long term"</p>
            </div>
        </footer>
    }
}
