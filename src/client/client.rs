struct client {
    connect: fn() -> Result<()>,
    get_file: fn(String) -> Result<Stream>,
}

impl client {
    fn new(connect: fn() -> Result<()>, get_files: fn() -> Result<Stream>) -> Self {
        client { connect, get_files }
    }
}
