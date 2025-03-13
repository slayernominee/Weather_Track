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
        "CREATE TABLE IF NOT EXISTS weather (id INTEGER PRIMARY KEY AUTOINCREMENT, timestamp INTEGER NOT NULL, query TEXT, city TEXT NOT NULL, cord_lon REAL NOT NULL, cord_lat REAL NOT NULL, weather_id INTEGER, weather_main TEXT, weather_description TEXT, weather_icon TEXT, base TEXT, main_temp REAL, main_feels_like REAL, main_temp_min REAL, main_temp_max REAL, main_pressure INTEGER, main_humidity INTEGER, main_sea_level INTEGER, main_grnd_level INTEGER, visibility INTEGER, wind_speed REAL, wind_deg INTEGER, wind_gust REAL, rain_1h REAL, clouds_all INTEGER, dt INTEGER, sys_country TEXT, sys_sunrise INTEGER, sys_sunset INTEGER, timezone INTEGER, city_id INTEGER, name TEXT, cod INTEGER)",
        [],
    )
    ?;

    Ok(())
}

pub fn dump_weather(timestamp: i64, query: &str, weather: WeatherResponse) -> SqliteResult<()> {
    let conn = Connection::open("weather.db")?;

    conn.execute(
            "INSERT INTO weather (timestamp, query, city, cord_lon, cord_lat, weather_id, weather_main, weather_description, weather_icon, base, main_temp, main_feels_like, main_temp_min, main_temp_max, main_pressure, main_humidity, main_sea_level, main_grnd_level, visibility, wind_speed, wind_deg, wind_gust, rain_1h, clouds_all, dt, sys_country, sys_sunrise, sys_sunset, timezone, city_id, name, cod)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32)",
            params![
                timestamp,
                query,
                weather.name,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct WeatherData {
    pub id: usize,
    pub timestamp: u64,
    pub query: String,
    pub coord: Coord,
    pub weather: Weather,
    pub base: String,
    pub main: Main,
    pub visibility: u32,
    pub wind: Wind,
    pub rain: Rain,
    pub clouds: Clouds,
    pub dt: u64,
    pub sys: Sys,
    pub timezone: i32,
    pub name: String,
    pub cod: u32,
}

pub fn get_weather(query: &str) -> SqliteResult<Vec<WeatherData>> {
    let conn = Connection::open("weather.db")?;
    let mut stmt = conn.prepare("SELECT * FROM weather WHERE query = ?1 ORDER BY id DESC")?;
    let weather_iter = stmt.query_map(params![query], |row| {
        Ok(WeatherData {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            query: row.get(2)?,

            coord: Coord {
                lon: row.get(4)?,
                lat: row.get(5)?,
            },
            weather: Weather {
                id: row.get(6)?,
                main: row.get(7)?,
                description: row.get(8)?,
                icon: row.get(9)?,
            },
            base: row.get(10)?,
            main: Main {
                temp: row.get(11)?,
                feels_like: row.get(12)?,
                temp_min: row.get(13)?,
                temp_max: row.get(14)?,
                pressure: row.get(15)?,
                humidity: row.get(16)?,
                sea_level: row.get(17)?,
                grnd_level: row.get(18)?,
            },
            visibility: row.get(19)?,
            wind: Wind {
                speed: row.get(20)?,
                deg: row.get(21)?,
                gust: Some(row.get(22)?),
            },
            rain: Rain {
                one_hour: Some(row.get(23)?),
            },
            clouds: Clouds { all: row.get(24)? },
            dt: row.get(25)?,
            sys: Sys {
                country: row.get(26)?,
                sunrise: row.get(27)?,
                sunset: row.get(28)?,
            },
            timezone: row.get(29)?,
            name: row.get(31)?,
            cod: row.get(32)?,
        })
    })?;

    let mut weather_data = Vec::new();
    for weather in weather_iter {
        weather_data.push(weather.unwrap());
    }

    Ok(weather_data)
}
