# Introduction
This is just some documentation for future me. Unless you are a smelly nerd, please stick to the [releases](https://github.com/Aveygo/AutoInt/releases).  

# My Hack

I'll begin by saying that AutoInt is a bit of a mess because I am behind a NAT and to serve the reports I use Supabase as a cheap proxy. **This is a dirty hack** and should really not concern 99% of developers. Just use the normal self-hosting solution on an ec2 instance or the docker image on your homelab (```sudo docker run --restart=always -d -p 8000:8000 aveygo/autoint:latest```) and you should be fine.

Regardless, AutoInt work in two states: the "default" self-hosting mode, and the "special" supabase mode.  

If the SUPABASE environment variable **is set** with the provided supabase private key, then the binary goes into the supabase mode and does not host any files. 

If the SUPABASE environment **is NOT set**, then the ```/report``` endpoint is created on the host machine, along with the provided static files.

**MAKE SURE THAT THE SUPABASE KEY IS NOT SET, OR NOTHING WILL APPEAR TO BE WORKING**

# Compiling
Ensure that you have [Rust](https://www.rust-lang.org/tools/install) and [Cross](https://github.com/cross-rs/cross) installed.

Linux: ```cross build --target=x86_64-unknown-linux-musl --release --bin autoint``` **NOTE THE STATIC COMPILATION!!! DOCKER WILL HATE YOU IF YOU FORGET!!!** </br>
Windows: ```cross build --target x86_64-pc-windows-gnu --release --bin autoint``` </br>
Macos: For some reason, ring v0.18.8 refuses to compile for macos </br>

The resulting binary file should be at ```target/release/<platform>/autoint```

# Docker
0. Edit the dockerfile to select the version of autoint to run
1. Build the container: ```sudo docker build --no-cache -t aveygo/autoint:latest .```
2. Push to dockerhub: ```sudo docker push aveygo/autoint:latest``` (will need to be authenticated as Aveygo - use ```*sudo* docker login -u aveygo```)
3. Pulling to target host: ```sudo docker image pull aveygo/autoint:latest```
4. Running in supabase mode: ```sudo docker run -e SUPABASE=PRIVATE_KEY_HERE --restart=always -d -p 8000:8000 aveygo/autoint:latest```