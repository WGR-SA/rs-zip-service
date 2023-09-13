struct zipper {
    create_zip: fn(Vector<Stream>),
    serve_zip: fn(Stream) -> Stream,
}

impl zipper {
    fn new(create_zip: fn(Vector<Stream>), serve_zip: fn(Stream) -> Stream) -> Self {
        zipper {
            create_zip,
            serve_zip,
        }
    }
}
