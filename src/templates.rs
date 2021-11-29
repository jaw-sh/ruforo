use askama_actix::Template;

#[derive(Template)]
#[template(path = "hello.html")]
pub struct HelloTemplate<'a> {
	name: &'a str,
}

