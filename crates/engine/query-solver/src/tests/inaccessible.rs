use crate::assert_solving_snapshots;

const SCHEMA: &str = r#"
enum join__Graph {
    A @join__graph(name: "a", url: "http://localhost:4200/union-intersection/a")
    B @join__graph(name: "b", url: "http://localhost:4200/union-intersection/b")
}

type Book
    @join__type(graph: A)
    @join__type(graph: B)
{
    title: String!
}

type Movie
    @join__type(graph: B)
{
    title: String!
}

type Query
    @join__type(graph: A)
    @join__type(graph: B)
{
    media: Media
    book: Media @join__field(graph: A, type: "Book") @join__field(graph: B, type: "Media")
    song: Media @join__field(graph: A)
    viewer: Viewer
}

type Song
    @join__type(graph: A)
{
    title: String!
}

type Viewer
    @join__type(graph: A)
    @join__type(graph: B)
{
    media: ViewerMedia
    book: ViewerMedia @join__field(graph: A, type: "Book") @join__field(graph: B, type: "ViewerMedia")
    song: ViewerMedia @join__field(graph: A)
}

union Media
    @join__type(graph: A)
    @join__type(graph: B)
    @join__unionMember(graph: A, member: "Book")
    @join__unionMember(graph: B, member: "Book")
    @join__unionMember(graph: A, member: "Song")
    @join__unionMember(graph: B, member: "Movie")
 = Book | Song | Movie

union ViewerMedia
    @join__type(graph: A)
    @join__type(graph: B)
    @join__unionMember(graph: A, member: "Book")
    @join__unionMember(graph: B, member: "Book")
    @join__unionMember(graph: A, member: "Song")
    @join__unionMember(graph: B, member: "Movie")
 = Book | Song | Movie
"#;

#[test]
fn inaccessible_field_for_one_subgraph() {
    // title is available in subgraph B not in A. Query.media is accessible in both.
    assert_solving_snapshots!(
        "inaccessible_field_for_one_subgraph",
        SCHEMA,
        r#"
        query {
          media {
            ... on Movie {
              title
            }
          }
        }
        "#
    );
}
