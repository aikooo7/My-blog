use actix_files::{self};
use actix_web::{get, web, App, Error, HttpResponse, HttpServer};
use anyhow::Result;
use glob::glob;
use lazy_static::lazy_static;
use std::{path::PathBuf, process::exit};
use tera::Tera;

async fn render_html(req: actix_web::HttpRequest, tmpl: web::Data<tera::Tera>) -> HttpResponse {
    // Extract filename from the request path
    let filename = req.match_info().query("filename").to_string();
    let filename_final = if filename.contains("html_separated") {
        return HttpResponse::Forbidden().body("Access denied: file is not accessible.");
    } else {
        format!("html/{}", filename)
    };

    let filename_pathbuf = PathBuf::from(filename);
    let mut context = tera::Context::new();
    context.insert(
        "filename",
        &filename_pathbuf
            .with_extension("")
            .to_string_lossy()
            .to_string()
            .replace('_', " "),
    );

    let rendered_html = tmpl
        .render(&filename_final, &context)
        .expect("Error rendering template");

    HttpResponse::Ok().body(rendered_html)
}

#[get("/")]
async fn home() -> Result<HttpResponse, Error> {
    let mut context = tera::Context::new();
    let mut tera = Tera::default();

    tera.add_template_file(
        "assets/html_separated/layout.html",
        Some("html_separated/layout.html"),
    )
    .expect("Error finding layout.html");

    tera.add_template_file("assets/html_separated/index.html", Some("index.html"))
        .expect("Error finding index.html");

    let files: Vec<String> = glob("assets/html/*.html")
        .expect("Error finding html files.")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .with_extension("")
                .file_name()
                .and_then(|os_str| os_str.to_str().map(String::from))
        })
        .collect();

    let filename: Vec<String> = files.iter().map(|file| file.replace('_', " ")).collect();

    context.insert("files", &files);
    context.insert("filename", &filename.join(", "));
    let rendered = tera
        .render("index.html", &context)
        .expect("Error rendering templates.");
    Ok(HttpResponse::Ok().body(rendered))
}

async fn notfound_handler() -> HttpResponse {
    let notfound_html = TEMPLATES.render("html_separated/404.html", &tera::Context::new());
    match notfound_html {
        Ok(notfound_html) => HttpResponse::NotFound()
            .content_type("text/html")
            .body(notfound_html),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(actix_web::web::Data::new(TEMPLATES.clone()))
            .service(home)
            .service(actix_files::Files::new("/dist", "dist").show_files_listing())
            .service(web::resource("/{filename:.+\\.html}").to(render_html))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("assets/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Error parsing templates. {}", e);
                exit(1);
            }
        };
        tera.autoescape_on(vec![".html"]);
        tera
    };
}
