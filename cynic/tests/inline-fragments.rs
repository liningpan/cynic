use {rstest::rstest, serde::Serialize, serde_json::json};

use cynic::{InlineFragments, QueryFragment};

#[derive(QueryFragment, Serialize)]
#[cynic(graphql_type = "Query", schema_path = "tests/test-schema.graphql")]
struct AllPostsQuery {
    all_data: Vec<PostOrAuthor>,
    #[arguments(id: "123")]
    node: Option<Node>,
}

#[derive(QueryFragment, Serialize, PartialEq, Debug)]
#[cynic(graphql_type = "BlogPost", schema_path = "tests/test-schema.graphql")]
struct Post {
    id: Option<cynic::Id>,
}

#[derive(QueryFragment, Serialize, PartialEq, Debug)]
#[cynic(graphql_type = "Author", schema_path = "tests/test-schema.graphql")]
struct Author {
    name: Option<String>,
}

#[derive(InlineFragments, Serialize, PartialEq, Debug)]
#[cynic(schema_path = "tests/test-schema.graphql")]
enum PostOrAuthor {
    Post(Post),
    Author(Author),
    #[cynic(fallback)]
    Other,
}

#[derive(InlineFragments, Serialize, PartialEq, Debug)]
#[cynic(schema_path = "tests/test-schema.graphql")]
enum Node {
    Post(Post),
    Author(Author),
    #[cynic(fallback)]
    Other,
}

#[derive(InlineFragments, Serialize, PartialEq, Debug)]
#[cynic(
    schema_path = "tests/test-schema.graphql",
    graphql_type = "PostOrAuthor",
    exhaustive
)]
enum ExhaustivePostOrAuthor {
    BlogPost(Post),
    Author(Author),
    #[cynic(fallback)]
    Other,
}

#[derive(InlineFragments, Serialize, PartialEq, Debug)]
#[cynic(
    schema_path = "tests/test-schema.graphql",
    graphql_type = "PostOrAuthor"
)]
enum PostOrAuthorGeneric<A: QueryFragment<SchemaType = schema::Author, VariablesFields = ()>> {
    Post(Post),
    Author(A),
    #[cynic(fallback)]
    Other,
}

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

#[test]
fn test_inline_fragment_query_output() {
    use cynic::QueryBuilder;

    let operation = AllPostsQuery::build(());

    insta::assert_display_snapshot!(operation.query, @r###"
    query AllPostsQuery {
      allData {
        __typename
        ... on BlogPost {
          id
        }
        ... on Author {
          name
        }
      }
      node(id: "123") {
        __typename
        ... on BlogPost {
          id
        }
        ... on Author {
          name
        }
      }
    }

    "###);
}

#[rstest]
#[case(json!({"__typename": "BlogPost", "id": null}), PostOrAuthor::Post(Post { id: None }))]
#[case(json!({"__typename": "Author", "name": null}), PostOrAuthor::Author(Author { name: None }))]
#[case(json!({"__typename": "SomeOtherThing"}), PostOrAuthor::Other)]
fn test_post_or_author_decoding(#[case] input: serde_json::Value, #[case] expected: PostOrAuthor) {
    assert_eq!(
        serde_json::from_value::<PostOrAuthor>(input).unwrap(),
        expected
    );
}

#[rstest]
#[case(json!({"__typename": "BlogPost", "id": null}), Node::Post(Post { id: None }))]
#[case(json!({"__typename": "Author", "name": null}), Node::Author(Author { name: None }))]
#[case(json!({"__typename": "SomeOtherThing"}), Node::Other)]
fn test_node_decoding(#[case] input: serde_json::Value, #[case] expected: Node) {
    assert_eq!(serde_json::from_value::<Node>(input).unwrap(), expected);
}

#[test]
fn test_decoding_fallback_with_extra_data_and_unit_fallback() {
    // This has a typename that doesn't exist _and_ some associated data.
    // Make sure we decode succesfully
    let data = r#"{"__typename":"Image","id":"4"}"#;

    assert_eq!(
        serde_json::from_str::<PostOrAuthor>(data).unwrap(),
        PostOrAuthor::Other
    );
}

#[derive(InlineFragments, Serialize, PartialEq, Debug)]
#[cynic(
    schema_path = "tests/test-schema.graphql",
    graphql_type = "PostOrAuthor"
)]
enum PostOrAuthorStringFallback {
    Post(Post),
    Author(Author),
    #[cynic(fallback)]
    Other(String),
}

#[test]
fn test_decoding_fallback_with_extra_data_and_string_fallback() {
    // This has a typename that doesn't exist _and_ some associated data.
    // Make sure we decode succesfully
    let data = r#"{"__typename":"Image","id":"4"}"#;

    assert_eq!(
        serde_json::from_str::<PostOrAuthorStringFallback>(data).unwrap(),
        PostOrAuthorStringFallback::Other("Image".into())
    );
}

#[derive(InlineFragments, Serialize, PartialEq, Debug)]
#[cynic(
    schema_path = "tests/test-schema.graphql",
    graphql_type = "PostOrAuthor"
)]
enum PostOrAuthorBox {
    Post(Box<Post>),
    Author(Author),
    #[cynic(fallback)]
    Other,
}

#[test]
fn test_decoding_boxed_variant() {
    let data = r#"{"__typename":"BlogPost","id":"4"}"#;

    assert_eq!(
        serde_json::from_str::<PostOrAuthorBox>(data).unwrap(),
        PostOrAuthorBox::Post(Box::new(Post {
            id: Some("4".into())
        }))
    );
}
