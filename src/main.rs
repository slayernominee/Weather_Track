use actix_web::rt::time::sleep;
use actix_web::rt::Runtime;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use std::fs;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

mod weather;
use weather::{dump_weather, fetch_weather, get_weather, init_db, WeatherData};

#[get("/")]
async fn index(data: web::Data<Cities>) -> impl Responder {
    let cities: Vec<String> = data.cities.read().unwrap().clone();

    let mut body = "<h1>Weather Track</h1>".to_string();

    body.push_str("<ul>");
    for city in cities {
        body.push_str(format!("<li><a href=\"/weather/{}\">{}</a></li>", city, city).as_str());
    }
    body.push_str("</ul>");

    HttpResponse::Ok().body(body)
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

#[derive(Debug, serde::Deserialize)]
struct CitiesJson {
    cities: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
    })
    .bind((host, port))?
    .run()
    .await
}
