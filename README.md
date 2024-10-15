Blob grid

This is simple monochrome field for collaborative drawing.

# Building

## Backend

```
cargo build --release
```

## Frontend

```
cd frontend
yarn install
yarn build
```

Then open http://localhost:5173 and enjoy

# Development

## Run backend
```
cargo run
```

### Usage

>>>
Usage: blobgrid [OPTIONS]

Options:
  -p, --port <PORT>                
  -d, --dump-path <DUMP_PATH>      
  -b, --bitmap-path <BITMAP_PATH>  
  -h, --help                       Print help
  -V, --version                    Print version
>>>

## Run frontend
```
cd frontend
yarn install
yarn dev
```

# Production

1. Build release binary

```
cargo build --release
```

2. Take binary from target/release/blobgrid and put somewhere on server
3. Build frontend
```
cd frontend
yarn install
yarn build
```
4. Take frontend from frontend/build and put to path with static on server
5. Configure systemd unit using template
```
config/blobgrid.systemd.conf
```
6. Configure nginx using template
```
config/blobgrid.nginx.conf
```

7. Reload systemd and nginx