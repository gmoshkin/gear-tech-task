#![feature(available_concurrency)]
#![feature(vec_spare_capacity)]
#![feature(maybe_uninit_extra)]

mod simple;
mod better;

trait TestTask {
    const PARALLELISM_THRESHOLD: usize = 4;

    fn split<T, F, R>(data: Vec<T>, f: F) -> Vec<R>
    where
        F: Fn(T) -> R,
        F: Send + 'static + Copy,
        T: Send + 'static,
        R: Send + 'static,
        ;
}

fn main() {
}
