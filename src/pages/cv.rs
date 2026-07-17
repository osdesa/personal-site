use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::cv_presentation::{CV_PDF_URL, CvDocument};
use crate::generated_cv::{CV as GENERATED_CV, SOURCE_COMMIT_SHA, SOURCE_TAG};
use crate::routes::{CV, title_for_path};

#[component]
pub fn CvPage() -> impl IntoView {
    view! {
        <Title text=title_for_path(CV.path) />
        <Meta
            name="description"
            content="Hayden Farrell's generated curriculum vitae: experience, education, projects and technical skills."
        />

        <CvDocument
            cv=Some(&GENERATED_CV)
            source_tag=SOURCE_TAG
            source_commit_sha=SOURCE_COMMIT_SHA
            pdf_url=Some(CV_PDF_URL)
        />
    }
}
