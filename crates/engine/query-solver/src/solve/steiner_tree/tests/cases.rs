use pretty_assertions::assert_eq;

use super::*;
use crate::solve::steiner_tree::{GreedyFlac, SteinerTree};

#[test]
fn step_by_step() {
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> a;
            a -> t1 [label=1];
            a -> t2 [label=2];
        }
        "#,
    );

    let outcome = runner.run_once();
    assert!(outcome.is_continue(), "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      a -> t1
      root -> a
    }
    ");

    let outcome = runner.run_once();
    assert!(outcome.is_break(), "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      a -> t1
      a -> t2
      root -> a
    }
    ");
}

#[test]
fn single_terminal_direct_path() {
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> t1 [label=5];
        }
        "#,
    );

    let total_weight = runner.run();
    assert_eq!(total_weight, 5, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      root -> t1
    }
    ");
}

#[test]
fn multiple_terminals_shared_edges() {
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> shared [label=10];
            shared -> t1 [label=3];
            shared -> t2 [label=5];
            shared -> t3 [label=2];
        }
        "#,
    );

    let total_weight = runner.run();
    assert_eq!(
        total_weight,
        20, // 10 + 3 + 5 + 2
        "\n{}",
        runner.debug_graph()
    );
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      root -> shared
      shared -> t1
      shared -> t2
      shared -> t3
    }
    ");
}

#[test]
fn degenerate_flow_detection() {
    // Graph where t1 can reach a through two paths, which would create degenerate flow
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> a [label=10];
            a -> b [label=2];
            a -> c [label=3];
            b -> t1 [label=1];
            c -> t1 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    assert_eq!(
        total_weight,
        13, // Should pick one path: root -> a -> b -> t1 (10 + 2 + 1)
        "\n{}",
        runner.debug_graph()
    );
    // The algorithm should mark one edge to avoid degenerate flow
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      a -> b
      b -> t1
      root -> a
    }
    ");
}

#[test]
fn complex_graph_multiple_paths() {
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> a [label=5];
            root -> b [label=8];
            a -> c [label=3];
            b -> c [label=2];
            c -> t1 [label=4];
            c -> t2 [label=6];
            a -> t2 [label=10];
            b -> t3 [label=7];
        }
        "#,
    );

    let total_weight = runner.run();
    assert_eq!(
        total_weight,
        // root -> a -> c -> t1,t2: 5 + 3 + 6 + 4 = 18
        // root -> b -> t3: 8 + 7 = 15
        33,
        "\n{}",
        runner.debug_graph()
    );
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      a -> c
      b -> t3
      c -> t1
      c -> t2
      root -> a
      root -> b
    }
    ");
}

#[test]
fn incremental_terminal_addition() {
    // Start with only t1 terminal by creating custom runner
    let (graph, nodes) = dot_graph(
        r#"
        digraph {
            root -> a [label=4];
            root -> b [label=6];
            a -> t1 [label=2];
            b -> t2 [label=3];
            a -> t3 [label=5];
        }
        "#,
    );
    let steiner_tree = SteinerTree::new(&graph, nodes["root"], vec![nodes["t1"]]);
    let greedy_flac = GreedyFlac::new(&graph);
    let mut runner = Runner {
        graph,
        nodes,
        greedy_flac,
        steiner_tree,
    };

    // First run with only t1
    let total_weight = runner.run();
    assert_eq!(
        total_weight,
        6, // root -> a -> t1: 4 + 2
        "\n{}",
        runner.debug_graph()
    );

    // Add t2 as a new terminal
    runner.extend_terminals(&["t2"]);
    let total_weight = runner.run();
    assert_eq!(
        total_weight,
        6 + 9, // root -> b -> t2: 6 + 3
        "\n{}",
        runner.debug_graph()
    );

    // Add t3 as another terminal
    runner.extend_terminals(&["t3"]);
    let total_weight = runner.run();
    assert_eq!(
        total_weight,
        6 + 9 + 5, // a -> t3: 5 (root -> a already in tree)
        "\n{}",
        runner.debug_graph()
    );

    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      a -> t1
      a -> t3
      b -> t2
      root -> a
      root -> b
    }
    ");
}

#[test]
fn weighted_edges_different_weights() {
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> a [label=1];
            root -> b [label=100];
            a -> c [label=50];
            b -> c [label=1];
            c -> t1 [label=1];
            a -> t2 [label=2];
            b -> t3 [label=2];
        }
        "#,
    );

    let total_weight = runner.run();
    assert_eq!(
        total_weight,
        // Optimal: root->a->t2 (3), root->b->c->t1 (102), root->b->t3 (102 already counted + 2 = 2)
        // But GreedyFLAC doesn't take the optimal path.
        156,
        "\n{}",
        runner.debug_graph()
    );

    // The algorithm should prefer the cheaper paths
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      a -> c
      a -> t2
      b -> t3
      c -> t1
      root -> a
      root -> b
    }
    ");
}

#[test]
fn diamond_shaped_graph() {
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> top [label=5];
            top -> left [label=3];
            top -> right [label=4];
            left -> bottom [label=6];
            right -> bottom [label=2];
            bottom -> t1 [label=1];
            left -> t2 [label=8];
            right -> t3 [label=7];
        }
        "#,
    );

    let total_weight = runner.run();
    assert_eq!(
        total_weight,
        30, // root->top->left->t2 (5+3+8=16) + top->right->bottom->t1 (4+2+1=7) + right->t3 (7-already have right)
        "\n{}",
        runner.debug_graph()
    );

    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      bottom -> t1
      left -> t2
      right -> bottom
      right -> t3
      root -> top
      top -> left
      top -> right
    }
    ");
}

#[test]
fn linear_chain_graph() {
    let mut runner = Runner::from_dot_graph(
        r#"
    digraph {
        root -> n1 [label=2];
        n1 -> n2 [label=3];
        n2 -> n3 [label=4];
        n3 -> n4 [label=5];
        n4 -> t1 [label=6];
        n2 -> t2 [label=10];
        n3 -> t3 [label=8];
    }
    "#,
    );

    let total_weight = runner.run();
    assert_eq!(
        total_weight,
        38, // root->n1->n2->n3->n4->t1 (2+3+4+5+6=20) + n2->t2 (10) + n3->t3 (8)
        "\n{}",
        runner.debug_graph()
    );

    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      n1 -> n2
      n2 -> n3
      n2 -> t2
      n3 -> n4
      n3 -> t3
      n4 -> t1
      root -> n1
    }
    ");
}

#[test]
fn properly_decrease_saturating_time() {
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            a -> t1;
            root -> a [label=10];
            b -> t1;
            root -> b [label=10];
            c -> t1;
            root -> c [label=10];
            c -> t2 [label=10];
            b -> t2 [label=10];
            a -> t2 [label=10];
            d -> t3;
            c -> d [label=10];
            a -> d [label=10];
            b -> t3;
        }
        "#,
    );

    let total_weight = runner.run();
    assert_eq!(
        total_weight,
        20, // root -> b (10) + b -> t2 (10)
        "\n{}",
        runner.debug_graph()
    );

    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      b -> t1
      b -> t2
      b -> t3
      root -> b
    }
    ");
}

#[test]
fn star_graph_greedy_trap() {
    // Star graph where greedy picks cheap initial edges but leads to expensive total
    // Optimal: root -> hub -> all terminals (100 + 4*10 = 140)
    // GreedyFLAC might pick: root -> t1 (1) then forced to root -> hub -> t2,t3,t4 (100 + 3*10 = 130)
    // Total: 131 vs optimal 140 - but in this case greedy does better!
    // Let's create a case where greedy is worse
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> cheap_path [label=1];
            cheap_path -> t1 [label=1];
            root -> expensive_hub [label=50];
            expensive_hub -> t1 [label=1];
            expensive_hub -> t2 [label=1];
            expensive_hub -> t3 [label=1];
            expensive_hub -> t4 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // GreedyFLAC will pick cheap_path first (saturates at time 2)
    // Then needs to add expensive_hub for other terminals
    // Total: 1 + 1 + 50 + 1 + 1 + 1 = 55
    // Optimal would be: 50 + 1 + 1 + 1 + 1 = 54
    // This demonstrates a 1-unit suboptimality
    assert_eq!(
        total_weight,
        55, // Suboptimal by 1 unit
        "\n{}",
        runner.debug_graph()
    );

    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      cheap_path -> t1
      expensive_hub -> t2
      expensive_hub -> t3
      expensive_hub -> t4
      root -> cheap_path
      root -> expensive_hub
    }
    ");
}

#[test]
fn asymmetric_multi_level() {
    // Multi-level graph where greedy choices at each level compound suboptimality
    // This tests how local greedy decisions accumulate into global suboptimality
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> level1_cheap [label=1];
            root -> level1_expensive [label=20];
            level1_cheap -> level2_trap [label=50];
            level1_expensive -> level2_good [label=2];
            level2_trap -> t1 [label=1];
            level2_trap -> t2 [label=1];
            level2_good -> t1 [label=1];
            level2_good -> t2 [label=1];
            level2_good -> t3 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // GreedyFLAC might pick level1_cheap first (saturates early)
    // Then forced to use level2_trap, and later add expensive path for t3
    // But in this case, the algorithm is smart enough to find optimal: 20 + 2 + 1 + 1 + 1 = 25
    // This shows the algorithm can sometimes avoid the greedy trap
    assert_eq!(total_weight, 25, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      level1_expensive -> level2_good
      level2_good -> t1
      level2_good -> t2
      level2_good -> t3
      root -> level1_expensive
    }
    ");
}

#[test]
fn high_cost_shared_edge() {
    // Graph where a high-cost shared edge seems attractive but should be avoided
    // Tests if algorithm properly evaluates the true cost of shared edges
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> shared_expensive [label=100];
            shared_expensive -> t1 [label=1];
            shared_expensive -> t2 [label=1];
            shared_expensive -> t3 [label=1];
            root -> path1 [label=35];
            path1 -> t1 [label=1];
            root -> path2 [label=35];
            path2 -> t2 [label=1];
            root -> path3 [label=35];
            path3 -> t3 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // Optimal using shared: 100 + 1 + 1 + 1 = 103
    // Optimal using separate: 35 + 1 + 35 + 1 + 35 + 1 = 108
    // GreedyFLAC should pick the shared edge
    assert_eq!(total_weight, 103, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      root -> shared_expensive
      shared_expensive -> t1
      shared_expensive -> t2
      shared_expensive -> t3
    }
    ");
}

#[test]
fn longer_cheaper_paths() {
    // Graph where optimal solution uses longer paths with more hops but lower total cost
    // Tests if greedy algorithm misses these due to focusing on immediate saturation
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> direct_expensive [label=100];
            direct_expensive -> t1 [label=1];
            root -> hop1 [label=10];
            hop1 -> hop2 [label=10];
            hop2 -> hop3 [label=10];
            hop3 -> t1 [label=1];
            hop3 -> t2 [label=1];
            direct_expensive -> t2 [label=50];
        }
        "#,
    );

    let total_weight = runner.run();
    // Direct path: 100 + 1 + 50 = 151
    // Longer path: 10 + 10 + 10 + 1 + 1 = 32
    // GreedyFLAC should eventually find the longer cheaper path
    assert_eq!(total_weight, 32, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      hop1 -> hop2
      hop2 -> hop3
      hop3 -> t1
      hop3 -> t2
      root -> hop1
    }
    ");
}

#[test]
fn unoptimal_equal_initial_costs_different_outcomes() {
    // Graph with multiple equally cheap initial paths that lead to vastly different total costs
    // Tests tie-breaking behavior and its impact on final solution quality
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> option_a [label=10];
            root -> option_b [label=10];
            option_a -> hub_a [label=1];
            option_b -> hub_b [label=50];
            hub_a -> t1 [label=50];
            hub_a -> t2 [label=50];
            hub_b -> t1 [label=1];
            hub_b -> t2 [label=1];
            hub_b -> t3 [label=1];
            option_a -> t3 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // Path A focused: 10 + 1 + 50 + 50 + 1 = 112 (for t1, t2, t3)
    // Path B focused: 10 + 50 + 1 + 1 + 1 = 63 (for t1, t2, t3)
    // Mixed paths possible, algorithm behavior depends on flow dynamics
    assert_eq!(total_weight, 73, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      hub_b -> t1
      hub_b -> t2
      option_a -> t3
      option_b -> hub_b
      root -> option_a
      root -> option_b
    }
    ");
}

#[test]
fn cascading_suboptimal_choices() {
    // Complex graph where early suboptimal choices cascade into worse decisions
    // Each greedy choice constrains future options, compounding suboptimality
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> early_trap [label=1];
            early_trap -> connector1 [label=20];
            connector1 -> t1 [label=1];
            connector1 -> expensive_bridge [label=30];
            expensive_bridge -> t2 [label=1];

            root -> late_optimal [label=15];
            late_optimal -> connector2 [label=2];
            connector2 -> t1 [label=1];
            connector2 -> t2 [label=1];
            connector2 -> t3 [label=1];

            early_trap -> t3 [label=60];
        }
        "#,
    );

    let total_weight = runner.run();
    // Greedy trap: 1 + 20 + 1 + 30 + 1 + 60 = 113
    // Optimal: 15 + 2 + 1 + 1 + 1 = 20
    // The algorithm should find something between these
    assert_eq!(total_weight, 20, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      connector2 -> t1
      connector2 -> t2
      connector2 -> t3
      late_optimal -> connector2
      root -> late_optimal
    }
    ");
}

#[test]
fn unoptimal_deceptive_density() {
    // Graph where initial density (cost per terminal) is deceptive
    // Low initial density leads to high final cost
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> cheap_dense [label=2];
            cheap_dense -> t1 [label=1];
            cheap_dense -> t2 [label=1];
            cheap_dense -> dead_end [label=100];

            root -> expensive_sparse [label=30];
            expensive_sparse -> t1 [label=1];
            expensive_sparse -> good_path [label=1];
            good_path -> t2 [label=1];
            good_path -> t3 [label=1];
            good_path -> t4 [label=1];

            dead_end -> t3 [label=1];
            dead_end -> t4 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // Greedy might pick cheap_dense first (density 2 for 2 terminals)
    // Then forced to pay 100 to reach t3, t4: 2 + 1 + 1 + 100 + 1 + 1 = 106
    // Optimal: 30 + 1 + 1 + 1 + 1 + 1 = 35
    assert_eq!(total_weight, 37, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      cheap_dense -> t1
      cheap_dense -> t2
      expensive_sparse -> good_path
      good_path -> t3
      good_path -> t4
      root -> cheap_dense
      root -> expensive_sparse
    }
    ");
}

#[test]
fn zero_weight_edges_trap() {
    // Zero-weight edges can mislead the algorithm by saturating instantly
    // They seem "free" but can lead to expensive paths being locked in
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> trap_path [label=0];
            trap_path -> expensive_connector [label=0];
            expensive_connector -> t1 [label=100];
            expensive_connector -> t2 [label=100];
 
            root -> good_path [label=30];
            good_path -> t1 [label=1];
            good_path -> t2 [label=1];
            good_path -> t3 [label=1];

            trap_path -> t3 [label=200];
        }
        "#,
    );

    let total_weight = runner.run();
    // Zero-weight edges saturate instantly, potentially locking in expensive paths
    // Trap path: 0 + 0 + 100 + 100 + 200 = 400
    // Good path: 30 + 1 + 1 + 1 = 33
    assert_eq!(total_weight, 33, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      good_path -> t1
      good_path -> t2
      good_path -> t3
      root -> good_path
    }
    ");
}

#[test]
fn terminal_ordering_sensitivity() {
    // The order in which terminals are processed can affect the outcome
    // This tests whether the algorithm is sensitive to terminal ordering
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> path_a [label=10];
            root -> path_b [label=10];
            path_a -> t1 [label=1];
            path_a -> connector_a [label=40];
            connector_a -> t2 [label=1];

            path_b -> t2 [label=1];
            path_b -> connector_b [label=40];
            connector_b -> t1 [label=1];

            path_a -> t3 [label=5];
            path_b -> t3 [label=5];
        }
        "#,
    );

    let total_weight = runner.run();
    // Depending on which terminal saturates first, different paths are chosen
    // Best: 10 + 1 + 10 + 1 + 5 = 27 (use both paths for t1, t2, share one for t3)
    // Worst: 10 + 1 + 40 + 1 + 5 = 57 (commit to one path early)
    assert_eq!(total_weight, 27, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      path_a -> t1
      path_b -> t2
      path_b -> t3
      root -> path_a
      root -> path_b
    }
    ");
}

#[test]
fn large_approximation_ratio() {
    // Demonstrates a case approaching the theoretical k-approximation bound
    // where k is the number of terminals
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> hub [label=1000];
            hub -> t1 [label=1];
            hub -> t2 [label=1];
            hub -> t3 [label=1];
            hub -> t4 [label=1];

            root -> direct1 [label=100];
            direct1 -> t1 [label=1];

            root -> direct2 [label=100];
            direct2 -> t2 [label=1];

            root -> direct3 [label=100];
            direct3 -> t3 [label=1];

            root -> direct4 [label=100];
            direct4 -> t4 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // Optimal: hub solution = 1000 + 4 = 1004
    // Suboptimal: individual paths = 4 * 101 = 404
    // The algorithm should find the better solution (404)
    assert_eq!(total_weight, 404, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      direct1 -> t1
      direct2 -> t2
      direct3 -> t3
      direct4 -> t4
      root -> direct1
      root -> direct2
      root -> direct3
      root -> direct4
    }
    ");
}

#[test]
fn bottleneck_graph() {
    // Graph with a bottleneck that all paths must go through
    // Tests how the algorithm handles forced convergence points
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> bottleneck [label=50];
            bottleneck -> branch1 [label=10];
            bottleneck -> branch2 [label=10];
            branch1 -> t1 [label=1];
            branch1 -> t2 [label=1];
            branch2 -> t3 [label=1];
            branch2 -> t4 [label=1];

            root -> expensive_bypass1 [label=60];
            expensive_bypass1 -> t1 [label=1];
            root -> expensive_bypass2 [label=60];
            expensive_bypass2 -> t2 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // Through bottleneck: 50 + 10 + 10 + 1 + 1 + 1 + 1 = 74
    // Bypassing for some: varies based on choices
    assert_eq!(total_weight, 74, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      bottleneck -> branch1
      bottleneck -> branch2
      branch1 -> t1
      branch1 -> t2
      branch2 -> t3
      branch2 -> t4
      root -> bottleneck
    }
    ");
}

#[test]
fn overlapping_paths_interdependencies() {
    // Complex overlapping paths where choosing one affects the cost of others
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> shared1 [label=20];
            root -> shared2 [label=20];
 
            shared1 -> middle [label=15];
            shared2 -> middle [label=15];

            middle -> t1 [label=5];
            middle -> t2 [label=5];

            shared1 -> direct_t1 [label=30];
            direct_t1 -> t1 [label=1];

            shared2 -> direct_t2 [label=30];
            direct_t2 -> t2 [label=1];

            root -> independent [label=35];
            independent -> t1 [label=1];
            independent -> t2 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // Multiple valid solutions with different trade-offs
    // Via middle: 20 + 15 + 5 + 5 = 45
    // Independent: 35 + 1 + 1 = 37
    // Mixed strategies possible
    assert_eq!(total_weight, 37, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      independent -> t1
      independent -> t2
      root -> independent
    }
    ");
}

#[test]
fn late_arriving_better_options() {
    // Graph where better options only become apparent late in the flow
    // Early saturation can lock out these better late-arriving options
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> early_cheap [label=1];
            early_cheap -> slow_path1 [label=20];
            slow_path1 -> slow_path2 [label=20];
            slow_path2 -> slow_path3 [label=20];
            slow_path3 -> t1 [label=1];
            slow_path3 -> t2 [label=1];

            root -> late_expensive [label=40];
            late_expensive -> t1 [label=1];
            late_expensive -> t2 [label=1];
            late_expensive -> t3 [label=1];
 
            early_cheap -> t3 [label=100];
        }
        "#,
    );

    let total_weight = runner.run();
    // Early cheap path: 1 + 20 + 20 + 20 + 1 + 1 + 100 = 163
    // Late expensive path: 40 + 1 + 1 + 1 = 43
    // The algorithm should eventually find the better path
    assert_eq!(total_weight, 43, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      late_expensive -> t1
      late_expensive -> t2
      late_expensive -> t3
      root -> late_expensive
    }
    ");
}

#[test]
fn unoptimal_multiple_terminals_different_depths() {
    // Terminals at different depths in the graph can cause suboptimal choices
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> shallow [label=5];
            shallow -> t1 [label=1];
 
            root -> medium [label=10];
            medium -> level2 [label=10];
            level2 -> t2 [label=1];
 
            root -> deep [label=15];
            deep -> deep2 [label=15];
            deep2 -> deep3 [label=15];
            deep3 -> t3 [label=1];
 
            shallow -> cross_link [label=50];
            cross_link -> t2 [label=1];
            cross_link -> t3 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // Optimal separate paths: 5 + 1 + 10 + 10 + 1 + 15 + 15 + 15 + 1 = 73
    // Using cross_link: 5 + 1 + 50 + 1 + 1 = 58
    // The algorithm should find a reasonable solution
    assert_eq!(total_weight, 73, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      deep -> deep2
      deep2 -> deep3
      deep3 -> t3
      level2 -> t2
      medium -> level2
      root -> deep
      root -> medium
      root -> shallow
      shallow -> t1
    }
    ");
}

#[test]
fn flow_rate_confusion() {
    // Complex flow rates can confuse the algorithm's saturation calculations
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> split1 [label=30];
            root -> split2 [label=30];
 
            split1 -> merge [label=20];
            split2 -> merge [label=20];
 
            merge -> t1 [label=10];
            merge -> t2 [label=10];
 
            split1 -> t3 [label=15];
            split2 -> t4 [label=15];
 
            root -> direct [label=60];
            direct -> t1 [label=1];
            direct -> t2 [label=1];
            direct -> t3 [label=1];
            direct -> t4 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // Direct path: 60 + 1 + 1 + 1 + 1 = 64
    // Split paths: complex depending on flow dynamics
    assert_eq!(total_weight, 64, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      direct -> t1
      direct -> t2
      direct -> t3
      direct -> t4
      root -> direct
    }
    ");
}

#[test]
fn parallel_asymmetric_branches() {
    // Parallel branches with asymmetric costs and terminal distributions
    let mut runner = Runner::from_dot_graph(
        r#"
        digraph {
            root -> branch_a [label=5];
            root -> branch_b [label=50];

            branch_a -> a_split1 [label=10];
            branch_a -> a_split2 [label=10];
            a_split1 -> t1 [label=1];
            a_split2 -> t2 [label=1];

            branch_b -> b_split1 [label=1];
            branch_b -> b_split2 [label=1];
            b_split1 -> t3 [label=1];
            b_split1 -> t4 [label=1];
            b_split2 -> t5 [label=1];
            b_split2 -> t6 [label=1];

            a_split1 -> expensive_cross [label=100];
            expensive_cross -> t3 [label=1];
            expensive_cross -> t4 [label=1];
            expensive_cross -> t5 [label=1];
            expensive_cross -> t6 [label=1];
        }
        "#,
    );

    let total_weight = runner.run();
    // Branch A for t1,t2: 5 + 10 + 10 + 1 + 1 = 27
    // Branch B for t3,t4,t5,t6: 50 + 1 + 1 + 1 + 1 + 1 + 1 = 56
    // Total optimal: 83
    // Using cross link would be much worse
    assert_eq!(total_weight, 83, "\n{}", runner.debug_graph());
    insta::assert_snapshot!(runner.steiner_graph(), @r"
    digraph {
      a_split1 -> t1
      a_split2 -> t2
      b_split1 -> t3
      b_split1 -> t4
      b_split2 -> t5
      b_split2 -> t6
      branch_a -> a_split1
      branch_a -> a_split2
      branch_b -> b_split1
      branch_b -> b_split2
      root -> branch_a
      root -> branch_b
    }
    ");
}
