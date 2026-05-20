use actix_web::{App, HttpServer, web};
use db::Database;

mod routes;
mod service;

#[derive(Clone)]
pub struct AppData {
    pub db: Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database = Database::init_db().await.unwrap();
    let app_data = AppData {
        db: database
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .service(routes::business::create_business)
            .service(routes::business::create_business_auth_key)
            .service(routes::customer::create_customer)
            .service(routes::customer::get_customer_by_id)
            .service(routes::invoice::create_invoice)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}