# tsdownsample

[![PyPI Latest Release](https://img.shields.io/pypi/v/tsdownsample.svg)](https://pypi.org/project/tsdownsample/)
[![support-version](https://img.shields.io/pypi/pyversions/tsdownsample)](https://img.shields.io/pypi/pyversions/tsdownsample)
[![Downloads](https://static.pepy.tech/badge/tsdownsample)](https://pepy.tech/project/tsdownsample)
[![CodeQL](https://github.com/predict-idlab/tsdownsample/actions/workflows/codeql.yml/badge.svg)](https://github.com/predict-idlab/tsdownsample/actions/workflows/codeql.yml)
[![Testing](https://github.com/predict-idlab/tsdownsample/actions/workflows/ci-downsample_rs.yml/badge.svg)](https://github.com/predict-idlab/tsdownsample/actions/workflows/ci-downsample_rs.yml)
[![Testing](https://github.com/predict-idlab/tsdownsample/actions/workflows/ci-tsdownsample.yml/badge.svg)](https://github.com/predict-idlab/tsdownsample/actions/workflows/ci-tsdownsample.yml)
<!-- TODO: codecov -->

Extremely fast **time series downsampling 📈** for visualization, written in Rust.

## Features ✨

* **Fast**: written in rust with PyO3 bindings
  - leverages optimized [argminmax](https://github.com/jvdd/argminmax) - which is SIMD accelerated with runtime feature detection
  - scales linearly with the number of data points
  <!-- TODO check if it scales sublinearly -->
  - multithreaded with Rayon (in Rust)
    <details>
      <summary><i>Why we do not use Python multiprocessing</i></summary>
      Citing the <a href="https://pyo3.rs/v0.17.3/parallelism.html">PyO3 docs on parallelism</a>:<br>
      <blockquote>
          CPython has the infamous Global Interpreter Lock, which prevents several threads from executing Python bytecode in parallel. This makes threading in Python a bad fit for CPU-bound tasks and often forces developers to accept the overhead of multiprocessing.
      </blockquote>
      In Rust - which is a compiled language - there is no GIL, so CPU-bound tasks can be parallelized (with <a href="https://github.com/rayon-rs/rayon">Rayon</a>) with little to no overhead.
    </details>
* **Efficient**: memory efficient
  - works on views of the data (no copies)
  - no intermediate data structures are created
* **Flexible**: works on any type of data
    - supported datatypes are 
      - for `x`: `f32`, `f64`, `i16`, `i32`, `i64`, `u16`, `u32`, `u64`, `datetime64`, `timedelta64`
      - for `y`: `f16`, `f32`, `f64`, `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `datetime64`, `timedelta64`, `bool`
    <details>
      <summary><i>!! 🚀 <code>f16</code> <a href="https://github.com/jvdd/argminmax">argminmax</a> is 200-300x faster than numpy</i></summary>
      In contrast with all other data types above, <code>f16</code> is *not* hardware supported (i.e., no instructions for f16) by most modern CPUs!! <br>
      🐌 Programming languages facilitate support for this datatype by either (i) upcasting to <u>f32</u> or (ii) using a software implementation. <br>
      💡 As for argminmax, only comparisons are needed - and thus no arithmetic operations - creating a <u>symmetrical ordinal mapping from <code>f16</code> to <code>i16</code></u> is sufficient. This mapping allows to use the hardware supported scalar and SIMD <code>i16</code> instructions - while not producing any memory overhead 🎉 <br>
      <i>More details are described in <a href="https://github.com/jvdd/argminmax/pull/1">argminmax PR #1</a>.</i>
    </details>
* **Easy to use**: simple & flexible API

## Install

```bash
pip install tsdownsample
```

## Usage

```python
from tsdownsample import MinMaxLTTBDownsampler
import numpy as np

# Create a time series
y = np.random.randn(10_000_000)
x = np.arange(len(y))

# Downsample to 1000 points (assuming constant sampling rate)
s_ds = MinMaxLTTBDownsampler().downsample(y, n_out=1000)

# Select downsampled data
downsampled_y = y[s_ds]

# Downsample to 1000 points using the (possible irregularly spaced) x-data
s_ds = MinMaxLTTBDownsampler().downsample(x, y, n_out=1000)

# Select downsampled data
downsampled_x = x[s_ds]
downsampled_y = y[s_ds]
```

## Downsampling algorithms & API

### Downsampling API 📑

Each downsampling algorithm is implemented as a class that implements a `downsample` method.
The signature of the `downsample` method:

```
downsample([x], y, n_out, **kwargs) -> ndarray[uint64]
```

**Arguments**:
- `x` is optional
- `x` and `y` are both positional arguments
- `n_out` is a mandatory keyword argument that defines the number of output values<sup>*</sup>
- `**kwargs` are optional keyword arguments *(see [table below](#downsampling-algorithms-📈))*:
  - `n_threads`: how many threads to use for multi-threading (default `1`, so no multi-threading)
  - ...

**Returns**: a `ndarray[uint64]` of indices that can be used to index the original data.

<sup>*</sup><i>When there are gaps in the time series, fewer than `n_out` indices may be returned.</i>
### Downsampling algorithms 📈

The following downsampling algorithms (classes) are implemented:

| Downsampler | Description | `**kwargs` |
| ---:| --- |--- |
| `MinMaxDownsampler` | selects the **min and max** value in each bin | `n_threads` |
| `M4Downsampler` | selects the [**min, max, first and last**](https://dl.acm.org/doi/pdf/10.14778/2732951.2732953) value in each bin | `n_threads` |
| `LTTBDownsampler` | performs the [**Largest Triangle Three Buckets**](https://skemman.is/bitstream/1946/15343/3/SS_MSthesis.pdf) algorithm | `n_threads` |
| `MinMaxLTTBDownsampler` | (*new two-step algorithm 🎉*) first selects `n_out` * `minmax_ratio` **min and max** values, then further reduces these to `n_out` values using the **Largest Triangle Three Buckets** algorithm | `n_threads`, `minmax_ratio`<sup>*</sup> |

<sup>*</sup><i>Default value for `minmax_ratio` is 4, which is empirically proven to be a good default. More details here: https://arxiv.org/abs/2305.00332</i>


## Limitations & assumptions 🚨

Assumes;
1. `x`-data is (non-strictly) monotonic increasing (i.e., sorted)
2. no `NaNs` in the data

---

<p align="center">
👤 <i>Jeroen Van Der Donckt</i>
</p>
