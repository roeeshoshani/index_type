use index_type::IndexType;

#[derive(IndexType)]
#[index_type(error = 123)]
struct BadIndex(u32);

fn main() {}
