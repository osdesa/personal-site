use leptos::prelude::*;

use crate::components::RouteMetadata;
use crate::cv_presentation::{CV_PDF_URL, CvDocument};
use crate::generated_cv::{CV as GENERATED_CV, SOURCE_COMMIT_SHA, SOURCE_TAG};
use crate::routes::CV;

#[component]
pub fn CvPage() -> impl IntoView {
    view! {
        <RouteMetadata route=CV />

        <CvDocument
            cv=Some(&GENERATED_CV)
            source_tag=SOURCE_TAG
            source_commit_sha=SOURCE_COMMIT_SHA
            pdf_url=Some(CV_PDF_URL)
        />
    }
}
