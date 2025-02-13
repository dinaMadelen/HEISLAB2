mod modules;

use modules::mod1 as mod1;
use modules::mod2 as mod2;

fn main() {
    mod1::hello_func();
    mod2::hello_func();
}