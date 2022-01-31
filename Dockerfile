FROM rust:1.58

# create a new empty project
RUN USER=root cargo new --bin api_verify
WORKDIR /api_verify

# hack to cache dependencies without cucumber errors
RUN mkdir tests && touch tests/public.rs && touch tests/private.rs
# install and cache dependencies
COPY ./Cargo.lock ./Cargo.toml ./
RUN cargo build && \ 
    rm src/*.rs && \
    rm ./target/debug/deps/api_verify*

# copy source code 
COPY ./tests ./tests
COPY ./features ./features
COPY ./schemas ./schemas
# build the project
RUN cargo build

# run tests
CMD ["cargo", "test"]
