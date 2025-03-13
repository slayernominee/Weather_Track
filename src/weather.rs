use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherResponse {
    pub coord: Coord,
    pub weather: Vec<Weather>,
    pub base: String,
    pub main: Main,
    pub visibility: Option<u32>, // Visibility might be missing, so it's optional
    pub wind: Option<Wind>,      // Wind data might be missing, so it's optional
    pub rain: Option<Rain>,      // Rain data might be missing, so it's optional
    pub clouds: Option<Clouds>,  // Clouds data might be missing, so it's optional
    pub dt: u64,
    pub sys: Sys,
    pub timezone: i32,
    pub id: u32,
    pub name: String,
    pub cod: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Coord {
    pub lon: f64,
    pub lat: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Weather {
    pub id: u32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Main {
    pub temp: f64,
    pub feels_like: f64,
    pub temp_min: f64,
    pub temp_max: f64,
    pub pressure: u32,
    pub humidity: u32,
    pub sea_level: u32,
    pub grnd_level: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wind {
    pub speed: f64,
    pub deg: u32,
    pub gust: Option<f64>, // Gust data might be missing, so it's optional
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rain {
    #[serde(rename = "1h")]
    pub one_hour: Option<f64>, // Rain data might be missing, so it's optional
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Clouds {
    pub all: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sys {
    pub country: String,
    pub sunrise: u64,
    pub sunset: u64,
}

const API_KEY: &str = "f2f4d7c33eafd6df44b9979716b9e56f";

pub async fn fetch_weather(city: &str) -> Result<WeatherResponse, reqwest::Error> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&lang=de&units=metric",
        city, API_KEY
    );

    let data = reqwest::get(&url).await?.text().await?;

    let weather: WeatherResponse = serde_json::from_str(data.as_str()).unwrap();

    Ok(weather)
}

pub fn init_db() -> SqliteResult<()> {
    let conn = Connection::open("weather.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS weather (id INTEGER PRIMARY KEY AUTOINCREMENT, timestamp INTEGER NOT NULL, day TEXT, query TEXT, cord_lon REAL NOT NULL, cord_lat REAL NOT NULL, weather_id INTEGER, weather_main TEXT, weather_description TEXT, weather_icon TEXT, base TEXT, main_temp REAL, main_feels_like REAL, main_temp_min REAL, main_temp_max REAL, main_pressure INTEGER, main_humidity INTEGER, main_sea_level INTEGER, main_grnd_level INTEGER, visibility INTEGER, wind_speed REAL, wind_deg INTEGER, wind_gust REAL, rain_1h REAL, clouds_all INTEGER, dt INTEGER, country TEXT, sys_sunrise INTEGER, sys_sunset INTEGER, timezone INTEGER, city_id INTEGER, name TEXT, cod INTEGER)",
        [],
    )
    ?;

    Ok(())
}

pub fn dump_weather(query: &str, weather: WeatherResponse) -> SqliteResult<()> {
    let conn = Connection::open("weather.db")?;

    let timestamp = Utc::now().timestamp();

    let datetime = DateTime::from_timestamp(timestamp, 0).unwrap();
    let date_str = datetime.format("%Y-%m-%d").to_string(); // "YYYY-MM-DD"

    conn.execute(
            "INSERT INTO weather (timestamp, day, query, cord_lon, cord_lat, weather_id, weather_main, weather_description, weather_icon, base, main_temp, main_feels_like, main_temp_min, main_temp_max, main_pressure, main_humidity, main_sea_level, main_grnd_level, visibility, wind_speed, wind_deg, wind_gust, rain_1h, clouds_all, dt, country, sys_sunrise, sys_sunset, timezone, city_id, name, cod)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32)",
            params![
                timestamp,
                date_str,
                query,
                weather.coord.lon,
                weather.coord.lat,
                weather.weather.get(0).map_or(0, |w| w.id), // Default to 0 if no weather data
                weather.weather.get(0).map_or("".to_string(), |w| w.main.clone()),
                weather.weather.get(0).map_or("".to_string(), |w| w.description.clone()),
                weather.weather.get(0).map_or("".to_string(), |w| w.icon.clone()),
                weather.base,
                weather.main.temp,
                weather.main.feels_like,
                weather.main.temp_min,
                weather.main.temp_max,
                weather.main.pressure,
                weather.main.humidity,
                weather.main.sea_level,
                weather.main.grnd_level,
                weather.visibility.unwrap_or(0), // Default to 0 if visibility is None
                weather.wind.as_ref().map_or(0.0, |w| w.speed), // Default to 0 if wind data is None
                weather.wind.as_ref().map_or(0, |w| w.deg),
                weather.wind.as_ref().and_then(|w| w.gust).unwrap_or(0.0), // Default to 0 if gust is None
                weather.rain.as_ref().and_then(|r| r.one_hour).unwrap_or(0.0), // Default to 0 if rain data is None
                weather.clouds.as_ref().map_or(0, |c| c.all),
                weather.dt,
                weather.sys.country,
                weather.sys.sunrise,
                weather.sys.sunset,
                weather.timezone,
                weather.id,
                weather.name,
                weather.cod,
            ],
        )?;
    Ok(())
}

#[derive(Debug)]
pub struct WeatherData {
    pub day: String,

    pub min_temp: f32,
    pub max_temp: f32,
    pub avg_temp: f32,

    pub samples: u32,

    pub min_humidity: f32,
    pub max_humidity: f32,
    pub avg_humidity: f32,

    pub min_wind_speed: f32,
    pub max_wind_speed: f32,
    pub avg_wind_speed: f32,

    pub min_rain_1h: f32,
    pub max_rain_1h: f32,
    pub avg_rain_1h: f32,

    pub icons: Vec<String>,

    pub max_wind_gust: f32,

    pub country: String,
    pub cord_lon: f64,
    pub cord_lat: f64,
}

pub fn get_weather(query: &str) -> SqliteResult<Vec<WeatherData>> {
    let conn = Connection::open("weather.db")?;

    let mut stmt = conn.prepare(
        "SELECT day, min(main_temp), max(main_temp), avg(main_temp), count(*), min(main_humidity), max(main_humidity), avg(main_humidity),
       min(wind_speed), max(wind_speed), avg(wind_speed), min(rain_1h), max(rain_1h), avg(rain_1h),
       GROUP_CONCAT(DISTINCT weather_icon) AS icons, max(wind_gust), GROUP_CONCAT(DISTINCT country), cord_lon, cord_lat
        FROM weather WHERE query = ?1 GROUP BY day, cord_lon, cord_lat ORDER BY id DESC",
    )?;
    let weather_iter = stmt.query_map(params![query], |row| {
        let icons: String = row.get(14)?;
        let icons = icons
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        Ok(WeatherData {
            day: row.get(0)?,

            min_temp: row.get(1)?,
            max_temp: row.get(2)?,
            avg_temp: row.get(3)?,

            samples: row.get(4)?,

            min_humidity: row.get(5)?,
            max_humidity: row.get(6)?,
            avg_humidity: row.get(7)?,

            min_wind_speed: row.get(8)?,
            max_wind_speed: row.get(9)?,
            avg_wind_speed: row.get(10)?,

            min_rain_1h: row.get(11)?,
            max_rain_1h: row.get(12)?,
            avg_rain_1h: row.get(13)?,

            icons, // 14

            max_wind_gust: row.get(15)?,

            country: row.get(16)?,
            cord_lon: row.get(17)?,
            cord_lat: row.get(18)?,
        })
    })?;

    let mut weather_data = Vec::new();
    for weather in weather_iter {
        weather_data.push(weather.unwrap());
    }

    Ok(weather_data)
}
