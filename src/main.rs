use actix_web::rt::time::sleep;
use actix_web::rt::Runtime;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::time::Duration;
use std::{env, fs, thread};

mod weather;
use weather::{dump_weather, fetch_weather, get_weather, init_db, WeatherData};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub cities: Vec<String>,
}

#[get("/")]
async fn index(data: web::Data<Cities>) -> impl Responder {
    let cities: Vec<String> = data.cities.read().unwrap().clone();

    let template = IndexTemplate { cities };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().unwrap())
}

#[derive(Template)]
#[template(path = "weather.html")]
pub struct WeatherTemplate {
    pub city: String,
    pub weather_data: Vec<WeatherData>,
}

#[get("/weather/{city}")]
async fn weather_info(city: web::Path<String>) -> impl Responder {
    let weather_data = get_weather(city.as_str()).unwrap();

    let template = WeatherTemplate {
        city: city.into_inner(),
        weather_data,
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().unwrap())
}

#[derive(Deserialize)]
struct AddCityForm {
    city: String,
}

#[post("/add_city")]
async fn add_city(form: web::Form<AddCityForm>, data: web::Data<Cities>) -> impl Responder {
    data.cities.write().unwrap().push(form.city.to_string());

    // write back to file
    let cities = CitiesJson {
        cities: data.cities.read().unwrap().clone(),
    };

    fs::write("cities.json", serde_json::to_string(&cities).unwrap()).unwrap();

    println!("Added City: {}", form.city);

    HttpResponse::SeeOther()
        .append_header(("location", "/"))
        .finish()
}

#[derive(Debug, Serialize, Deserialize)]
struct CitiesJson {
    cities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cities {
    cities: RwLock<Vec<String>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let args: Vec<String> = env::args().collect();

    let host = "0.0.0.0";

    let mut port: u16 = 8080;

    if args.len() > 1 {
        port = args[1].parse().unwrap_or(8080);
    }

    let cities = fs::read_to_string("cities.json").unwrap();
    let cities: CitiesJson = serde_json::from_str(cities.as_str()).unwrap();
    let cities = web::Data::new(Cities {
        cities: RwLock::new(cities.cities),
    });

    let cities_for_loop = cities.clone();

    init_db().expect("Failed to init the db");

    thread::spawn(move || {
        Runtime::new().unwrap().block_on(async {
            loop {
                let cts = cities_for_loop.cities.read().unwrap().clone();

                // fetch weather data for each city
                for city in cts {
                    let weather_data = fetch_weather(city.as_str()).await.unwrap();

                    match dump_weather(city.as_str(), weather_data) {
                        Ok(_) => println!("Weather data for {} saved", city),
                        Err(e) => println!("Failed to save weather data for {}: {}", city, e),
                    }
                }

                // sleep 10min before next run
                sleep(Duration::from_secs(10 * 60)).await;
            }
        });
    });

    println!("running on: http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(cities.clone())
            .service(index)
            .service(weather_info)
            .service(add_city)
    })
    .bind((host, port))?
    .run()
    .await
}
