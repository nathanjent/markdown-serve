extern crate tera;
extern crate markdown;
extern crate serde_json;
extern crate tiny_http;

use std::env;
use std::path::Path;
use std::io::prelude::*;
use std::fs::{OpenOptions, read_dir};
use std::collections::HashMap;
use tera::{Context, Tera, TeraResult, TeraError, Value, to_value};
use tiny_http::{Server, Response};

fn main() {
    let dir_name = env::args().nth(1)
        .unwrap_or(env::current_dir().unwrap().to_str().unwrap().to_string());
    let path = Path::new(&*dir_name);
    println!("{}", path.display());
    let tera_path = path.join("templates/**/*");
    println!("{}", tera_path.display());

    let mut tera = Tera::new(
        tera_path.to_str().expect("Failed to get templates path."));
    tera.register_filter("markdown", markdown_filter);

    let server = Server::http("0.0.0.0:8000").unwrap();

    for request in server.incoming_requests() {
        println!("received request! method: {:?}, url: {:?}, headers: {:?}",
                 request.method(),
                 request.url(),
                 request.headers()
                 );

        let mut ctx = Context::new();
        let file_path = Path::new("/");
        let file_path = file_path.join(request.url());
        let file_path = path.join(file_path.file_name().unwrap());
        println!("Requesting file: {}", file_path.display());
        if file_path.is_file() {
            let file = OpenOptions::new()
                .read(true)
                .open(&file_path);
            if let Ok(mut in_file) = file {
                let mut input = String::new();
                in_file.read_to_string(&mut input).expect("File read fail.");
                println!("{}", input);
                ctx.add("content", &&*input);
            } else {
                ctx.add("content", &"**bold** and `beautiful`");
            }
        } else {
            ctx.add("content", &r"
### 404
Requested URL not found."
            );
        }
        let rendered = tera.render("md_body.html", ctx)
            .expect("Failed to render template");
        println!("{}", rendered);

        let response = Response::from_string(rendered)
            .with_header(
                "Content-type: text/html".parse::<tiny_http::Header>().unwrap());
        request.respond(response);
    }
}

/// Macro borrowed from Tera source; was not part of public API
macro_rules! try_get_value {
    ($filter_name:expr, $var_name:expr, $ty:ty, $val:expr) => {{
        match serde_json::value::from_value::<$ty>($val.clone()) {
            Ok(s) => s,
            Err(_) => {
                return
                    Err(TeraError::FilterIncorrectArgType(
                        $filter_name.to_string(),
                        $var_name.to_string(),
                        $val,
                        stringify!($ty).to_string())
                    );
            }
        }
    }};
}

pub fn markdown_filter(value: Value, _: HashMap<String, Value>) -> TeraResult<Value> {
    let s = try_get_value!("markdown", "value", String, value);
    Ok(to_value(markdown::to_html(s.as_str())))
}
