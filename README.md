# Weather Track

aktuell auf Deutsch, evtl auf Englisch übersetzen falls benötigt

## Beschreibung

Webanwendung wo man Städte eingeben kann für die dann alle 10min das aktuelle Wetter protokolliert wird, dann aggegriert wird. Und letztendlich
dann in einer Tabelle über einen Zeitraum angezeigt wird.

{ platzhalter bilder }

## Deployment

### Directly

env variable OPENWEATHERMAP_APIKEY setzen bzw. in .env file speichern (siehe .env.example)

```sh
cargo build --release
./target/release/weather_track (<port>)
```

default port = 8080

### Docker

evtl. noch die docker-compose anpassen (wegen port und api key)

```sh
docker pull ghcr.io/slayernominee/weather_track:latest
docker-compose up -d
```

### ...

und dann einfach hinter nem reverse proxy betreiben + wenn gewollt einfach nen http auth davor machen
