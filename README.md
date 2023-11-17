create a .env file:
```bash
XXX_STORAGE_PROVIDER=openstack
XXX_STORAGE_URL=
XXX_STORAGE_USER=
XXX_STORAGE_SECRET=
XXX_STORAGE_REGION=
XXX_STORAGE_BUCKET=

YYY_STORAGE_PROVIDER=s3
...
PORT=3000

```

```bash
RUST_LOG=info cargo run 
cargo build --release
```

then add /etc/systemd/system a zip.service file

```bash
[Unit]
Description=Rust Actix Zip Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/var/www/rs-zip-service
Environment="RUST_LOG=info"
ExecStart=/var/www/rs-zip-service/target/release/zip_service
Restart=always
StandardOutput=append:/var/log/zip_service.log
StandardError=append:/var/log/zip_service-error.log

[Install]
WantedBy=multi-user.target
````

run

```bash
sudo systemctl start zip.service
```

then test!

```bash
wget --quiet \
  --method GET \
  --header 'X-Client: CLIENT_A_SECRET' \
  --header 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJjbGllbnQiOiJDTElFTlRfQV9TRUNSRVQiLCJleHAiOjE3MzE3ODQ4ODZ9.efHZ435qFrds-7bprQAy-fy5YbjeL4rerZIY6ruefC8' \
  --output-document \
  - http://localhost:8080/
```