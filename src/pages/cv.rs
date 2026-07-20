use leptos::prelude::*;

use crate::components::{RouteMetadata, remove_static_description_on_mount};
use crate::cv_presentation::{CV_PDF_URL, CvDocument};
use crate::generated_cv::{CV as GENERATED_CV, SOURCE_COMMIT_SHA, SOURCE_TAG};
use crate::routes::{CV, metadata_for_path};

#[component]
pub fn CvPage() -> impl IntoView {
    let metadata = metadata_for_path(CV.path);
    remove_static_description_on_mount();

    view! {
        <RouteMetadata route=metadata />

        <CvDocument
            cv=Some(&GENERATED_CV)
            source_tag=SOURCE_TAG
            source_commit_sha=SOURCE_COMMIT_SHA
            pdf_url=Some(CV_PDF_URL)
        />
    }
}
