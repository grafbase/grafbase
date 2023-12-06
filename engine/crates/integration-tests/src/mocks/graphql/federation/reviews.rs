// See https://github.com/async-graphql/examples
use async_graphql::{ComplexObject, Context, EmptyMutation, EmptySubscription, Enum, Object, Schema, SimpleObject, ID};

pub struct FakeFederationReviewsSchema;

impl FakeFederationReviewsSchema {
    fn schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
        let reviews = vec![
            Review {
                id: "review-1".into(),
                body: "A highly effective form of birth control.".into(),
                pictures: vec![
                    Picture {
                        url: "http://localhost:8080/ugly_hat.jpg".to_string(),
                        width: 100,
                        height: 100,
                        alt_text: "A Trilby".to_string(),
                    },
                    Picture {
                        url: "http://localhost:8080/troll_face.jpg".to_string(),
                        width: 42,
                        height: 42,
                        alt_text: "The troll face meme".to_string(),
                    },
                ],
            },
            Review {
                id: "review-2".into(),
                body:
                    "Fedoras are one of the most fashionable hats around and can look great with a variety of outfits."
                        .into(),
                pictures: vec![],
            },
            Review {
                id: "review-3".into(),
                body: "This is the last straw. Hat you will wear. 11/10".into(),
                pictures: vec![],
            },
        ];

        Schema::build(Query, EmptyMutation, EmptySubscription)
            .data(reviews)
            .enable_federation()
            .finish()
    }
}

#[async_trait::async_trait]
impl super::super::Schema for FakeFederationReviewsSchema {
    async fn execute(
        &self,
        _headers: Vec<(String, String)>,
        request: async_graphql::Request,
    ) -> async_graphql::Response {
        Self::schema().execute(request).await
    }

    fn sdl(&self) -> String {
        Self::schema().sdl_with_options(async_graphql::SDLExportOptions::new().federation())
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct User {
    id: ID,
    #[graphql(override_from = "accounts")]
    review_count: u32,
    #[graphql(external)]
    joined_timestamp: u64,
}

#[derive(Enum, Eq, PartialEq, Copy, Clone)]
#[allow(clippy::enum_variant_names)]
enum Trustworthiness {
    ReallyTrusted,
    KindaTrusted,
    NotTrusted,
}

#[ComplexObject]
impl User {
    async fn reviews<'a>(&self, ctx: &'a Context<'_>) -> Vec<&'a Review> {
        let reviews = ctx.data_unchecked::<Vec<Review>>();
        reviews
            .iter()
            .filter(|review| review.get_author().id == self.id)
            .collect()
    }

    #[graphql(requires = "joinedTimestamp")]
    async fn trustworthiness(&self) -> Trustworthiness {
        if self.joined_timestamp < 1_000 && self.review_count > 1 {
            Trustworthiness::ReallyTrusted
        } else if self.joined_timestamp < 2_000 {
            Trustworthiness::KindaTrusted
        } else {
            Trustworthiness::NotTrusted
        }
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct Product {
    upc: String,
    #[graphql(external)]
    price: u32,
}

#[ComplexObject]
impl Product {
    async fn reviews<'a>(&self, ctx: &'a Context<'_>) -> Vec<&'a Review> {
        let reviews = ctx.data_unchecked::<Vec<Review>>();
        reviews
            .iter()
            .filter(|review| review.get_product().upc == self.upc)
            .collect()
    }
}

#[derive(SimpleObject)]
#[graphql(complex)]
struct Review {
    id: ID,
    body: String,
    pictures: Vec<Picture>,
}

#[ComplexObject]
impl Review {
    #[graphql(provides = "price")]
    async fn product<'a>(&self) -> Product {
        self.get_product()
    }

    async fn author(&self) -> User {
        self.get_author()
    }
}

impl Review {
    fn get_product(&self) -> Product {
        match self.id.as_str() {
            "review-1" => Product {
                upc: "top-1".to_string(),
                price: 10,
            },
            "review-2" => Product {
                upc: "top-2".to_string(),
                price: 20,
            },
            "review-3" => Product {
                upc: "top-3".to_string(),
                price: 30,
            },
            _ => panic!("Unknown review id"),
        }
    }

    fn get_author(&self) -> User {
        let user_id: ID = match self.id.as_str() {
            "review-1" | "review-2" => "1234",
            "review-3" => "7777",
            _ => panic!("Unknown review id"),
        }
        .into();
        user_by_id(user_id, None)
    }
}

#[derive(SimpleObject)]
#[graphql(shareable)]
struct Picture {
    url: String,
    width: u32,
    height: u32,
    #[graphql(inaccessible)] // Field not added to Accounts yet
    alt_text: String,
}

struct Query;

#[Object]
impl Query {
    #[graphql(entity)]
    async fn find_user_by_id(&self, #[graphql(key)] id: ID, joined_timestamp: Option<u64>) -> User {
        user_by_id(id, joined_timestamp)
    }

    #[graphql(entity)]
    async fn find_product_by_upc(&self, upc: String) -> Product {
        Product { upc, price: 0 }
    }
}

fn user_by_id(id: ID, joined_timestamp: Option<u64>) -> User {
    let review_count = match id.as_str() {
        "1234" => 2,
        "7777" => 1,
        _ => 0,
    };
    // This will be set if the user requested the fields that require it.
    let joined_timestamp = joined_timestamp.unwrap_or(9001);
    User {
        id,
        review_count,
        joined_timestamp,
    }
}
