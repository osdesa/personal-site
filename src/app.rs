use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::{SiteShell, StructuredData};
use crate::pages::{CvPage, HomePage, NotFoundPage, ProjectsPage};

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
                </Routes>
            </SiteShell>
        </Router>
    }
}
