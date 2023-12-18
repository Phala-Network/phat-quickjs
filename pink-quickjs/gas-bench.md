# Gas consuming statistics

| algorithm         | API                     | gas            | gas time(s) | wall time(s) | iter/s (gas time) | iter/s (wall time) | gas time / wall time |
| ----------------- | ----------------------- | -------------- | ----------- | ------------ | ----------------- | ------------------ | -------------------- |
| sha3              | Pink.hash()             | 4617812840     | 0.00462     | 0.0002987    | 216.5             | 3347               | 15.46x               |
| sha3              | @noble/sha3             | 20073274222067 | 20.07       | 0.053905     | 0.049             | 18.55              | 372.37x              |
| sha3              | @noble/sha3 (in Sidevm) | -              | -           | -            | -                 | 150                | -                    |
| scale codec       | Pink.SCALE              | 56486325739    | 0.05648     | 0.007185     | -                 | -                  | 7.86x                |
| scale codec       | Pink.SCALE(js)          | 3007242541960  | 3.007       | 0.03376      | -                 | -                  | 89.07x               |
| empty loop        | js (for loop)           | 1062423527     | 0.00106     | 1.91e-06     | -                 | -                  | 556.18x              |
| empty loop        | rust (for in {})        | 353706         | 3.5371e-07  | 1.91e-06     | -                 | -                  | 9.69x                |
| empty loop        | rust (loop {})          | -              | -           | -            | -                 | -                  | 5.25x                |
| 100K memory alloc | new Uint8Array()        | 29326427077    | 0.02932     | 0.002782     | 34.106            | 359.45             | 10.54x               |
| 1K memory alloc   | new Uint8Array()        | 2238125423     | 0.00223     | 8.045e-05    | 448.43            | 12430.08           | 27.82x               |
| regex             | str.matchAll            | 3107885449     | 0.00311     | 0.000182     | -                 | -                  | 17.08x               |

# Conclusion

- the gas time is always higher than the wall time across different operations.
- The gas time ratio to wall time variant between 5x to 500x when running different workloads.

  The wasmi switched the gas metering to a inprecise way `one fuel per instruction` in [#701](https://github.com/paritytech/wasmi/pull/705).

- The pure js `sha3` hash is one of the worst cases.

  It require turn the gas limit up to 10x or so in order to run a single pure js hash calucation.
