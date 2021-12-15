use actix_web::App;
use awc::Client;
use criterion::{criterion_group, criterion_main, Criterion};
use futures_util::future::join_all;

// Shamelessly stolen from https://github.com/actix/actix-web/blob/master/benches/server.rs
// benchmark sending all requests at the same time
fn bench_view_css(c: &mut Criterion) {
    // We are using System here, since Runtime requires preinitialized tokio
    let rt = actix_rt::System::new();
    let srv = rt.block_on(async {
        actix_test::start(|| App::new().service(ruforo::frontend::css::view_css))
    });
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

criterion_group!(server_benches, bench_view_css);
criterion_main!(server_benches);
