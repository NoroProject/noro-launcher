//! Response formatter entrypoint for noro-admin.

use serde_json::Value;
use crate::format_tables::*;

pub fn print_response(v: &Value) {
    if let Some(arr) = extract_items(v) {
        if arr.is_empty() {
            println!("\x1b[33m(empty list / no records found)\x1b[0m");
            return;
        }
        if arr[0].get("username").is_some() {
            print_users_table(arr);
        } else if arr[0].get("modloader").is_some() && arr[0].get("name").is_some() {
            print_servers_table(arr);
        } else if arr[0].get("mc_version").is_some() && arr[0].get("version").is_some() {
            print_builds_table(arr);
        } else if arr[0].get("size").is_some() && (arr[0].get("path").is_some() || arr[0].get("file").is_some()) {
            print_files_table(arr);
        } else {
            print_generic_table(arr);
        }
    } else if v.is_object() {
        print_object_card(v);
    } else if v.is_boolean() {
        if v.as_bool().unwrap_or(false) {
            println!("\x1b[32m✔ Success (true)\x1b[0m");
        } else {
            println!("\x1b[31m✖ Failed (false)\x1b[0m");
        }
    } else {
        println!("{}", v);
    }
}

fn extract_items(v: &Value) -> Option<&Vec<Value>> {
    if let Some(arr) = v.as_array() {
        Some(arr)
    } else {
        v.get("items").and_then(|i| i.as_array())
    }
}

fn print_object_card(v: &Value) {
    if let Some(obj) = v.as_object() {
        if obj.len() == 1 && obj.contains_key("ok") {
            println!("\x1b[32m✔ Action executed successfully\x1b[0m");
            return;
        }
        println!("\x1b[1;36m✦ Details ✦\x1b[0m");
        for (k, val) in obj {
            let val_str = match val {
                Value::String(s) => s.clone(),
                Value::Bool(b) => if *b { "\x1b[32mtrue\x1b[0m".into() } else { "\x1b[31mfalse\x1b[0m".into() },
                Value::Null => "\x1b[38;5;244mnull\x1b[0m".into(),
                Value::Array(arr) => {
                    if arr.is_empty() {
                        "\x1b[38;5;244m[]\x1b[0m".into()
                    } else {
                        let items: Vec<String> = arr.iter().map(|item| match item {
                            Value::String(s) => s.clone(),
                            Value::Object(map) => {
                                if let Some(name) = map.get("display_name").or_else(|| map.get("name")).or_else(|| map.get("username")).and_then(|v| v.as_str()) {
                                    name.to_string()
                                } else if let Some(perm) = map.get("permission").and_then(|v| v.as_str()) {
                                    perm.to_string()
                                } else if let Some(prov) = map.get("provider").and_then(|v| v.as_str()) {
                                    let u = map.get("username").and_then(|v| v.as_str()).unwrap_or("");
                                    if u.is_empty() { prov.to_string() } else { format!("{prov}:{u}") }
                                } else {
                                    serde_json::to_string(map).unwrap_or_default()
                                }
                            }
                            _ => item.to_string(),
                        }).collect();
                        items.join(", ")
                    }
                }
                Value::Object(map) => format!("{{{} keys}}", map.len()),
                _ => val.to_string(),
            };
            println!("  \x1b[1;33m{:<24}\x1b[0m : {}", k, val_str);
        }
    } else {
        println!("{}", v);
    }
}
