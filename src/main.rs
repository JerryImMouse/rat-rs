use std::env;
use rat::*;


fn main() {
    let raw_args = env::args().collect::<Vec<String>>();
    let rat_args = RatArgs::new(raw_args);

    let rat = Rat::new(rat_args, std::io::stdout());

    rat.exec();
}
