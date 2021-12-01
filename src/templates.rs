use askama_actix::Template;

#[derive(Template)]
#[template(path = "hello.html")]
pub struct HelloTemplate<'a> {
	pub name: &'a str,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
	pub logged_in: bool,
	pub username: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
	pub logged_in: bool,
	pub username: Option<&'a str>,
}

