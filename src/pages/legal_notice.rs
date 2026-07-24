use leptos::prelude::*;

use crate::components::{Container, RouteMetadata};
use crate::generated_cv::CV as GENERATED_CV;
use crate::routes::LEGAL_NOTICE;

/// Public terms and ownership information for the portfolio website.
#[component]
pub fn LegalNoticePage() -> impl IntoView {
    let email = GENERATED_CV.profile.contact.email.as_ref();

    view! {
        <RouteMetadata route=LEGAL_NOTICE />

        <section class="page-hero page-hero--compact" aria-labelledby="legal-notice-title">
            <Container>
                <p class="eyebrow">"Website information"</p>
                <h1 id="legal-notice-title">"Legal notice"</h1>
                <p class="page-hero__lead">"Information about this website, its content and the terms that apply when using it."</p>
                <p>"Last updated: 24 July 2026."</p>
            </Container>
        </section>

        <section class="section notice" aria-label="Legal notice details">
            <Container>
                <div class="notice__content">
                    <section class="notice__section" aria-labelledby="website-information-title">
                        <h2 id="website-information-title">"Website information"</h2>
                        <div>
                            <p>"This website is a personal software engineering portfolio operated by Hayden Farrell."</p>
                            <p>
                                "For questions about the website or this notice, email "
                                <a href=format!("mailto:{email}")>{email}</a>"."
                            </p>
                        </div>
                    </section>

                    <section class="notice__section" aria-labelledby="terms-title">
                        <h2 id="terms-title">"Terms of use"</h2>
                        <div>
                            <p>"The content is provided for general professional-information purposes. Reasonable care is taken to keep it accurate, but no warranty is given that it is complete, current or suitable for a particular purpose."</p>
                            <p>"You may browse and link to the website for lawful purposes. You use the website and any reliance on its content at your own discretion."</p>
                        </div>
                    </section>

                    <section class="notice__section" aria-labelledby="intellectual-property-title">
                        <h2 id="intellectual-property-title">"Intellectual property"</h2>
                        <div>
                            <p>"Unless otherwise stated, all original content, source code (where not licensed separately), website design, text and graphics are © Hayden Farrell."</p>
                            <p>"Linked repositories may be available under their own licence terms. You may share short excerpts from this website with appropriate attribution, but may not reproduce substantial content or use it commercially without prior permission."</p>
                        </div>
                    </section>

                    <section class="notice__section" aria-labelledby="links-title">
                        <h2 id="links-title">"External links"</h2>
                        <div>
                            <p>"Links to third-party websites are provided for convenience. Hayden Farrell does not control those sites and is not responsible for their content, availability or privacy practices. Review the relevant third party's terms and privacy information before using its services."</p>
                            <p>"Links to GitHub repositories are also subject to GitHub's own terms and privacy policies."</p>
                        </div>
                    </section>

                    <section class="notice__section" aria-labelledby="disclaimer-title">
                        <h2 id="disclaimer-title">"Disclaimer and liability"</h2>
                        <div>
                            <p>"Nothing on this website constitutes professional advice or creates any contractual or professional relationship."</p>
                            <p>"To the fullest extent permitted by law, no liability is accepted for loss or damage arising from use of the website, reliance on its content, or the availability of linked third-party services."</p>
                        </div>
                    </section>
                </div>
            </Container>
        </section>
    }
}
