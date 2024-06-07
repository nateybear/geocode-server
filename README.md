# Tiger Geocoder API

I made this to have an easier geocoding service to interface with in my other projects. The pre-built images are made to interface with my local [PostGIS](https://postgis.net/) installation, which has the TIGER shapefiles loaded for the state of Texas. 

## Usage

Both routes can be configured with two query parameters:
- `format`: set to either `csv` or `json` to control the output format. Default is `json`.
- `results`: for each address, the TIGER geocoder will return up to `results` number of possible matches. Default is 10.

### Single Address

Make a `POST` request to `/geocode` with `Content Type: application/json` and make the body of the request the string that you want to geocode.

### Multiple Addresses

Make a `POST` request to `/geocode/batch` with `Content Type: application/json`. This time, the body of the request should be a JSON array of the addresses that you want to geocode. The server will by default use a pool of four connections to the database, so you will have faster execution than serial if you are using batch geocoding. The output is again either a single CSV or JSON array, depending on the query parameter.

## Configuration

Configuration consists of two things:
1. A PostGIS installation with the appropriate TIGER shapefiles loaded for your state. See [this tutorial](https://opendesignarch.blogspot.com/2015/09/installing-tiger-geocoder-on-postgis-21.html) to learn how to load shapefiles into your PostGIS installation.
2. A `.env` file in this directory with the following keys:
    - `PGUSER`
    - `PGPASSWORD`
    - `PGDATABASE`
    - `PGHOST`
    - `DATABASE_URL` (optional: this is for "online" SQLx compilation to work)

With these two ingredients, a `docker build .` command should build the Docker image for this service.
