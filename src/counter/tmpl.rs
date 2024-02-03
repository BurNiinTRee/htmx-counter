use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub(super) struct Counter {
    pub count: i64,
}

#[derive(Template)]
#[template(path = "count.html")]
pub(super) struct Count {
    pub count: i64,
}
