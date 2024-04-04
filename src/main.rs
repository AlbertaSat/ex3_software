use crate::obc_client::ObcClient;

#[macro_use] extern crate rocket;

use rocket::fs::{relative, FileServer};
use rocket::serde::{Deserialize, json::Json};
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

mod obc_client;
mod message;

static OBC_CLIENT: Lazy<Mutex<ObcClient>> = Lazy::new(|| Mutex::new(
                            ObcClient::new("localhost".to_string(), 50000)
                                                      ));

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct WebCommand<'r> {
    payload: &'r str,
    cmd: &'r str,
    data: &'r str,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/api/cmd", format = "json", data = "<input>")]
async fn webcmd(input: Json<WebCommand<'_>>) {
    println!("got a form! payload {}, op {}, data {}", input.payload, input.cmd, input.data);
    let mut client = OBC_CLIENT.lock().await;
    match client.send_cmd([input.payload, input.cmd, input.data]).await {
        Ok(rc) => println!("client response {}", rc),
        Err(e) => println!("client send err {}", e),
    };
}

#[launch]
async fn rocket() -> _ {
    let mut client = OBC_CLIENT.lock().await;
    match client.connect().await {
        Ok(_) => println!("connected to obc"),
        Err(e) => println!("connection error: {}", e),
    }
    rocket::build()
        .mount("/", routes![index, webcmd])
        .mount("/", FileServer::from(relative!("static")))
}

