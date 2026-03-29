use index_type::IndexTooBigError;

#[derive(IndexTooBigError)]
#[index_too_big_error(msg = "bad")]
struct BadError(u8);

fn main() {}
