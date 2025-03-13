# Weather Track

aktuell auf Deutsch, evtl auf Englisch übersetzen falls benötigt

## Beschreibung

Webanwendung wo man Städte eingeben kann für die dann alle 10min das aktuelle Wetter protokolliert wird, dann aggegriert wird. Und letztendlich
dann in einer Tabelle über einen Zeitraum angezeigt wird.

## Deployment

```sh
cargo build --release
./target/release/weather_track (<port>)
```

default port = 8080

und dann einfach hinter nem reverse proxy betreiben + wenn gewollt einfach nen http auth davor machen

## TODOs

- paar bilder hinzufügen
- regenerate openweathermaps api key
