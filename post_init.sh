cargo install diesel_cli --no-default-features --features sqlite
diesel setup
diesel migration run
