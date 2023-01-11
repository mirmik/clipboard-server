use clap::{Arg, App};
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, BufRead};
use std::io::Write;
use json::JsonValue;
use base64 as b64;


static DEFAULT_PORT : &str = "10304";

static mut CLIPBOARD_GLOBAL : String = String::new();

fn ok_result() -> JsonValue {
    let mut json = JsonValue::new_object();
    json["status"] = "ok".into();
    json
}

fn clipboard_update(clipboard: String) {
    println!("Clipboard updated: {}", &clipboard);
    unsafe { CLIPBOARD_GLOBAL= clipboard; }
}

fn clipboard_upload_command(json : JsonValue) -> JsonValue {
    let clipboard = json["data"].as_str().unwrap();
    clipboard_update(clipboard.to_string());
    ok_result()
}

fn cliboard_upload_base64_command(json : JsonValue) -> JsonValue {
    let clipboard = json["data"].as_str().unwrap();
    let decoded = b64::decode(clipboard).unwrap();
    let decoded_string = String::from_utf8(decoded).unwrap();
    clipboard_update(decoded_string);   
    ok_result()
}

fn clipboard_download_command(_ : JsonValue) -> JsonValue {
    let mut json = JsonValue::new_object();
    json["status"] = "ok".into();
    unsafe { json["data"] = CLIPBOARD_GLOBAL.clone().into(); }
    json
}

fn clipboard_download_base64_command(_ : JsonValue) -> JsonValue {
    let mut json = JsonValue::new_object();
    json["status"] = "ok".into();
    unsafe { 
        let encoded = b64::encode(CLIPBOARD_GLOBAL.clone());
        json["data"] = encoded.into();
    }
    json
}

fn unknown_command(input : JsonValue) -> JsonValue {
    let cmd = input["command"].as_str().unwrap();
    let mut json = JsonValue::new_object();
    json["status"] = "error".into();
    json["message"] = format!("Unknown command: {}", cmd).into();
    json
}

fn command_analyze(line: String) -> String {
    // parse json with error handling
    let json = match json::parse(&line) {
        Ok(v) => v,
        Err(e) => {
            let mut json = JsonValue::new_object();
            json["status"] = "error".into();
            json["message"] = format!("Error parsing json: {}", e).into();
            return json.dump();
        }
    };
    
    let command = json["command"].as_str().unwrap();
    let res = match command {
        "clipboard_upload" => clipboard_upload_command(json),
        "clipboard_upload_base64" => cliboard_upload_base64_command(json),
        "clipboard_download" => clipboard_download_command(json),
        "clipboard_download_base64" => clipboard_download_base64_command(json),
        _ => unknown_command(json),
    };

    let res_string = res.dump();
    println!("Result: {}", res_string);
    res_string
}

fn stream_handler(stream: TcpStream) {
    let mut reader = BufReader::new(stream);

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).unwrap();
        if bytes_read == 0 {
            break;
        }
        let res = command_analyze(line);
        let res_plus_newline = format!("{}\n", res);
        reader.get_mut().write(res_plus_newline.as_bytes()).unwrap();
    }
}

fn main() {
    let matches = App::new("My Test Program")
        .version("0.1.0")
        .author("Hackerman Jones <hckrmnjones@hack.gov>")
        .about("Teaches argument parsing")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("Sets a custom port")
            .takes_value(true))
        .get_matches();

    let port = matches.value_of("port").unwrap_or(DEFAULT_PORT);
    println!("Value for port: {}", port);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                stream_handler(stream);
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }
}