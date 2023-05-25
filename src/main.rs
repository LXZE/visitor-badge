use std::fs;

#[macro_use]
extern crate diesel;
use actix_web::{error, get, web, middleware, App, HttpResponse, HttpServer, Responder, Result};
use diesel::{prelude::*, r2d2};

use ab_glyph::FontArc;
extern crate shield_maker;
use shield_maker::{Renderer, Metadata, Style, FontFamily};

mod actions;
mod models;
mod schema;

type DbPool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

#[get("/")]
async fn get_badge(pool: web::Data<DbPool>, font: web::Data<FontArc>) -> Result<impl Responder> {
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
        Some(visitor) => {
            let count = visitor.view_count.to_string();
            let count_slice = &count[..];

            let badge_meta = &Metadata {
                style: Style::FlatSquare,
                label: "Profile views",
                message: count_slice,
                font: font.get_ref().clone(),
                font_family: FontFamily::Default,
                label_color: None,
                color: Some("orange"),
            };
            let badge_output = Renderer::render(badge_meta);
            HttpResponse::Ok()
                .insert_header(("Content-Type", "image/svg+xml;charset=utf-8"))
                .insert_header(("Cache-Control", "max-age=120, s-maxage=120"))
                .body(badge_output)
            // HttpResponse::Ok()
            //     .body(count)
        },
        None => HttpResponse::NotFound().body("query error"),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool = initialize_db_pool();
    let font_bytes = fs::read("src/fonts/DejaVuSans.ttf")
        .expect("could not read DejaVuSans.ttf");
    let font = FontArc::try_from_vec(font_bytes)
        .expect("could not parse DejaVuSans.ttf");

    log::info!("starting Actix HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(font.clone()))
            .wrap(middleware::Logger::default())
            .service(get_badge)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file")
}
