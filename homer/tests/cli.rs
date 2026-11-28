//! CLI transcript tests: run the real `homer` binary and compare its output
//! with the transcripts in `tests/cmd/`. A local mockito server stands in for
//! the Zillow API, so these never call RapidAPI. Cases that look a property
//! up run in a sandbox folder and keep the quota in `quota.json` there.
//!
//! To update a transcript after an intended output change, run
//! `TRYCMD=overwrite cargo test --test cli`.

#![allow(clippy::expect_used)]

#[test]
fn cli_transcripts() {
    let mut server = mockito::Server::new();
    let _property = server
        .mock("GET", "/property")
        .match_query(mockito::Matcher::UrlEncoded(
            "address".into(),
            "123 Main St, Seattle, WA 98101".into(),
        ))
        .match_header("x-rapidapi-key", "test-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "zpid": 48749425,
                "address": {
                    "streetAddress": "123 Main St",
                    "city": "Seattle",
                    "state": "WA",
                    "zipcode": "98101"
                },
                "price": 750000,
                "bedrooms": 3,
                "bathrooms": 2.5,
                "livingArea": 1850,
                "yearBuilt": 1998,
                "homeType": "SINGLE_FAMILY",
                "hdpUrl": "/homedetails/123-Main-St-Seattle-WA-98101/1_zpid/"
            }"#,
        )
        .create();
    // Zillow matches loosely: this address finds a different house.
    let _fuzzy = server
        .mock("GET", "/property")
        .match_query(mockito::Matcher::UrlEncoded(
            "address".into(),
            "123 Main, Seattle, WA".into(),
        ))
        .match_header("x-rapidapi-key", "test-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "zpid": 48749999,
                "address": {
                    "streetAddress": "1230 Main St",
                    "city": "Seattle",
                    "state": "WA",
                    "zipcode": "98102"
                },
                "price": 615000,
                "bedrooms": 2
            }"#,
        )
        .create();
    // RapidAPI answers 200 with no property when Zillow has no match.
    let _not_found = server
        .mock("GET", "/property")
        .match_query(mockito::Matcher::UrlEncoded(
            "address".into(),
            "1 Nowhere Rd, Nowhere, WA 00000".into(),
        ))
        .match_header("x-rapidapi-key", "test-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "No property found for this address"}"#)
        .create();

    trycmd::TestCases::new()
        .env("HOMER_BASE_URL", server.url())
        .case("tests/cmd/*.toml");
}
