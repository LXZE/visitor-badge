#[macro_use]
extern crate diesel;
use actix_web::{error, get, web, middleware, App, HttpResponse, HttpServer, Responder, Result};
use diesel::{prelude::*, r2d2};

mod actions;
mod models;
mod schema;

type DbPool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

#[get("/")]
async fn get_badge(pool: web::Data<DbPool>) -> Result<impl Responder> {
    let visitor_info = web::block(move || {
        let mut conn = pool.get()?;
        let user = "lxze".to_string();
        actions::update_and_get_user_viewcount(&mut conn, &user)
            .map_err(|err| println!("{:?}", err)).ok();
        actions::get_user_viewcount(&mut conn, &user)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    Ok(match visitor_info {
        Some(visitor) =>
            HttpResponse::Ok().body(
                format!("Profile views: {0}", visitor.view_count)
            ),
        None => HttpResponse::NotFound().body("query error"),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // initialize DB pool outside of `HttpServer::new` so that it is shared across all workers
    let pool = initialize_db_pool();

    log::info!("starting Actix HTTP server at http://localhost:8080");


    // let manager = SqliteConnectionManager::file("/data/sqlite.db");
    // let pool = Pool::new(manager).unwrap();

    // let counter = web::Data::new(AppState {
    //     counter: Mutex::new(0),
    // });

    HttpServer::new(move || {
        App::new()
            // .app_data(counter.clone())
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(get_badge)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn initialize_db_pool() -> DbPool {
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new("/data/sqlite.db");
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file")
}
