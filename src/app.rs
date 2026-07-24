use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::{
    components::{Redirect, Route, Router, Routes},
    path,
};

use crate::components::{SiteShell, StructuredData};
use crate::pages::{
    CvPage, HomePage, LegalNoticePage, NotFoundPage, PrivacyNoticePage, ProjectsPage,
};
use crate::routes::LEGAL_NOTICE;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <StructuredData />
        <Router>
            <SiteShell>
                <Routes fallback=NotFoundPage>
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("projects") view=ProjectsPage />
                    <Route path=path!("cv") view=CvPage />
                    <Route path=path!("legal") view=LegalNoticePage />
                    <Route path=path!("privacy") view=PrivacyNoticePage />
                    <Route
                        path=path!("legal-notice")
                        view=|| view! { <Redirect path=LEGAL_NOTICE.path /> }
                    />
                </Routes>
            </SiteShell>
        </Router>
    }
}
