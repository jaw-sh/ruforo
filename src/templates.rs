use askama_actix::Template;

#[derive(Template)]
#[template(path = "create_user.html")]
pub struct CreateUserTemplate<'a> {
    pub logged_in: bool,
    pub username: Option<&'a str>,
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
