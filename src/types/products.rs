use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Products {
    pub id: ProductId,
    pub name: String,
    pub price: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductId(pub i32);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewProducts {
    pub name: String,
    pub price: i32,
}