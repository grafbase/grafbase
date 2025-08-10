# Benchmarks

## Initial

```sh
Case 8: schema: 31 KB / query: 0 KB
Case 16: schema: 205 KB / query: 1 KB
Case 32: schema: 1496 KB / query: 5 KB
Case 48: schema: 4897 KB / query: 12 KB
Case 3@1:2-2: schema: 0 KB / query: 0 KB
Case 4@1:3-4: schema: 1 KB / query: 6 KB
Case 5@1:4-4: schema: 1 KB / query: 23 KB
Case 7@1:4-4: schema: 9 KB / query: 23 KB
Case 9@3:3-4: schema: 17 KB / query: 6 KB
query_plan1/3@1:2-2     time:   [37.054 µs 37.118 µs 37.182 µs]
                        thrpt:  [5.5659 MiB/s 5.5754 MiB/s 5.5850 MiB/s]
                 change:
                        time:   [-4.8156% -4.4063% -4.0191%] (p = 0.00 < 0.05)
                        thrpt:  [+4.1874% +4.6094% +5.0592%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
query_plan1/4@1:3-4     time:   [6.8610 ms 7.0127 ms 7.1637 ms]
                        thrpt:  [912.27 KiB/s 931.90 KiB/s 952.51 KiB/s]
                 change:
                        time:   [-3.0498% -0.9940% +1.1944%] (p = 0.37 > 0.05)
                        thrpt:  [-1.1803% +1.0039% +3.1458%]
                        No change in performance detected.
query_plan1/5@1:4-4     time:   [72.461 ms 72.849 ms 73.542 ms]
                        thrpt:  [320.43 KiB/s 323.48 KiB/s 325.22 KiB/s]
                 change:
                        time:   [-4.2591% -3.1805% -2.1468%] (p = 0.00 < 0.05)
                        thrpt:  [+2.1939% +3.2850% +4.4486%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking query_plan1/7@1:4-4: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.1s, or reduce sample count to 70.
query_plan1/7@1:4-4     time:   [110.96 ms 138.58 ms 166.18 ms]
                        thrpt:  [141.80 KiB/s 170.05 KiB/s 212.37 KiB/s]
                 change:
                        time:   [-27.327% -1.2717% +29.762%] (p = 0.95 > 0.05)
                        thrpt:  [-22.936% +1.2881% +37.603%]
                        No change in performance detected.
query_plan1/9@3:3-4     time:   [44.098 ms 47.229 ms 50.346 ms]
                        thrpt:  [129.81 KiB/s 138.37 KiB/s 148.20 KiB/s]
                 change:
                        time:   [-10.094% -1.5264% +8.3907%] (p = 0.73 > 0.05)
                        thrpt:  [-7.7411% +1.5501% +11.228%]
                        No change in performance detected.
```

# GENE

## Shortest path

# SHORTESTPATH ALGORITHM RESULTS

# ShortestPath - gene42.stp

Main algorithm:
Cost difference: 3 (+2.4%)
Nodes in tree: 130 (39% of graph)
Preparation time: 110.59µs
Growth time: 204.708µs
Total time: 315.298µs

Quick estimate:
Cost difference: 12 (+9.5%)
Time: 28.988µs

# ShortestPath - gene61a.stp

Main algorithm:
Cost difference: 4 (+2.0%)
Nodes in tree: 210 (53% of graph)
Preparation time: 104.818µs
Growth time: 464.326µs
Total time: 569.144µs

Quick estimate:
Cost difference: 11 (+5.4%)
Time: 25.561µs

# ShortestPath - gene61b.stp

Main algorithm:
Cost difference: 3 (+1.5%)
Nodes in tree: 203 (36% of graph)
Preparation time: 157.344µs
Growth time: 406.7µs
Total time: 564.044µs

Quick estimate:
Cost difference: 31 (+15.6%)
Time: 25.04µs

# ShortestPath - gene61c.stp

Main algorithm:
Cost difference: 5 (+2.6%)
Nodes in tree: 202 (37% of graph)
Preparation time: 146.532µs
Growth time: 443.053µs
Total time: 589.585µs

Quick estimate:
Cost difference: 29 (+14.8%)
Time: 22.084µs

# ShortestPath - gene61f.stp

Main algorithm:
Cost difference: 6 (+3.0%)
Nodes in tree: 205 (50% of graph)
Preparation time: 105.571µs
Growth time: 401.279µs
Total time: 506.85µs

Quick estimate:
Cost difference: 16 (+8.1%)
Time: 24.569µs

Summary for ShortestPath:
Datasets tested: 5
Average cost difference: 4.2 (+2.3%)
Total time: 2.544921ms
