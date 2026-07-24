use leptos::prelude::*;

use crate::components::{Container, RouteMetadata};
use crate::generated_cv::CV as GENERATED_CV;
use crate::routes::PRIVACY_NOTICE;

/// Public privacy and data-protection information for the portfolio website.
#[component]
pub fn PrivacyNoticePage() -> impl IntoView {
    let email = GENERATED_CV.profile.contact.email.as_ref();

    view! {
        <RouteMetadata route=PRIVACY_NOTICE />

        <section class="page-hero page-hero--compact" aria-labelledby="privacy-notice-title">
            <Container>
                <p class="eyebrow">"Data protection"</p>
                <h1 id="privacy-notice-title">"Privacy notice"</h1>
                <p class="page-hero__lead">"How limited personal and technical information is handled when you visit this website or make contact by email."</p>
                <p>"Last updated: 24 July 2026."</p>
            </Container>
        </section>

        <section class="section notice" aria-label="Privacy notice details">
            <Container>
                <div class="notice__content">
                    <section class="notice__section" aria-labelledby="controller-title">
                        <h2 id="controller-title">"Data controller"</h2>
                        <div>
                            <p>"Hayden Farrell is the data controller for the personal data described in this notice."</p>
                        </div>
                    </section>

                    <section class="notice__section" aria-labelledby="privacy-contact-title">
                        <h2 id="privacy-contact-title">"Contact details"</h2>
                        <div>
                            <p>
                                "For privacy questions or requests, email "
                                <a href=format!("mailto:{email}")>{email}</a>"."
                            </p>
                        </div>
                    </section>

                    <section class="notice__section" aria-labelledby="correspondence-title">
                        <h2 id="correspondence-title">"Email correspondence"</h2>
                        <div>
                            <p>"If you email the published address, the information you provide is used to read and respond to your message and to keep necessary records of the correspondence."</p>
                            <p>"Correspondence is retained only for as long as reasonably necessary for those purposes or to meet a legal obligation."</p>
                        </div>
                    </section>

                    <section class="notice__section" aria-labelledby="cloudflare-title">
                        <h2 id="cloudflare-title">"Cloudflare hosting"</h2>
                        <div>
                            <p>"This website is hosted using Cloudflare Pages. In order to deliver and secure the website, Cloudflare processes technical information such as IP addresses, browser information and request metadata in accordance with its own infrastructure and privacy practices."</p>
                            <p>"This website does not intentionally use advertising or behavioural tracking technologies."</p>
                        </div>
                    </section>

                    <section class="notice__section" aria-labelledby="uk-gdpr-title">
                        <h2 id="uk-gdpr-title">"UK GDPR information"</h2>
                        <div>
                            <p>"Where UK data-protection law applies, limited personal data is processed only where necessary for legitimate interests in responding to correspondence and maintaining a secure, functioning website, or where required by law."</p>
                            <p>"Those interests are considered against the rights and freedoms of the people whose information is processed."</p>
                        </div>
                    </section>

                    <section class="notice__section" aria-labelledby="rights-title">
                        <h2 id="rights-title">"Your rights"</h2>
                        <div>
                            <p>"Where UK data-protection law applies, you may have rights to request access to, correction or deletion of your personal data, and to object to or restrict certain processing. To make a request, email "<a href=format!("mailto:{email}")>{email}</a>"."</p>
                            <p>"You may also raise a concern with the UK Information Commissioner's Office."</p>
                        </div>
                    </section>
                </div>
            </Container>
        </section>
    }
}
