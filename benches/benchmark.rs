use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware::Logger, web, App};
use awc::Client;
use criterion::{criterion_group, criterion_main, Criterion};
use futures_util::future::join_all;
use ruforo::{
    create_user, filesystem, forum, frontend, index, init, login, logout, member,
    middleware::AppendContext, post, session, thread,
};

// Shamelessly stolen from https://github.com/actix/actix-web/blob/master/benches/server.rs
// benchmark sending all requests at the same time
fn bench_view_css(c: &mut Criterion) {
    // We are using System here, since Runtime requires preinitialized tokio
    let rt = actix_rt::System::new();
    let srv =
        rt.block_on(async { actix_test::start(|| App::new().service(frontend::css::view_css)) });
    let url = srv.url("/style.css");
    c.bench_function("bench_view_css", move |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let client = Client::new().get(url.clone()).freeze().unwrap();
                let start = std::time::Instant::now();
                let burst = (0..iters).map(|_| client.send());
                let resps = join_all(burst).await;
                let elapsed = start.elapsed();
                let failed = resps.iter().filter(|r| r.is_err()).count();
                if failed > 0 {
                    eprintln!("failed {} requests (might be bench timeout)", failed);
                };

                elapsed
            })
        })
    });
}

fn bench_view_thread(c: &mut Criterion) {
    init::init();

    let rt = actix_rt::System::new();
    let data = rt.block_on(async {
        init::init_db().await;
        web::Data::new(session::init_data().await)
    });
    let srv = rt.block_on(async {
        actix_test::start(move || {
            let policy = CookieIdentityPolicy::new(&[0; 32]) // TODO: Set a 32B Salt
                .name("auth")
                .secure(true);
            App::new()
                .app_data(data.clone())
                // Order of middleware IS IMPORTANT and is in REVERSE EXECUTION ORDER.
                .wrap(AppendContext::default())
                .wrap(IdentityService::new(policy))
                .wrap(Logger::new("%a %{User-Agent}i"))
                // https://www.restapitutorial.com/lessons/httpmethods.html
                // GET    edit_ (get edit form)
                // PATCH  update_ (apply edit)
                // GET    view_ (read/view/render entity)
                // Note: PUT and PATCH were added, removed, and re-added(?) to the HTML5 spec for <form method="">
                .service(index::view_index)
                .service(create_user::create_user_get)
                .service(create_user::create_user_post)
                .service(login::view_login)
                .service(login::post_login)
                .service(logout::view_logout)
                .service(member::view_members)
                .service(filesystem::view_file_ugc)
                .service(filesystem::view_file_canonical)
                .service(filesystem::put_file)
                .service(post::delete_post)
                .service(post::destroy_post)
                .service(post::edit_post)
                .service(post::update_post)
                .service(post::view_post_by_id)
                .service(post::view_post_in_thread)
                .service(forum::create_thread)
                .service(forum::view_forum)
                .service(frontend::css::view_css)
                .service(thread::create_reply)
                .service(thread::view_thread)
                .service(thread::view_thread_page)
        })
    });
    let url = srv.url("/threads/1/");

    c.bench_function("bench_view_thread", move |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let client = Client::new().get(url.clone()).freeze().unwrap();
                let start = std::time::Instant::now();
                let burst = (0..iters).map(|_| client.send());
                let resps = join_all(burst).await;
                let elapsed = start.elapsed();
                let failed = resps.iter().filter(|r| r.is_err()).count();
                if failed > 0 {
                    eprintln!("failed {} requests (might be bench timeout)", failed);
                };

                elapsed
            })
        })
    });
}

criterion_group!(server_benches, bench_view_css, bench_view_thread);
criterion_main!(server_benches);
