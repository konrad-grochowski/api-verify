# Api verify
Application for testing API using 2 factor authentication.
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
