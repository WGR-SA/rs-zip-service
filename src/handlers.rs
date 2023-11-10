use crate::config;
use crate::storage;
use actix_web::{http::header::ContentType, web, HttpRequest, HttpResponse, Responder};

pub async fn handle_compress_files(req: HttpRequest) -> impl Responder {
    println!("handler");

    let client = req.headers().get("X-Client").unwrap().to_str().unwrap();

    let config: config::StorageConfig = config::get_config(client).unwrap();
    println!("config: {:?}", config);

    let storage_instance = storage::get_storage(config);

    //let files =  storage_instance.get_files(req.);
    HttpResponse::Ok().content_type(ContentType::json()).body({
        let body = web::Bytes::from_static(b"zip");
        body
    })
}
