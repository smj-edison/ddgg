use std::println;

use crate::graph::Graph;

#[test]
fn test_use_old_index() {
    let mut graph: Graph<(), ()> = Graph::new();

    let (first, _) = graph.add_vertex(()).unwrap();
    graph.remove_vertex(first).unwrap();

    let (second, _) = graph.add_vertex(()).unwrap();

    assert_eq!(first.0.index, second.0.index);
    assert_ne!(first.0.generation, second.0.generation);

    graph.get_vertex(second).unwrap();
    graph.get_vertex(first).unwrap_err();
}

#[test]
fn test_vertex_data() {
    let mut graph: Graph<i32, ()> = Graph::new();

    let (first, _) = graph.add_vertex(2).unwrap();
    let (second, _) = graph.add_vertex(4).unwrap();

    assert_eq!(graph.get_vertex(first).unwrap().data, 2);
    assert_eq!(graph.get_vertex(second).unwrap().data, 4);
}

#[test]
fn test_undo_redo() {
    let mut graph: Graph<i32, ()> = Graph::new();

    let (first, diff_1) = graph.add_vertex(2).unwrap();
    println!("{:?}", graph);

    let (_, diff_2) = graph.remove_vertex(first).unwrap();
    let (second, diff_3) = graph.add_vertex(4).unwrap();

    // test that using the diffs in the wrong order produce correct results
    graph.rollback_diff(diff_1.clone()).unwrap_err();
    graph.rollback_diff(diff_2.clone()).unwrap_err();

    // also, applying diffs in the wrong order should produce an error
    graph.apply_diff(diff_1.clone()).unwrap_err();
    graph.apply_diff(diff_2.clone()).unwrap_err();

    // now, let's try rolling back and reapplying
    graph.rollback_diff(diff_3.clone()).unwrap();
    graph.rollback_diff(diff_2.clone()).unwrap();
    assert_eq!(graph.get_vertex(first).unwrap().data, 2);
    graph.rollback_diff(diff_1.clone()).unwrap();

    graph.apply_diff(diff_1.clone()).unwrap();
    graph.apply_diff(diff_2.clone()).unwrap();
    graph.apply_diff(diff_3.clone()).unwrap();

    graph.get_vertex(first).unwrap_err();

    assert_eq!(graph.get_vertex(second).unwrap().data, 4);
}
