use perseus::prelude::*;
use sycamore::prelude::*;

/// Where people reach Spark Hill. The site has no contact page or form (it
/// is static files with no server), so "Contact Us" goes to the Nextdoor
/// page, which takes messages.
pub(crate) const CONTACT_URL: &str =
    "https://nextdoor.com/pages/private-reading-tutor-oceanside-ca/";

fn index_page<G: Html>(cx: Scope) -> View<G> {
    page(cx, t!(cx, "tagline"), link!(cx, ""))
}

/// A link to one of the page's sections, from `home`, the page's own URL
/// with its locale (`en-US/`). A bare `/#id` would send the Perseus router
/// to `/`, which redirects to the locale page at the top and loses the
/// section. `rel="external"` makes the router leave the click to the
/// browser, so it scrolls in place on `/en-US/`, or loads it from `/en-US`
/// (where Perseus's own redirect leaves the address).
fn section_link<G: Html>(cx: Scope, home: &str, id: &str, label: &'static str) -> View<G> {
    let href = format!("{home}#{id}");
    view! { cx,
        a(href=href, rel="external", class="text-sky-50 hover:text-teal-500 pr-4 pl-4 hidden md:flex") { (label) }
    }
}

/// The whole home page: nav, the four sections and the footer. It takes the
/// translated tagline and the locale's home URL so tests can render it
/// without a translator.
fn page<G: Html>(cx: Scope, tagline: String, home: String) -> View<G> {
    let what_we_do = section_link(cx, &home, "what-we-do", "What we do");
    let who_we_are = section_link(cx, &home, "who-we-are", "Who we are");
    let how_we_do_it = section_link(cx, &home, "how-we-do-it", "How we do it");
    view! { cx,
        header {
            nav(class="bg-sky-900") {
                div(class="mx-auto pr-4 pl-4 2xl:p-0 2xl:max-w-screen-2xl") {
                    div(class="flex items-center justify-start flex-wrap pt-3 pb-3 md:pt-4 md:pb-4 md:justify-end") {
                        a(href="/", class="text-sky-50 text-2xl font-semibold pb-2") { "Spark Hill" }
                        div(class="flex-grow")
                        (what_we_do)
                        (who_we_are)
                        (how_we_do_it)
                        a(href=CONTACT_URL, target="_blank", rel="noopener noreferrer", class="p-2 pt-1 border-2 rounded-full text-sky-50 border-sky-50 hover:border-transparent hover:bg-teal-500") { "Contact Us" }
                    }
                }
            }
        }
        main(class="mr-4 ml-4 text-center md:text-left") {
            div(class="w-full bg-white pt-10") {
                div(class="container max-w-screen-2xl flex flex-wrap justify-around items-center") {
                    h1(class="text-7xl text-sky-700 text-center font-medium lg:w-1/2") { (tagline) }
                    img(src=".perseus/static/images/reading.382e285b.svg", alt="Person reading", class="lg:w-1/2")
                }
            }
            div(class="w-full bg-white pt-10") {
                div(class="container max-w-screen-2xl prose prose-stone") {
                    h1(id="what-we-do", class="text-sky-700") { "What we do" }
                    p {
                        "We provide a range of services to help individuals improve their reading skills. \
                        These services may include one-on-one tutoring or small group instruction, using \
                        specialized teaching methods and techniques to help individuals with dyslexia or \
                        other reading difficulties. We also offer assessments and evaluations to identify \
                        an individual’s specific needs and challenges, and then develop a personalized plan \
                        to address those needs. Additionally, we provide training and support for parents, \
                        teachers, and other professionals who work with individuals with reading \
                        difficulties, to help them understand how to effectively support and assist these \
                        individuals."
                    }
                }
            }
            div(class="w-full bg-white pt-10") {
                div(class="container max-w-screen-2xl flex flex-wrap justify-around items-center prose prose-stone") {
                    div(class="p-4 xl:w-1/4") {
                        img(src=".perseus/static/images/Classroom.e9004107.jpg", alt="Classroom")
                    }
                    div(class="xl:w-3/4") {
                        h1(id="who-we-are", class="text-sky-700") { "Who we are" }
                        p {
                            "Meagan has extensive experience working with struggling readers and readers \
                            who just need a little boost."
                        }
                        p {
                            "Her specialty is in reading remediation programs, meaning there is a strong \
                            focus on re-training the brain. This is done by filling in the gaps a reader \
                            has and bringing them up to their appropriate level at the pace they need."
                        }
                        p {
                            "Meagan worked as a Center Director at Lindamood-Bell Learning Processes in \
                            Chicago and worked at The Windward School for children with Dyslexia in New \
                            York. She developed a reading remediation program for struggling readers at \
                            the Rosalyn Yalow Charter School in the Bronx, New York and trained teachers to \
                            implement the programs."
                        }
                        p {
                            "Meagan has conducted online and in-person tutoring since 2010 and has \
                            initiated the development of a supplementary reading app."
                        }
                        p {
                            "She is a triathlete who just completed her first Ironman triathlon and \
                            recently relocated to Oceanside, CA with her husband."
                        }
                    }
                }
            }
            div(class="w-full bg-white pt-10") {
                div(class="container max-w-screen-2xl prose prose-stone") {
                    h1(id="how-we-do-it", class="text-sky-700") { "How we do it" }
                    p {
                        "There are several tests that are commonly used to diagnose dyslexia, including \
                        the Dyslexia Early Screening Test (DEST), the Phonological Awareness Test (PAT), \
                        and the Comprehensive Test of Phonological Processing (CTOPP). These tests are \
                        typically administered by a trained professional, such as a psychologist or \
                        speech-language pathologist. In addition to these standardized tests, the \
                        evaluation process may also include a review of the individual’s medical and \
                        educational history, as well as an assessment of their cognitive abilities and \
                        academic achievement. The evaluation process is typically tailored to the \
                        individual’s specific needs and may involve a combination of testing and other \
                        assessment methods."
                    }
                }
            }
            div(class="w-full bg-white pt-10") {
                div(class="container max-w-screen-2xl prose prose-stone text-center pb-10") {
                    h1(id="testimonials", class="text-sky-700") { "Testimonials" }
                    p { "Read what families say about us on Nextdoor." }
                    a(href=CONTACT_URL, target="_blank", rel="noopener noreferrer", class="not-prose no-underline text-2xl p-2 pt-1 rounded-full text-sky-700 border-2 border-sky-700 hover:border-transparent hover:bg-teal-500") { "Nextdoor" }
                }
            }
            div(class="w-full bg-white pt-10 pb-14") {
                div(class="container max-w-screen-2xl text-center") {
                    a(href=CONTACT_URL, target="_blank", rel="noopener noreferrer", class="not-prose no-underline text-5xl p-4 pt-2 rounded-full text-sky-700 border-8 border-sky-700 hover:border-transparent hover:bg-teal-500") { "Contact Us" }
                }
            }
        }
        footer(class="bg-sky-900") {
            div(class="mx-auto pr-4 pl-4 2xl:p-0 2xl:max-w-screen-2xl") {
                div(class="flex items-center justify-end flex-wrap pt-6 pb-6") {
                    a(href="https://dyslexiaida.org", target="_blank", rel="noopener noreferrer", class="text-sky-50 hover:text-teal-500 pr-4 pl-4") { "dyslexiaida.org" }
                    a(href="https://CHADD.org", target="_blank", rel="noopener noreferrer", class="text-sky-50 hover:text-teal-500 pr-4 pl-4") { "CHADD.org" }
                    a(href="https://ASHA.org", target="_blank", rel="noopener noreferrer", class="text-sky-50 hover:text-teal-500 pr-4 pl-4") { "ASHA.org" }
                    a(href="https://blue.eel.education", target="_blank", rel="noopener noreferrer", class="text-sky-50 hover:text-teal-500 pr-4 pl-4") { "Blue Eel" }
                }
                div(class="text-sky-100 text-right pb-2 mr-2") {
                    "© 2023 Spark Hill. All Rights Reserved."
                }
            }
        }
    }
}

#[engine_only_fn]
fn head(cx: Scope) -> View<SsrNode> {
    view! { cx,
        title { "Spark Hill - Reading Instruction" }
        link(rel = "icon", type = "image/x-icon", href = ".perseus/static/favicon.ico")
        link(rel = "stylesheet", href = ".perseus/static/css/reset.css")
        link(rel = "stylesheet", href = ".perseus/static/css/theme.css")
        link(rel = "stylesheet", href = ".perseus/static/css/index.css")
    }
}

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("index").view(index_page).head(head).build()
}

#[cfg(test)]
mod tests {
    use super::{page, CONTACT_URL};
    use std::collections::HashSet;
    use std::path::Path;

    const HOME: &str = "en-US/";

    fn render() -> String {
        sycamore::render_to_string(|cx| page(cx, "Everyone deserves to read".into(), HOME.into()))
    }

    /// Every value of `attr="..."` in the HTML, in order.
    fn attr_values(html: &str, attr: &str) -> Vec<String> {
        let needle = format!(" {attr}=\"");
        html.split(&needle)
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .map(str::to_string)
            .collect()
    }

    /// Each opening tag in the HTML, as the text between `<` and `>`.
    fn tags(html: &str) -> Vec<&str> {
        html.split('<')
            .skip(1)
            .filter_map(|rest| rest.split('>').next())
            .filter(|tag| !tag.starts_with('/') && !tag.starts_with('!'))
            .collect()
    }

    #[test]
    fn shows_the_tagline_and_every_section() {
        let html = render();
        assert!(html.contains("Everyone deserves to read"));
        for heading in [
            "What we do",
            "Who we are",
            "How we do it",
            "Testimonials",
            "Contact Us",
        ] {
            assert!(html.contains(heading), "missing {heading}");
        }
        assert!(html.contains("<header"));
        assert!(html.contains("<footer"));
    }

    #[test]
    fn has_one_main_landmark() {
        assert_eq!(render().matches("<main").count(), 1);
    }

    #[test]
    fn section_ids_are_unique() {
        let ids = attr_values(&render(), "id");
        let unique: HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len(), "duplicate ids in {ids:?}");
    }

    #[test]
    fn every_page_anchor_has_a_section() {
        let html = render();
        let ids: HashSet<_> = attr_values(&html, "id").into_iter().collect();
        for href in attr_values(&html, "href") {
            if let Some(anchor) = href.strip_prefix(&format!("{HOME}#")) {
                assert!(ids.contains(anchor), "no section for {href}");
            }
        }
    }

    #[test]
    fn links_only_to_pages_that_exist() {
        // The site has one page, so internal links are "/" or its anchors.
        for href in attr_values(&render(), "href") {
            assert!(
                href == "/"
                    || href.starts_with(&format!("{HOME}#"))
                    || href.starts_with("https://"),
                "{href} goes to a page the site doesn't have"
            );
        }
    }

    #[test]
    fn contact_us_goes_somewhere_that_takes_messages() {
        let html = render();
        let contact = format!("href=\"{CONTACT_URL}\"");
        assert!(html.matches(&contact).count() >= 2);
    }

    #[test]
    fn has_no_placeholder_testimonials() {
        let html = render();
        assert!(!html.contains("Ms. Smith"));
        assert!(!html.contains("Ms. Jones"));
    }

    #[test]
    fn section_links_are_left_to_the_browser() {
        let html = render();
        let section_links: Vec<_> = tags(&html)
            .into_iter()
            .filter(|t| t.contains(&format!("href=\"{HOME}#")))
            .collect();
        assert_eq!(section_links.len(), 3);
        for tag in section_links {
            assert!(tag.contains("rel=\"external\""), "{tag}");
        }
    }

    #[test]
    fn new_tab_links_dont_expose_the_opener() {
        for tag in tags(&render()) {
            if tag.contains("target=\"_blank\"") {
                assert!(tag.contains("rel=\"noopener noreferrer\""), "{tag}");
            }
        }
    }

    #[test]
    fn images_have_alt_text_and_exist() {
        let html = render();
        let images: Vec<_> = tags(&html)
            .into_iter()
            .filter(|t| t.starts_with("img"))
            .collect();
        assert_eq!(images.len(), 2);
        for img in images {
            assert!(img.contains(" alt=\""), "{img} has no alt text");
        }
        for src in attr_values(&html, "src") {
            let file = src
                .strip_prefix(".perseus/static/")
                .unwrap_or_else(|| panic!("{src} isn't a static file"));
            assert!(Path::new("static").join(file).exists(), "{src} missing");
        }
    }
}
