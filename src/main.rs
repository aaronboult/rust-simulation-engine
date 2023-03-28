use async_std::task::block_on;
use simulation_engine::run;

fn main() {
    block_on(run());
}
