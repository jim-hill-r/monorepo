use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Experience<'a> {
    pub title: &'a str,
    pub company: &'a str,
    pub timeframe: &'a str,
    pub description: Vec<&'a str>,
}
