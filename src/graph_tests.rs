use std::println;

use crate::graph::Graph;

#[test]
fn test_use_old_index() {
    let mut graph: Graph<(), ()> = Graph::new();

    let (first, _) = graph.add_vertex(());
    graph.remove_vertex(first).unwrap();

    let (second, _) = graph.add_vertex(());

    assert_eq!(first.0.index, second.0.index);
    assert_ne!(first.0.generation, second.0.generation);

    graph.get_vertex(second).unwrap();
    assert!(matches!(graph.get_vertex(first), None));
}

#[test]
fn test_vertex_data() {
    let mut graph: Graph<i32, ()> = Graph::new();

    let (first, _) = graph.add_vertex(2);
    let (second, _) = graph.add_vertex(4);

    assert_eq!(*graph.get_vertex(first).unwrap().data(), 2);
    assert_eq!(*graph.get_vertex(second).unwrap().data(), 4);
}

#[test]
fn test_undo_redo() {
    let mut graph: Graph<i32, ()> = Graph::new();

    let (first, diff_1) = graph.add_vertex(2);
    println!("{:?}", graph);

    let (_, diff_2) = graph.remove_vertex(first).unwrap();
    let (second, diff_3) = graph.add_vertex(4);

    // test that using the diffs in the wrong order produce correct results
    graph.rollback_diff(diff_1.clone()).unwrap_err();
    graph.rollback_diff(diff_2.clone()).unwrap_err();

    // also, applying diffs in the wrong order should produce an error
    graph.apply_diff(diff_1.clone()).unwrap_err();
    graph.apply_diff(diff_2.clone()).unwrap_err();

    // now, let's try rolling back and reapplying
    graph.rollback_diff(diff_3.clone()).unwrap();
    graph.rollback_diff(diff_2.clone()).unwrap();
    assert_eq!(*graph.get_vertex(first).unwrap().data(), 2);
    graph.rollback_diff(diff_1.clone()).unwrap();

    graph.apply_diff(diff_1.clone()).unwrap();
    graph.apply_diff(diff_2.clone()).unwrap();
    graph.apply_diff(diff_3.clone()).unwrap();

    assert!(matches!(graph.get_vertex(first), None));

    assert_eq!(*graph.get_vertex(second).unwrap().data(), 4);
}

#[test]
fn test_undo_redo_edges() {
    let mut graph: Graph<String, String> = Graph::new();

    let (first_vertex, diff_1) = graph.add_vertex("first_vertex".into());
    let (second_vertex, diff_2) = graph.add_vertex("second_vertex".into());
    let (third_vertex, diff_3) = graph.add_vertex("third_vertex".into());

    let (first_edge, diff_4) = graph
        .add_edge(first_vertex, second_vertex, "first_edge".into())
        .unwrap();
    let (second_edge, diff_5) = graph
        .add_edge(second_vertex, third_vertex, "second_edge".into())
        .unwrap();

    let (_, diff_6) = graph.remove_vertex(third_vertex).unwrap();
    assert!(matches!(graph.get_edge(second_edge), None));

    let (_, diff_7) = graph.remove_edge(first_edge).unwrap();
    assert!(matches!(graph.get_edge(first_edge), None));

    graph.rollback_diff(diff_7.clone()).unwrap();
    graph.get_edge(first_edge).unwrap();
    graph.rollback_diff(diff_6.clone()).unwrap();
    graph.get_edge(second_edge).unwrap();
    graph.rollback_diff(diff_5.clone()).unwrap();
    graph.rollback_diff(diff_4.clone()).unwrap();
    graph.rollback_diff(diff_3.clone()).unwrap();
    graph.rollback_diff(diff_2.clone()).unwrap();
    graph.rollback_diff(diff_1.clone()).unwrap();

    graph.apply_diff(diff_1.clone()).unwrap();
    graph.apply_diff(diff_2.clone()).unwrap();
    graph.apply_diff(diff_3.clone()).unwrap();
    graph.apply_diff(diff_4.clone()).unwrap();
    graph.apply_diff(diff_5.clone()).unwrap();
    graph.get_edge(second_edge).unwrap();
    graph.apply_diff(diff_6.clone()).unwrap();

    assert!(matches!(graph.get_edge(second_edge), None));
    graph.get_edge(first_edge).unwrap();
    graph.apply_diff(diff_7.clone()).unwrap();
    assert!(matches!(graph.get_edge(first_edge), None));
}

#[test]
fn test_modify_data() {
    let mut graph: Graph<String, String> = Graph::new();

    let (first_vertex, _diff_1) = graph.add_vertex("first_vertex".into());
    let (second_vertex, _diff_2) = graph.add_vertex("second_vertex".into());

    let (first_edge, _diff_3) = graph
        .add_edge(first_vertex, second_vertex, "first_edge".into())
        .unwrap();
    let (_, diff_4) = graph
        .update_vertex(first_vertex, "first_vertex_modified".into())
        .unwrap();
    let (_, diff_5) = graph
        .update_edge(first_edge, "first_edge_modified".into())
        .unwrap();

    assert_eq!(
        *graph.get_vertex(first_vertex).unwrap().data(),
        "first_vertex_modified".to_string()
    );
    assert_eq!(
        *graph.get_edge(first_edge).unwrap().data(),
        "first_edge_modified".to_string()
    );

    graph.rollback_diff(diff_5.clone()).unwrap();
    graph.rollback_diff(diff_4.clone()).unwrap();
    assert_eq!(
        *graph.get_vertex(first_vertex).unwrap().data(),
        "first_vertex".to_string()
    );
    assert_eq!(
        *graph.get_edge(first_edge).unwrap().data(),
        "first_edge".to_string()
    );

    graph.apply_diff(diff_4.clone()).unwrap();
    graph.apply_diff(diff_5.clone()).unwrap();
    assert_eq!(
        *graph.get_vertex(first_vertex).unwrap().data(),
        "first_vertex_modified".to_string()
    );

    assert_eq!(
        *graph.get_edge(first_edge).unwrap().data(),
        "first_edge_modified".to_string()
    );
}

#[test]
#[cfg(feature = "serde_string_indexes")]
fn test_custom_serde() {
    use crate::VertexIndex;

    let mut graph: Graph<(), ()> = Graph::new();

    let (index, _) = graph.add_vertex(()).unwrap();

    assert_eq!(serde_json::to_string(&index).unwrap(), r#""0.0""#);
    assert_eq!(
        serde_json::from_str::<VertexIndex>(&r#""0.0""#).unwrap(),
        index
    );
}
