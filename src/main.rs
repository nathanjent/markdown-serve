extern crate tera;
extern crate markdown;
extern crate serde_json;

use std::env;
use std::io::prelude::*;
use std::fs::OpenOptions;
use std::collections::HashMap;
use tera::{Context, Tera, TeraResult, TeraError, Value, to_value};

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

fn main() {
    let mut tera = Tera::new("templates/**/*");
    tera.register_filter("markdown", markdown_filter);
    let mut ctx = Context::new();

    if let Some(file_name) = env::args().nth(1) {
        println!("{}", file_name);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_name);
        if let Ok(mut in_file) = file {
            let mut input = String::new();
            in_file.read_to_string(&mut input).expect("File read fail.");
            ctx.add("content", &&*input);
        }
    } else {
        ctx.add("content", &"**bold** and `beautiful`");
    }

    let rendered = tera.render("md_body.html", ctx).expect("Failed to render template");
    println!("{}", rendered);
}
