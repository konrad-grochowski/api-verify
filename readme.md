# Api verify
Application for testing API using 2 factor authentication.
The tests consist of scenarios which send requests to either public or private API.
# Necessary files to use the project
* Json schemas in the "./schemas" catalogue; precisely:
    * asset_pair_schema.json
    * server_time_schema.json
* .env file at the repository root; it has to contain:
    * OTP_SECRET
    * API_KEY
    * API_SECRET
    * API_LINK
    * OPEN_ORDERS_ENDPOINT
    * ASSET_PAIR_ENDPOINT
    * SERVER_TIME_ENDPOINT
# Usage
After updating the needed files, run:
`docker-compose up --build`
to execute the tests.
The results will be present in the "results" directory.
