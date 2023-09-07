FROM debian:buster-slim

ENV TZ Asia/Shanghai

WORKDIR /app

COPY ./target/release/rocket-admin /app/

CMD ["./rocket-admin"]