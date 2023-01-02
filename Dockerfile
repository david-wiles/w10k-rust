FROM rust:1.65

WORKDIR /usr/src/w10k
COPY . .

RUN cargo build --release
RUN cp target/release/w10k /usr/local/bin/w10k

#FROM scratch
#COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/w10k"]
