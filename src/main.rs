#[actix_web::main]
async fn main() -> std::io::Result<()> {
    ruforo::init::init().await
}
