use actix_web::rt::time::sleep;
use actix_web::rt::Runtime;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

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
async fn weather_info(city: actix_web::web::Path<String>) -> impl Responder {
    let weather_data = get_weather(city.as_str()).unwrap();

    let template = WeatherTemplate {
        city: city.into_inner(),
        weather_data,
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(template.render().unwrap())
}

#[get("/add_city/{city}")]
async fn add_city(city: actix_web::web::Path<String>, data: web::Data<Cities>) -> impl Responder {
    data.cities.write().unwrap().push(city.to_string());

    // write back to file
    let cities = CitiesJson {
        cities: data.cities.read().unwrap().clone(),
    };

    fs::write("cities.json", serde_json::to_string(&cities).unwrap()).unwrap();

    println!("Added City: {}", city);

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
    let host = "127.0.0.1";
    let port = 8080;

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
