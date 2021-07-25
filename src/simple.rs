use super::TestTask;
use std::thread::*;

/// Simple solution with no external dependencies. Implemented this one while I was on a plane.
/// There was no internet, I couldn't download any crates from crates.io so I had to only use the
/// `std` stuff. There are some issues with this solution, but just for the hell of it I decided
/// not to fix them and leave the code as it was by the time the plane landed.
struct SimpleTestTaskSolution;

impl TestTask for SimpleTestTaskSolution {

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

        let n_jobs = available_concurrency().map(usize::from).unwrap_or(1);
        let n_each = n_tasks / n_jobs;
        let mut n_rem = n_tasks % n_jobs;
        let mut add_one_if_remaining = ||
            if n_rem > 0 {
                n_rem -= 1;
                1
            } else {
                0
            };
        let mut latest_task_i = 0;
        let mut jobs = Vec::with_capacity(n_jobs);
        let mut data = data;

        for _ in 0..n_jobs {
            let n_tasks_for_current_job = n_each + add_one_if_remaining();

            // Using `drain` is not ideal, because it forces a lot of values to be moved each time
            // the next `Drain` is dropped. It's better just to take the values by index from the
            // original `data`
            let cur_tasks = data
                .drain(0..n_tasks_for_current_job)
                .collect::<Vec<_>>();

            jobs.push(
                spawn(move ||
                    (latest_task_i, cur_tasks.into_iter().map(f).collect::<Vec<_>>())
                )
            );

            latest_task_i += n_tasks_for_current_job;
        }

        let mut results = Vec::with_capacity(n_jobs);

        // Using `join` on the thread handles in sequence is bad because there's a desync with how
        // the threads are actually finishing their tasks, e.g. if the 1st thread finishes last,
        // the current thread will be needlessly blocked until it's done and won't be able to
        // process the results which are already done.
        // I should've used the `std::sync::mpsc::channel` to transfer the results, this would've
        // fixed the issue, but I forgot it was in `std` and didn't think to grep for it in the
        // "~/.rustup/toolchains/<version>/share/doc/rust/html/std/" in which I was looking up the
        // docs for `std` modules.
        results.extend(
            jobs.into_iter().map(|job| job.join().expect("One of the tasks panicked"))
        );
        // I should've used a `BTreeMap` here to avoid a relatively heavy use of `sort_by`
        results.sort_by(|(ls, _), (rs, _)| ls.cmp(rs));

        let mut res = Vec::with_capacity(n_tasks);

        for (_, mut rs) in results {
            res.append(&mut rs);
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
            SimpleTestTaskSolution::split(vec![1, 2, 3], |i| i + 1),
            vec![2, 3, 4],
        );
    }

    #[test]
    fn par1() {
        assert_eq!(
            SimpleTestTaskSolution::split(vec![1, 2, 3, 4, 5], |i| i + 1),
            vec![2, 3, 4, 5, 6],
        );
    }

    #[test]
    fn par2() {
        assert_eq!(
            SimpleTestTaskSolution::split(
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

