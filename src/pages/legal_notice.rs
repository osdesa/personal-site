use leptos::prelude::*;

use crate::components::{Container, RouteMetadata, remove_static_description_on_mount};
use crate::generated_cv::CV as GENERATED_CV;
use crate::routes::{LEGAL_NOTICE, metadata_for_path};

/// Public legal and privacy information for the portfolio website.
#[component]
pub fn LegalNoticePage() -> impl IntoView {
    let metadata = metadata_for_path(LEGAL_NOTICE.path);
    let email = GENERATED_CV.profile.contact.email.as_ref();
    remove_static_description_on_mount();

    view! {
        <RouteMetadata route=metadata />

        <section class="page-hero page-hero--compact" aria-labelledby="legal-notice-title">
            <Container>
                <p class="eyebrow">"Website information"</p>
                <h1 id="legal-notice-title">"Legal notice"</h1>
                <p class="page-hero__lead">"Terms for using this personal portfolio and information about how limited personal data is handled."</p>
                <p>"Last reviewed: 24 July 2026."</p>
            </Container>
        </section>

        <section class="section legal-notice" aria-label="Legal notice details">
            <Container>
                <div class="legal-notice__content">
                    <section class="legal-notice__section" aria-labelledby="operator-title">
                        <h2 id="operator-title">"Site operator"</h2>
                        <div>
                            <p>"This website is a personal software engineering portfolio operated by Hayden Farrell."</p>
                            <p>"Hayden Farrell is the data controller for the personal data described in this notice."</p>
                            <p>
                                "For questions about this notice or the website, email "
                                <a href=format!("mailto:{email}")>{email}</a>"."
                            </p>
                        </div>
                    </section>

                    <section class="legal-notice__section" aria-labelledby="use-title">
                        <h2 id="use-title">"Use of this website"</h2>
                        <div>
                            <p>"The content is provided for general professional-information purposes. Reasonable care is taken to keep it accurate, but no warranty is given that it is complete, current or suitable for a particular purpose."</p>
                            <p>"Nothing on this website creates a professional-client relationship, an offer of services, or legal, financial or other professional advice. You use the website and any reliance on its content at your own discretion."</p>
                        </div>
                    </section>

                    <section class="legal-notice__section" aria-labelledby="intellectual-property-title">
                        <h2 id="intellectual-property-title">"Intellectual property"</h2>
                        <div>
                            <p>"Unless stated otherwise, the website design, written content and original materials are owned by Hayden Farrell. You may link to this website and share short excerpts with appropriate attribution, but may not reproduce substantial content or use it commercially without prior permission."</p>
                        </div>
                    </section>

                    <section class="legal-notice__section" aria-labelledby="links-title">
                        <h2 id="links-title">"External links"</h2>
                        <div>
                            <p>"Links to third-party websites are provided for convenience. Hayden Farrell does not control those sites and is not responsible for their content, availability or privacy practices. Review the relevant third party's terms and privacy information before using its services."</p>
                        </div>
                    </section>

                    <section class="legal-notice__section" aria-labelledby="privacy-title">
                        <h2 id="privacy-title">"Privacy and data handling"</h2>
                        <div>
                            <p>"This website has no user accounts, contact form, advertising or analytics scripts. It does not intentionally use non-essential cookies or similar tracking technologies."</p>
                            <p>"If you email the published address, the information you provide is used only to read, respond to and keep necessary records of the correspondence. It is retained only for as long as reasonably necessary for that purpose or to meet a legal obligation."</p>
                            <p>"Where UK data-protection law applies, the lawful basis for this limited processing is the operator's legitimate interests in responding to correspondence and maintaining a secure, functioning website."</p>
                            <p>"The site is hosted on Cloudflare Pages. As part of delivering and securing the site, Cloudflare may process limited technical request information, such as IP address and device or traffic data, on the operator's behalf."</p>
                        </div>
                    </section>

                    <section class="legal-notice__section" aria-labelledby="rights-title">
                        <h2 id="rights-title">"Your rights and contact"</h2>
                        <div>
                            <p>"Where UK data-protection law applies, you may have rights to request access to, correction or deletion of your personal data, and to object to or restrict certain processing. To make a request, email "<a href=format!("mailto:{email}")>{email}</a>"."</p>
                            <p>"You may also raise a concern with the UK Information Commissioner's Office. This notice is governed by the laws of England and Wales."</p>
                        </div>
                    </section>
                </div>
            </Container>
        </section>
    }
}
