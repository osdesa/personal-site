use leptos::prelude::*;
use leptos_router::components::A;

use crate::content::portfolio;
use crate::cv::SocialPlatform;
use crate::generated_cv::CV as GENERATED_CV;
use crate::routes::{HOME, LEGAL_NOTICE, NAVIGATION_ROUTES};

#[component]
pub fn SiteShell(children: Children) -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);
    let site_profile = portfolio().profile;
    let imported_profile = &GENERATED_CV.profile;

    view! {
        <a
            class="skip-link"
            href="#main-content"
            on:click=move |event| {
                event.prevent_default();
                focus_main_content();
            }
        >
            "Skip to main content"
        </a>
        <header class="site-header">
            <div class="container site-header__inner">
                <A href=HOME.path attr:class="brand" attr:aria-label="Go to homepage">
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
                inert=move || !menu_open.get()
            >
                <ul class="container">
                    {NAVIGATION_ROUTES.iter().map(|route| view! {
                        <li>
                            <A
                                href=route.path
                                attr:class="mobile-nav__link"
                                on:click=move |_| {
                                    set_menu_open.set(false);
                                    focus_main_content();
                                }
                            >
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
                    <A href=HOME.path attr:class="brand brand--footer" attr:aria-label="Go to homepage">
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
                        <li>
                            <a href=format!("mailto:{}", imported_profile.contact.email)>"Email"</a>
                        </li>
                        {imported_profile.social_links.iter().filter(|link| link.platform == SocialPlatform::LinkedIn).map(|link| view! {
                            <li>
                                <a href=link.url.as_ref() target="_blank" rel="noreferrer">
                                    "LinkedIn"
                                    <span class="sr-only">" (opens in a new tab)"</span>
                                </a>
                            </li>
                        }).collect_view()}
                        {imported_profile.social_links.iter().filter(|link| link.platform == SocialPlatform::GitHub).map(|link| view! {
                            <li>
                                <a href=link.url.as_ref() target="_blank" rel="noreferrer">
                                    "GitHub"
                                    <span class="sr-only">" (opens in a new tab)"</span>
                                </a>
                            </li>
                        }).collect_view()}
                    </ul>
                </div>
            </div>
            <div class="container site-footer__base">
                <p>{format!("© 2026 {}.", imported_profile.full_name)}</p>
                <A href=LEGAL_NOTICE.path>"Legal notice"</A>
            </div>
        </footer>
    }
}

#[cfg(target_arch = "wasm32")]
fn focus_main_content() {
    use leptos::wasm_bindgen::JsCast;

    let Some(main_content) = leptos::web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("main-content"))
    else {
        return;
    };

    let Ok(main_content) = main_content.dyn_into::<leptos::web_sys::HtmlElement>() else {
        return;
    };

    let _ = main_content.focus();
}

#[cfg(not(target_arch = "wasm32"))]
fn focus_main_content() {}
