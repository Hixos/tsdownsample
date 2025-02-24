use argminmax::ArgMinMax;

use super::lttb::{lttb_with_x, lttb_without_x};
use super::types::Num;

use super::minmax;
use num_traits::{AsPrimitive, FromPrimitive};

// ----------------------------------- NON-PARALLEL ------------------------------------

// ----------- WITH X

pub fn minmaxlttb_with_x<Tx: Num + AsPrimitive<f64> + FromPrimitive, Ty: Num + AsPrimitive<f64>>(
    x: &[Tx],
    y: &[Ty],
    n_out: usize,
    minmax_ratio: usize,
) -> Vec<usize>
where
    for<'a> &'a [Ty]: ArgMinMax,
{
    minmaxlttb_generic(
        x,
        y,
        n_out,
        minmax_ratio,
        None,
        MinMaxFunctionWithX::Serial(minmax::min_max_with_x),
    )
}

// ----------- WITHOUT X

pub fn minmaxlttb_without_x<Ty: Num + AsPrimitive<f64>>(
    y: &[Ty],
    n_out: usize,
    minmax_ratio: usize,
) -> Vec<usize>
where
    for<'a> &'a [Ty]: ArgMinMax,
{
    minmaxlttb_generic_without_x(
        y,
        n_out,
        minmax_ratio,
        None,
        MinMaxFunctionWithoutX::Serial(minmax::min_max_without_x),
    )
}

// ------------------------------------- PARALLEL --------------------------------------

// ----------- WITH X

pub fn minmaxlttb_with_x_parallel<
    Tx: Num + AsPrimitive<f64> + FromPrimitive + Send + Sync,
    Ty: Num + AsPrimitive<f64> + Send + Sync,
>(
    x: &[Tx],
    y: &[Ty],
    n_out: usize,
    minmax_ratio: usize,
    n_threads: usize,
) -> Vec<usize>
where
    for<'a> &'a [Ty]: ArgMinMax,
{
    minmaxlttb_generic(
        x,
        y,
        n_out,
        minmax_ratio,
        Some(n_threads),
        MinMaxFunctionWithX::Parallel(minmax::min_max_with_x_parallel),
    )
}

// ----------- WITHOUT X

pub fn minmaxlttb_without_x_parallel<Ty: Num + AsPrimitive<f64> + Send + Sync>(
    y: &[Ty],
    n_out: usize,
    minmax_ratio: usize,
    n_threads: usize,
) -> Vec<usize>
where
    for<'a> &'a [Ty]: ArgMinMax,
{
    minmaxlttb_generic_without_x(
        y,
        n_out,
        minmax_ratio,
        Some(n_threads),
        MinMaxFunctionWithoutX::Parallel(minmax::min_max_without_x_parallel),
    )
}

// ----------------------------------- GENERICS ------------------------------------

// types to make function signatures easier to read
type ThreadCount = usize;
type OutputCount = usize;

pub enum MinMaxFunctionWithX<Tx: Num + AsPrimitive<f64>, Ty: Num + AsPrimitive<f64>> {
    Serial(fn(&[Tx], &[Ty], OutputCount) -> Vec<usize>),
    Parallel(fn(&[Tx], &[Ty], OutputCount, ThreadCount) -> Vec<usize>),
}

pub enum MinMaxFunctionWithoutX<Ty: Num + AsPrimitive<f64>> {
    Serial(fn(&[Ty], OutputCount) -> Vec<usize>),
    Parallel(fn(&[Ty], OutputCount, ThreadCount) -> Vec<usize>),
}

#[inline(always)]
pub(crate) fn minmaxlttb_generic<Tx: Num + AsPrimitive<f64>, Ty: Num + AsPrimitive<f64>>(
    x: &[Tx],
    y: &[Ty],
    n_out: usize,
    minmax_ratio: usize,
    n_threads: Option<usize>,
    f_minmax: MinMaxFunctionWithX<Tx, Ty>,
) -> Vec<usize>
where
    for<'a> &'a [Ty]: ArgMinMax,
{
    assert_eq!(x.len(), y.len());
    assert!(minmax_ratio > 1);
    // Apply first min max aggregation (if above ratio)
    if x.len() / n_out > minmax_ratio {
        // Get index of min max points
        let mut index = match f_minmax {
            MinMaxFunctionWithX::Serial(func) => func(
                &x[1..(x.len() - 1)],
                &y[1..(x.len() - 1)],
                n_out * minmax_ratio,
            ),
            MinMaxFunctionWithX::Parallel(func) => func(
                &x[1..(x.len() - 1)],
                &y[1..(x.len() - 1)],
                n_out * minmax_ratio,
                n_threads.unwrap(), // n_threads cannot be None
            ),
        };
        // inplace + 1
        index.iter_mut().for_each(|elem| *elem += 1);
        // Prepend first and last point
        index.insert(0, 0);
        index.push(x.len() - 1);
        // Get x and y values at index
        let x = unsafe {
            index
                .iter()
                .map(|i| *x.get_unchecked(*i))
                .collect::<Vec<Tx>>()
        };
        let y = unsafe {
            index
                .iter()
                .map(|i| *y.get_unchecked(*i))
                .collect::<Vec<Ty>>()
        };
        // Apply lttb on the reduced data
        let index_points_selected = lttb_with_x(x.as_slice(), y.as_slice(), n_out);
        // Return the original index
        return index_points_selected
            .iter()
            .map(|i| index[*i])
            .collect::<Vec<usize>>();
    }
    // Apply lttb on all data when requirement is not met
    lttb_with_x(x, y, n_out)
}

#[inline(always)]
pub(crate) fn minmaxlttb_generic_without_x<Ty: Num + AsPrimitive<f64>>(
    y: &[Ty],
    n_out: usize,
    minmax_ratio: usize,
    n_threads: Option<usize>,
    f_minmax: MinMaxFunctionWithoutX<Ty>,
) -> Vec<usize>
where
    for<'a> &'a [Ty]: ArgMinMax,
{
    assert!(minmax_ratio > 1);
    // Apply first min max aggregation (if above ratio)
    if y.len() / n_out > minmax_ratio {
        // Get index of min max points
        let mut index = match f_minmax {
            MinMaxFunctionWithoutX::Serial(func) => {
                func(&y[1..(y.len() - 1)], n_out * minmax_ratio)
            }
            MinMaxFunctionWithoutX::Parallel(func) => func(
                &y[1..(y.len() - 1)],
                n_out * minmax_ratio,
                n_threads.unwrap(), // n_threads cannot be None
            ),
        };
        // inplace + 1
        index.iter_mut().for_each(|elem| *elem += 1);
        // Prepend first and last point
        index.insert(0, 0);
        index.push(y.len() - 1);
        // Get y values at index
        let y = unsafe {
            index
                .iter()
                .map(|i| *y.get_unchecked(*i))
                .collect::<Vec<Ty>>()
        };
        // Apply lttb on the reduced data (using the preselect data its index)
        let index_points_selected = lttb_with_x(index.as_slice(), y.as_slice(), n_out);
        // Return the original index
        return index_points_selected
            .iter()
            .map(|i| index[*i])
            .collect::<Vec<usize>>();
    }
    // Apply lttb on all data when requirement is not met
    lttb_without_x(y, n_out).to_vec()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use rstest_reuse::{self, *};

    use super::{minmaxlttb_with_x, minmaxlttb_without_x};
    use super::{minmaxlttb_with_x_parallel, minmaxlttb_without_x_parallel};

    use dev_utils::utils;

    fn get_array_f32(n: usize) -> Vec<f32> {
        utils::get_random_array(n, f32::MIN, f32::MAX)
    }

    // Template for the n_threads matrix
    #[template]
    #[rstest]
    #[case(1)]
    #[case(utils::get_all_threads() / 2)]
    #[case(utils::get_all_threads())]
    #[case(utils::get_all_threads() * 2)]
    fn threads(#[case] n_threads: usize) {}

    #[test]
    fn test_minmaxlttb_with_x() {
        let x = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let y = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let sampled_indices = minmaxlttb_with_x(&x, &y, 4, 2);
        assert_eq!(sampled_indices, vec![0, 1, 5, 9]);
    }

    #[test]
    fn test_minmaxlttb_without_x() {
        let y = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let sampled_indices = minmaxlttb_without_x(&y, 4, 2);
        assert_eq!(sampled_indices, vec![0, 1, 5, 9]);
    }

    #[apply(threads)]
    fn test_minmaxlttb_with_x_parallel(n_threads: usize) {
        let x = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let y = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let sampled_indices = minmaxlttb_with_x_parallel(&x, &y, 4, 2, n_threads);
        assert_eq!(sampled_indices, vec![0, 1, 5, 9]);
    }

    #[apply(threads)]
    fn test_minmaxlttb_without_x_parallel(n_threads: usize) {
        let y = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let sampled_indices = minmaxlttb_without_x_parallel(&y, 4, 2, n_threads);
        assert_eq!(sampled_indices, vec![0, 1, 5, 9]);
    }

    #[apply(threads)]
    fn test_many_random_runs_same_output(n_threads: usize) {
        const N: usize = 20_000;
        const N_OUT: usize = 100;
        const MINMAX_RATIO: usize = 5;
        for _ in 0..100 {
            // TODO: test with x
            let arr = get_array_f32(N);
            let idxs1 = minmaxlttb_without_x(arr.as_slice(), N_OUT, MINMAX_RATIO);
            let idxs2 =
                minmaxlttb_without_x_parallel(arr.as_slice(), N_OUT, MINMAX_RATIO, n_threads);
            assert_eq!(idxs1, idxs2);
        }
    }
}
