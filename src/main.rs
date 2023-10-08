use actix_files::{self};
use actix_web::{get, web, App, Error, HttpResponse, HttpServer};
use glob::glob;
use lazy_static::lazy_static;
use log::{error, info};
use std::{path::PathBuf, process::exit};
use tera::Tera;

async fn render_html(req: actix_web::HttpRequest, tmpl: web::Data<tera::Tera>) -> HttpResponse {
    // Extract filename from the request path
    let filename_pattern = format!("assets/**/{}", req.match_info().query("filename"));

    let filename = glob(&filename_pattern);

    let check_filename = match filename {
        Ok(paths) => {
            let valid_paths: Vec<PathBuf> = paths.filter_map(Result::ok).collect();

            for path in &valid_paths {
                if !path.exists() {
                    return notfound_handler().await;
                }
            }

            let valid_filenames: Vec<String> = valid_paths
                .iter()
                .filter_map(|path| {
                    if path.to_string_lossy().contains("html_separated/") {
                        None
                    } else {
                        path.strip_prefix("assets/")
                            .ok()
                            .map(|stripped_path| stripped_path.to_string_lossy().to_string())
                    }
                })
                .collect();
            valid_filenames.first().cloned()
        }
        Err(err) => {
            error!("Error finding forbidden file: {}", err);
            None
        }
    };

    let filename_pathbuf = match &check_filename {
        Some(filename) => PathBuf::from(filename),
        None => {
            let message: String = "Error transforming checked_filename to pathbuf.".to_string();
            info!("{} NOTE: This is a info since if the user goes to a non existing page this will be also triggered.", message);
            return HttpResponse::InternalServerError().body(message);
        }
    };
    let mut context = tera::Context::new();
    context.insert(
        "filename",
        &filename_pathbuf
            .with_extension("")
            .file_name()
            .ok_or_else(|| {
                "Error inserting filename to context"
                    .to_string()
                    .replace('_', " ")
            })
            .map(|os_str| os_str.to_string_lossy().to_string()) // Convert &OsStr to String
            .map_err(|err| error!("Error inserting filename to context: {}", err))
            .unwrap_or_else(|err| {
                let message = format!("Error inserting the context filename {:?}", err);
                error!("{}", message);
                message
            }),
    );

    match check_filename {
        Some(filename_final) => match tmpl.render(&filename_final, &context) {
            Ok(rendered_html) => HttpResponse::Ok().body(rendered_html),
            Err(err) => {
                error!("Error rendering template: {}", err);
                servererror_handler().await
            }
        },
        None => HttpResponse::Forbidden().body("Access denied: file is not accessible."),
    }
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
