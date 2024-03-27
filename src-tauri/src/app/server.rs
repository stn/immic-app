// use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{http, middleware, web, App, HttpServer};
use tauri::AppHandle;

use crate::plugins::browser;

struct TauriAppState {
    // app: Mutex<AppHandle>,
}

#[actix_web::main]
pub async fn init(_app: AppHandle) -> std::io::Result<()> {
    let tauri_app = web::Data::new(TauriAppState {
        // app: Mutex::new(app.clone()),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|_origin, _req_head| {
                true
            })
            .allowed_methods(vec!["GET", "POST"])
            // .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(tauri_app.clone())
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .service(browser::browserlog)
    })
    .bind(("127.0.0.1", 3294))?
    .run()
    .await
}
