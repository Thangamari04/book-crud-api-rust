use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use sqlx::{PgPool, postgres::PgPoolOptions};
use dotenv::dotenv;
use sqlx::Row;
use serde::{Serialize, Deserialize}; // Import Deserialize trait

// Define a struct to represent a book
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)] // Implement Deserialize trait
struct Book {
    id: i32,
    title: String,
    author: String,
    price: i32,
    pages: i32,
    is_published: bool,
}

// Struct to represent the data for adding a new book
#[derive(Debug, Deserialize)] // Implement Deserialize trait
struct NewBook {
    title: String,
    author: String,
    price: i32,
    pages: i32,
    is_published: bool,
}

// Handler to fetch all books
async fn get_books(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, Book>("SELECT * FROM books")
        .fetch_all(&**pool)
        .await;

    match result {
        Ok(books) => HttpResponse::Ok().json(books),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Handler to add a new book
async fn add_book(new_book: web::Json<NewBook>, pool: web::Data<PgPool>) -> impl Responder {
    match sqlx::query(
        r#"INSERT INTO books (title, author, price, pages, is_published)
        VALUES ($1, $2, $3, $4, $5) RETURNING id, title, author, price, pages, is_published"#)
        .bind(&new_book.title)
        .bind(&new_book.author)
        .bind(new_book.price)
        .bind(new_book.pages)
        .bind(new_book.is_published)
        .fetch_one(&**pool)
        .await
    {
        Ok(row) => {
            match row.try_get::<i32, _>("id") {
                // Handle successful insertion
                Ok(id) => {
                    // Construct and return a JSON response with the inserted book
                    let book = Book {
                        id,
                        title: new_book.title.clone(), // Use the provided title
                        author: new_book.author.clone(),
                        price: new_book.price as i32,
                        pages: new_book.pages as i32,
                        is_published: new_book.is_published,
                    };
                    HttpResponse::Created().json(book)
                },
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        },
        // Handle constraint violation error (duplicate title)
        Err(sqlx::Error::Database(_)) => {
            HttpResponse::BadRequest().body("Book with the same title already exists")
        },
        // Handle other errors
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Create a PostgreSQL connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").expect("postgres://postgres:postgres@localhost:5432/postgres"))
        .await
        .expect("Failed to create pool");

    // Start the Actix web server
    HttpServer::new(move || {
        App::new()
            // Pass the PostgreSQL connection pool as a web::Data to the application
            .app_data(web::Data::new(pool.clone()))
            // Define the route and handler to fetch all books
            .route("/books", web::get().to(get_books))
            // Define the route and handler to add a new book
            .route("/books", web::post().to(add_book))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


