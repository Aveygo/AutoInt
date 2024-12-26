FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y wget libc6 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
RUN wget -O autoint https://github.com/Aveygo/AutoInt/releases/download/0.2.0/autoint_linux_x64_v0.2.0
RUN chmod +x autoint
CMD ["./autoint"]