FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y wget libc6 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN wget -O autoint https://github.com/Aveygo/AutoInt/releases/download/0.3.1/autoint 
RUN chmod +x autoint
CMD ["./autoint"]