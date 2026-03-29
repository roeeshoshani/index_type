use index_type::IndexTooBigError;

#[derive(IndexTooBigError)]
#[index_too_big_error(msg = 123)]
struct BadMessage;

fn main() {}
