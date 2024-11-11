use rango::Application;

#[tokio::main]
async fn main() {
    let _web_app = Application::new()
        .run()
        .await;
}