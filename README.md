# piko
A distributed message broker protocol &amp; implementation.

### Projects related to this repository 
* [node](./node) - Node implementation.
* [pcm](https://github.com/lyuben-todorov/pcm) - Cluster manager script.
* [piko-cli](https://github.com/lyuben-todorov/piko-cli) - CLI.
* [willis](https://github.com/lyuben-todorov/willis) - Testing & diagnostic.

### Benchmarks
Currently, I'm benchmarking the following situations:
1. n messages to single node in a cluster of m nodes.
2. Split n messages evenly between m nodes.
3. Test 1 but with async client handling.


| # | n    | m     |sample size|seq. client| ops/s        |
|---|------|-------|-----------|-----------|--------------|
| 1 | 1000 | 2     |  20       |yes        | 90.2 ± 0.16 |
| 2 | 1000 | 2     |  20       |yes        | 181 ± 3.05   |
| 3 | 1000 | 2     |  20       |no         | 1850 ± 245   |


Split test does in fact achieve twice the throughput.

Performace doesn't degrade over time. Here are the results after 20 sequential test 1s:
![](https://i.imgur.com/vJuGeOU.png)

Test 2 doesn't work with async client handling and regardless the throughput is low.

This means the project will migrate to tokio.
