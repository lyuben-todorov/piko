# piko
A distributed message broker protocol &amp; implementation.

### Projects related to this repository 
* [node](./node) - Stand-alone node implementation.
* [pcm](https://github.com/lyuben-todorov/pcm) - Cluster manager script
* [piko-cli](https://github.com/lyuben-todorov/piko-cli) - CLI for the node.
* [willis](https://github.com/lyuben-todorov/willis) - Testing & diagnostic tool.

### Benchmarks
Currently, I'm benchmarking the following situations:
1. n messages to single node in a cluster of m nodes.
2. Split n messages evenly between m nodes

| # | n    | m     |sample size| ops/s        |
|---|------|-------|-----------|--------------|
| 1 | 1000 | 2     |  12       | 90.5 ± 0.204 |
| 2 | 1000 | 2     |  12       | 181 ± 3.05   |

Split test does in fact achieve twice the throughput which I didn't really expect.

Throughput doesn't degrade over time. Here are the results after 20 sequential test 1s:
```
Test took 10.95s, avg 91.28832617671608 ops/s
Test took 11.13s, avg 89.84989167916743 ops/s
Test took 11.12s, avg 89.93355580202427 ops/s
Test took 11.03s, avg 90.69839532931583 ops/s
Test took 11.09s, avg 90.17601433784777 ops/s
Test took 11.09s, avg 90.19428649514765 ops/s
Test took 11.08s, avg 90.23900903903723 ops/s
Test took 11.12s, avg 89.91824617951093 ops/s
Test took 11.07s, avg 90.34364315462742 ops/s
Test took 11.10s, avg 90.0993192231039 ops/s
Test took 11.11s, avg 90.02371945410295 ops/s
Test took 11.16s, avg 89.597966342346 ops/s
Test took 11.06s, avg 90.45435916584853 ops/s
Test took 11.12s, avg 89.90486013539433 ops/s
Test took 11.10s, avg 90.10758589850738 ops/s
Test took 11.13s, avg 89.8839059973488 ops/s
Test took 11.12s, avg 89.96529986294377 ops/s
Test took 11.11s, avg 90.00471254414344 ops/s
Test took 11.07s, avg 90.34414629414599 ops/s
Test took 11.06s, avg 90.38522595367704 ops/s
```
![](https://i.imgur.com/vJuGeOU.png)
