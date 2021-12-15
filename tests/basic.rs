#[cfg(test)]
mod tests {
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_index_get() {
        let mut app = test::init_service(App::new().service(ruforo::frontend::css::view_css)).await;
        let req = test::TestRequest::default().uri("/style.css").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
}
