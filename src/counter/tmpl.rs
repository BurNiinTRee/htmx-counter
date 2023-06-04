use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub(super) struct Counter {
    pub count: i64,
}
