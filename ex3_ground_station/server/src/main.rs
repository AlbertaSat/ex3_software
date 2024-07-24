use rocket::fs::{relative, FileServer};
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use dotenv::dotenv;

#[macro_use]
extern crate rocket;

mod obc_client;
mod message;
mod types;
mod utils;

use types::command::Command;
use utils::file_ops::{read_commands, write_command};
use obc_client::ObcClient;

static OBC_CLIENT: Lazy<Mutex<ObcClient>> = Lazy::new(|| Mutex::new(
    ObcClient::new("localhost".to_string(), 50000)
));

#[get("/api/cmd", format="json")]
async fn get_cmds() -> Json<Vec<Command>> {
    let commands = read_commands().expect("Could not read commands");
    Json(commands)
}

#[post("/api/cmd", format = "json", data = "<input>")]
async fn post_cmd(input: Json<Command>) -> Status {
    println!("Got a form! Payload: {}, Cmd: {}, Data: {}", input.payload, input.cmd, input.data);

    let new_command = input.into_inner();

    write_command(new_command.clone()).expect("Could not write command");

    let mut client = OBC_CLIENT.lock().await;
    match client.send_cmd([
        new_command.payload.as_str(),
        new_command.cmd.as_str(),
        new_command.data.as_str()
    ]).await {
        Ok(rc) => println!("Client response: {}", rc),
        Err(e) => println!("Client send error: {}", e),
    };

    Status::Ok
}

#[options("/api/cmd")]
fn options_cors() -> Status {
    Status::Ok
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok(); 

    let mut client = OBC_CLIENT.lock().await;
    match client.connect().await {
        Ok(_) => println!("Connected to OBC"),
        Err(e) => println!("Connection error: {}", e),
    }

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_headers(AllowedHeaders::all())
        .allow_credentials(true)
        .to_cors().expect("Error creating CORS options");

    rocket::build()
        .mount("/", routes![get_cmds, post_cmd, options_cors])
        .mount("/", FileServer::from(relative!("static")))
        .attach(cors)
}
