FROM  debian:bookworm-slim
WORKDIR /app

RUN apt update && apt install libssl3 -y

COPY ./target/release/ttbackend ./ttbackend

EXPOSE 3000
CMD [ "./ttbackend" ]
