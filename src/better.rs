use super::TestTask;
use threadpool::*;
use std::{
    sync::mpsc::channel,
    collections::*,
};

/// A somewhat better solution which uses `threadpool` and `std::sync::mpsc::channel` and fixes
/// some other inefficiencies.
struct BetterTestTaskSolution;

impl TestTask for BetterTestTaskSolution {
    fn split<T, F, R>(data: Vec<T>, f: F) -> Vec<R>
    where
        F: Fn(T) -> R,
        F: Send + 'static + Copy,
        T: Send + 'static,
        R: Send + 'static,
    {
        let n_tasks = data.len();
        if n_tasks <= Self::PARALLELISM_THRESHOLD {
            return data.into_iter().map(f).collect()
        }

        let n_jobs = num_cpus::get();
        let n_each = n_tasks / n_jobs;
        let mut n_rem = n_tasks % n_jobs;
        let mut add_one_if_remaining = ||
            if n_rem > 0 {
                n_rem -= 1;
                1
            } else {
                0
            };
        let mut n_handled_tasks = 0;

        let mut data = data;
        let pool = ThreadPool::new(n_jobs);
        let (tx, rx) = channel();

        for _ in 0..n_jobs {
            let n_cur_tasks = n_each + add_one_if_remaining();
            n_handled_tasks += n_cur_tasks;
            let cur_data = data.split_off(n_tasks - n_handled_tasks);

            let tx = tx.clone();
            pool.execute(move ||
                tx
                    .send((
                        n_tasks - n_handled_tasks,
                        cur_data.into_iter().map(f).collect::<Vec<_>>()
                    ))
                    .expect("Failed to send results through the channel")
            );
        }

        drop(tx);

        let res_map = rx.iter().collect::<BTreeMap<_, _>>();

        let mut res = Vec::with_capacity(n_tasks);

        for (_, mut r_data) in res_map {
            res.append(&mut r_data)
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seq() {
        assert_eq!(
            BetterTestTaskSolution::split(vec![1, 2, 3], |i| i + 1),
            vec![2, 3, 4],
        );
    }

    #[test]
    fn par1() {
        assert_eq!(
            BetterTestTaskSolution::split(vec![1, 2, 3, 4, 5], |i| i + 1),
            vec![2, 3, 4, 5, 6],
        );
    }

    #[test]
    fn par2() {
        assert_eq!(
            BetterTestTaskSolution::split(
                vec![
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
                ],
                |i| i * i
            ),
            vec![
                1, 4, 9, 16, 25, 36, 49, 64, 81, 100, 121, 144, 169, 196, 225, 256, 289, 324, 361,
            ],
        );
    }
}

